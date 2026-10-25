# Connecting 17 Agent / CLI products to CausativeLens

v0.1.1 adds native-log normalization, folder listening and a GUI wizard. See [your first observation](monitoring-start.md). The v0.1.0 audit below is retained for provenance and vendor trace preparation.

Language: [English (US)](runtime-integration.md) · [简体中文](runtime-integration_CN.md)

Back to [documentation](../README.md) · [quick start](../quick-start.md)

> An agent tells us what it intended. The operating system leaves a record of what happened. Evidence begins where those two timelines meet.

Scope: Windows x64, the current repository and v0.1.0 Beta. Checked **2026-10-25**. All releases before v1.0.0 are test releases. Check vendor commands against your installed help first. Running the example vendor tasks may consume your account's credits; they were not run on your behalf.

## Read this first: what works today

**The current package does not provide complete, one-command live system monitoring for these 17 products.** This guide was checked against CLI, daemon, capture and Harness source, and the packaged daemon was exercised through its Named Pipe RPC. Earlier documentation described planned wiring as completed functionality; those completion claims were too strong.

| Layer | Current state | What this establishes |
|---|---|---|
| Runtime catalog | All 17 entries; WorkBuddy has two variants, plus DeepSeek: 19 descriptors | Name recognition, not a native integration |
| Runtime protocol | All use `jsonl-tier3`; library doctor reports `tier3-jsonl-unverified` | No proof of testing every version from the past three months |
| Vendor trace acquisition | Several vendors offer streams, hooks or exports | A raw trace is not yet input received by CausativeLens |
| Vendor format → HarnessEvent | Generic parsing/tail primitives exist; vendor converters and automatic startup are not connected | Vendor JSONL cannot be imported as Harness JSONL without conversion |
| Manual HarnessEvent → RPC → assertion → why | **Verified on this computer**, reproducible in section 6 | A manual reasoning-input protocol test |
| `clens run` / `clens attach` | Currently return `status: offline` | They do not start a complete observation or launch the target agent |
| ETW → spool → resident daemon | No usable production end-to-end connection yet | Manually ingested system events are not real collection evidence |
| Persistence / GUI subscription | Not established by this test | RPC sessions are in memory, not saved `.cllens` files |

`clens doctor` can return `status: ok` through an offline fallback. Check for an actual `daemon: connected` result. Even that proves reachability only. There is no usable `clens adapters doctor` CLI subcommand today.

The guide distinguishes three outcomes:

- **Raw trace ready:** vendor logs, version and execution environment were recorded.
- **Reasoning input verified:** explicitly normalized events reached a specific RPC session and yielded assertions.
- **Complete monitoring:** real system events arrive continuously, align with the trace, and survive save/reopen. The current package cannot claim this outcome.

## Contents

- [1. Shared preparation](#prepare)
- [2. Integration routes for all 17 products](#routes)
- [3. The five priority products](#priority)
- [4. The remaining 12 entries](#others)
- [5. Converting records to HarnessEvent](#normalize)
- [6. A reproducible manual RPC walkthrough](#rpc)
- [7. Acceptance criteria](#acceptance)
- [8. Troubleshooting](#troubleshooting)
- [9. Implementation priorities](#next)
- [10. Official sources and code evidence](#sources)

<a id="prepare"></a>
## 1. Shared preparation

### 1.1 Locate the actual executor

| Execution environment | Windows host visibility | What must not be inferred |
|---|---|---|
| Native Windows CLI / worker | Windows process tree once capture is connected | Every tool request necessarily performed its requested IO |
| WSL2 Linux | Some host and boundary activity | Complete Linux process/file observations from Windows ETW |
| Linux Docker container | Host boundary activity and exported logs | Container syscalls are Windows file events |
| SSH / remote gateway / cloud agent | Local client, network boundary and exported remote traces | A remote edit is a local file write |
| Manus / web agent in a browser | Local browser activity | The browser PID is the cloud agent's process root |

**Follow the executor.** Record the execution host for MCP, SSH and remote tools. A local client does not imply local tools. This guide does not deliver a Linux, WSL or macOS collector.

### 1.2 Resolve the installed binaries

Use a standard-user PowerShell:

~~~powershell
$ErrorActionPreference = 'Stop'
$clensExe = (Get-Command clens.exe -ErrorAction Stop).Source
$installDir = Split-Path -Parent $clensExe
$daemonExe = Join-Path $installDir 'clens-daemon.exe'
& $clensExe --version
& $clensExe doctor --json
Test-Path -LiteralPath $daemonExe
~~~

Reopen the terminal if PATH is stale. Elevating the GUI or daemon does not implement missing run/attach wiring. Capture is intended to be the sole elevated component.

### 1.3 Separate each sample and use UTF-8

The examples below reuse `$runDir`:

~~~powershell
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$suffix = [Guid]::NewGuid().ToString('N').Substring(0, 8)
$runDir = Join-Path $env:LOCALAPPDATA "causativelens\integration\$stamp-$suffix"
New-Item -ItemType Directory -Path $runDir -Force | Out-Null
$utf8NoBom = [Text.UTF8Encoding]::new($false)
~~~

PowerShell 7 is recommended. Windows PowerShell 5.1 redirection can create UTF-16, which is unsuitable for UTF-8 JSONL. This function writes and flushes each line. It **only saves raw output**; it is not an adapter or background tail service.

~~~powershell
function Save-Utf8Lines {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [AllowEmptyString()][string]$Line,
        [Parameter(Mandatory = $true)][string]$Path
    )
    begin {
        $encoding = [Text.UTF8Encoding]::new($false)
        $writer = [IO.StreamWriter]::new($Path, $false, $encoding)
    }
    process {
        $writer.WriteLine($Line)
        $writer.Flush()
    }
    end { $writer.Dispose() }
}
~~~

Use this for small samples and check the final line after interruption. Keep stderr separate: merging `2>&1` can corrupt JSONL. After each vendor pipeline, before running another native executable, save its exit code:

~~~powershell
$vendorExitCode = $LASTEXITCODE
[IO.File]::WriteAllText((Join-Path $runDir 'exit-code.txt'), [string]$vendorExitCode, $utf8NoBom)
~~~

A nonempty log does not prove success. Check nonzero exits, banners, ANSI escapes and progress output.

### 1.4 Record provenance

Replace the version placeholder with the actual installed version. Make a separate record for each product and distribution channel:

~~~powershell
$metadata = [ordered]@{
    runtime = 'claude-code'
    reported_version = 'REPLACE_WITH_ACTUAL_VERSION'
    sampled_at_utc = [DateTime]::UtcNow.ToString('o')
    execution_environment = 'windows-native'
    execution_host = $env:COMPUTERNAME
    working_directory = (Get-Location).Path
    source_kind = 'vendor-stdout'
    compatibility = 'unverified'
}
[IO.File]::WriteAllText((Join-Path $runDir 'metadata.json'), ($metadata | ConvertTo-Json -Depth 8), $utf8NoBom)
~~~

Also record CLI/IDE/Desktop/SDK, region, profile and vendor session ID. Redact tokens, prompts, usernames, source and tool results before sharing. Metadata-only system collection does not automatically redact Harness logs.

<a id="routes"></a>
## 2. Integration routes for all 17 products

An available vendor export is **not a completed native CausativeLens live adapter**.

| No. | Product | Preferred raw source | Recommended approach |
|---|---|---|---|
| 01 | **Hermes Agent** | sessions export; Observer Hooks | Export first, then a read-only observer |
| 02 | **Claude Code and related products** | CLI stream-json; hooks/transcript | Verify CLI and other surfaces separately |
| 03 | **OpenAI Codex** | `codex exec --json` | Verify exec and Desktop separately |
| 04 | Antigravity CLI | Distribution-supported export/hook | Confirm product identity first |
| 05 | **OpenClaw** | Gateway JSON logs + session transcript | Keep the two sources separate |
| 06 | **Manus** | Task export/API/webhook if available | State cloud visibility limits |
| 07 | AutoGPT | Executor events / platform task logs | Separate Classic, Platform and containers |
| 08 | CodeBuddy | CLI stream-json; separate IDE export | Convert its own schema |
| 09 | WorkBuddy CN / international | Each variant's task export/log | Independent version fixtures |
| 10 | Trae Agent / TRAE CLI | Help-confirmed output / IDE logs | Separate IDE, CLI and open-source agent |
| 11 | Aider | Chat and LLM histories | Text, not Harness JSONL |
| 12 | ZCode | Distribution-specific tool trace | Confirm the actual client |
| 13 | Qoder CN / CLI | CLI stream-json; separate IDE export | Verify command and region |
| 14 | CodeArts Agent CLI | Distribution-specific event log | Record region and build |
| 15 | MiniMax Code CLI | Actual client's output / standalone logs | A model provider is not a runtime identity |
| 16 | GitHub Copilot CLI | Official logging/monitoring and session export | Separate standalone, legacy gh and IDE |
| 17 | Cursor CLI | `agent` stream-json | Verify that agent belongs to Cursor |

Compatibility is recorded and warned about, without a forced version block. `recent_window_months=3` is a test target, not evidence that every recent release passed.

<a id="priority"></a>
## 3. The five priority products

### 01. Hermes Agent

**A. Confirm the installed surface.** CLI, TUI and Desktop may use a gateway that owns execution; a terminal PID is not necessarily the agent root.[H1][H2]

~~~powershell
Get-Command hermes -ErrorAction Stop | Select-Object Name, Source
hermes --version
hermes chat --help
hermes sessions --help
hermes sessions export --help
~~~

If installed only in WSL, run vendor commands there and label the environment `wsl2`. Exporting to Windows does not provide Linux system events.

**B. Generate a small session** in a test directory containing README.md, using the product's existing authentication and permissions:

~~~powershell
hermes chat -q "Read README.md and summarize it. Do not edit files."
hermes sessions list
~~~

Record the actual session ID, especially with concurrent gateway tasks.

**C. Export that session**, after checking the installed help:

~~~powershell
$hermesSessionId = Read-Host 'Hermes session ID'
$hermesExport = Join-Path $runDir 'hermes-session-export.jsonl'
hermes sessions export $hermesExport --session-id $hermesSessionId --redact
~~~

The default export has **one session object per line, containing nested messages**, not one HarnessEvent per line. Expand sessions → messages → tool calls. A snapshot is useful offline; repeatedly overwriting it and tailing it is not a reliable live stream.

**D. For live traces, use the official Observer Hooks contract.** It includes `pre_tool_call` / `post_tool_call` and `hermes.observer.v1` telemetry.[H3]

| Source field | Conversion purpose |
|---|---|
| session_id / task_id | Session and task separation |
| turn_id / tool_call_id / api_request_id | Correlation, deduplication and subtask identity |
| tool_name / args | Canonical tool and argument mapping |
| post_tool_call result/status/error/duration_ms | Separate result events and health |
| Vendor timestamp / receive time | Keep distinct; never silently substitute |

A read-only observer needs bounded queuing, must not block the agent, and should fail open. It must not return behavioral approval, denial, argument changes or termination decisions. **The Beta does not package a Hermes→CausativeLens observer plugin or an installation command for it.** This is a future integration point.

**Current completion criterion:** a specified session export plus correlated call/result samples, normalized under section 5 and tested under section 6. Identify the actual gateway/WSL executor separately.

### 02. Claude Code and related products

**A. Verify the executable.**

~~~powershell
Get-Command claude -ErrorAction Stop | Select-Object Name, Source
claude --version
claude --help
~~~

**B. Save the official non-interactive message stream**, using existing permissions.[C1]

~~~powershell
$raw = Join-Path $runDir 'claude.raw.jsonl'
$err = Join-Path $runDir 'claude.stderr.log'
claude -p "Read README.md and summarize it. Do not edit files." --output-format stream-json --verbose 2> $err | Save-Utf8Lines -Path $raw
~~~

If access is denied, preserve the denial. Authorize the test directory through Claude's normal process or use an interactive session sample. Do not disable permissions to collect logs.

**C. Convert real messages, not just the final answer.**

| Source | Canonical mapping |
|---|---|
| Assistant text block | Message text; not proof that private reasoning is exposed |
| assistant.message.content[] tool_use | tool_call; preserve block ID |
| Read + input.file_path | read + arguments.path |
| Edit + input.file_path | edit + arguments.path |
| Write + input.file_path | write; not necessarily first-time create |
| Bash + input.command | shell + arguments.cmd |
| tool_result, including failure/cancellation | tool_result; use tool=null in this Beta, section 5 |
| parent_tool_use_id | Subagent relation, not an unrelated root tool |

These are mapping rules requiring fixtures from the sampled version, not a packaged converter. A final result alone cannot reconstruct file operations.

**D. Verify CLI, IDE, Desktop and SDK independently.** Hooks expose fields such as `transcript_path`, `tool_name` and `tool_input` for locating and correlating traces.[C2] Use the relevant version's read-only export mechanism without replacing existing hooks. Desktop MCP and remote tools are separate execution surfaces.

**Completion criterion:** correlated tool call/results and assertions through RPC. Actual IO still needs real system collection.

### 03. OpenAI Codex

This section followed **OpenAI Docs** source checking. Official pages returned `403` on this network, so flags were additionally checked against the installed CLI: `codex-cli 0.164.0-alpha.11.1`. Only help/version queries were run, **not model tasks**.[O1]

~~~powershell
Get-Command codex -ErrorAction Stop | Select-Object Name, Source
codex --version
codex exec --help
~~~

If Desktop is installed but PATH lacks codex, locate its actual CLI. Do not hardcode a version-specific application directory.

~~~powershell
$raw = Join-Path $runDir 'codex.raw.jsonl'
$err = Join-Path $runDir 'codex.stderr.log'
codex exec --json --sandbox read-only "Read README.md and summarize it. Do not edit files." 2> $err | Save-Utf8Lines -Path $raw
~~~

`--json` is the event stream. `--output-schema` constrains the final answer and does not replace execution telemetry. Do not bypass the sandbox or approvals for logging.

Conversion requirements:

- Retain thread/turn/item identity and distinguish messages from executions.
- Map command execution to shell only with actual command data. Correlate start/completion/failure rather than producing duplicate intentions.
- Patch summaries are client reports, not independently observed FileWrite events.
- Tool output containing “read X” is not a new read request.
- Verify Desktop/IDE, background and remote execution separately.
- Avoid `--ephemeral` when relying on persisted vendor session storage.

**Completion criterion:** versioned raw events, correlated execution states and explicit normalization. Do not claim access to hidden reasoning when it is not exposed.

### 05. OpenClaw

**A. Locate the gateway.**

~~~powershell
Get-Command openclaw -ErrorAction Stop | Select-Object Name, Source
openclaw --version
openclaw logs --help
~~~

Record profile, gateway host and tool execution environment. Keep tokens in the product's existing credential mechanism.

**B. Save the gateway JSON stream.** This is an official entry point.[W1]

~~~powershell
$raw = Join-Path $runDir 'openclaw.gateway.raw.jsonl'
$err = Join-Path $runDir 'openclaw.stderr.log'
openclaw logs --follow --json 2> $err | Save-Utf8Lines -Path $raw
~~~

Run a test task in another terminal. Stop with Ctrl+C and check the last line. Deduplicate reconnects and log changes using vendor cursors or stable IDs.

**C. Obtain the session transcript separately.** Locate the specified session using actual deployment configuration and the official session documentation.[W2] Gateway logs can contain only lifecycle notices and errors. This guide has not verified a universal transcript path.

**D. Join the sources by session/task/tool-call identity.** Notices are system events; only actual tool requests become tool_call. Gateway health is not tool success. Internal operations on Linux, Docker or remote hosts are not local Windows evidence.

**Completion criterion:** real session tool activity, not merely “gateway started”, with version, profile, host and cursor provenance.

### 06. Manus

**No `manus.exe --json` command has been verified here.** The official platform documents Task and Webhook entry points. This review did not establish account eligibility, concrete API parameters or complete tool schemas.[M1]

1. Record task ID, client version and execution location; distinguish cloud tasks and local connectors.
2. Export the specified task/activity if the product supports it. Preserve steps, times and tool IDs. A final report is not a per-step trace.
3. If your account exposes APIs/webhooks, follow current official documentation for your own tasks, including signature verification, deduplication and receive times. This guide invents no endpoint and starts no public receiver.
4. A task-completed notification is system status. Only actual structured tool records become tool_call.
5. If a local connector performs operations, identify its worker. The browser is not the cloud sandbox process root.

**Completion criterion:** task-attributed samples with explicit visibility. Missing tool details mean limited trace preparation, not complete causal monitoring.

<a id="others"></a>
## 4. The remaining 12 entries

Each still needs raw samples → section 5 normalization → section 6 RPC. Unverified flags are not presented as working commands.

### 04. Google Antigravity CLI

1. Identify IDE, standalone CLI or unofficial wrapper; record publisher, version and executable.
2. Inspect that executable's help for log/export/headless/hooks. **Do not substitute Gemini CLI flags.**
3. Obtain a read-only tool call/result from a structured stream, or an offline IDE activity export.
4. Identify local/remote/sandbox workers; the IDE PID is not necessarily the executor.

No verified CLI/schema contract was established in this review. Completion requires replayable tool samples; screenshots are not machine-verifiable events.

### 07. AutoGPT

1. Separate Classic, Platform and cloud tasks. Record package/repository version, worker and task ID.
2. For an executor you control, export tool names, arguments, times and result IDs from existing callbacks/events. That exporter is not included in this Beta.
3. For containers, save diagnostic logs using your actual container name:

~~~powershell
$containerName = Read-Host 'Actual AutoGPT worker container name'
docker logs --follow --timestamps $containerName 2> (Join-Path $runDir 'autogpt.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'autogpt.container.log')
~~~

4. Docker logs are text, not a guaranteed tool schema. They do not expose Linux container syscalls to Windows ETW.

Completion requires correlated task/worker/tool activity, not only web administration logs.

### 08. CodeBuddy

Official documentation confirms `codebuddy`, `-p` and `--output-format stream-json` independently of Claude.[B1][B2]

~~~powershell
Get-Command codebuddy -ErrorAction Stop | Select-Object Name, Source
codebuddy --version
codebuddy --help
$raw = Join-Path $runDir 'codebuddy.raw.jsonl'
$err = Join-Path $runDir 'codebuddy.stderr.log'
codebuddy -p "Reply with the word READY. Do not use tools." --output-format stream-json 2> $err | Save-Utf8Lines -Path $raw
~~~

This tests the stream only, with **no file evidence**. Non-interactive tools needing authorization may be blocked. Use normally authorized interactive transcripts or version-supported read-only hooks rather than bypassing permissions.

Record CN/international/enterprise authentication and separate IDE/CLI. Background terminal states can arrive after a turn's result: the first result is not necessarily the end of every task. Similar-looking messages still need independent fixtures.

Completion requires correlated actual tool activity, not merely READY.

### 09. WorkBuddy CN / international

1. Use separate `workbuddy-cn` and `workbuddy-international` directories; record full build, channel, region and executor.
2. Run a read-only task in each and obtain its supported activity/task/diagnostic export. A universal JSONL flag or log path was not verified.
3. Even if CodeBuddy executes its tools, prove the relation using worker PID and session/tool IDs before reusing a schema.
4. Separate desktop, remote, mobile and mini-program surfaces. A final report remains a report.

Catalog aliases are not executable names. Each variant needs its own real call/result fixture.

### 10. Trae Agent / TRAE CLI

1. Distinguish IDE, standalone CLI and open-source agent, retaining distribution/repository provenance.
2. Use help-confirmed CLI streams, supported IDE logs, or a read-only exporter at the open-source executor's existing event outlet.
3. Separate plans, requests, results and subtask IDs. A plan file is not execution evidence.
4. Record the worker host/workspace for background and remote execution.

No single verified command covers all three surfaces. Completion requires correlated calls/results and a known environment.

### 11. Aider

Official options expose chat and LLM histories, which are Markdown/text, not Harness JSONL.[A1]

~~~powershell
aider --version
aider --help
aider --message "Read README.md and summarize it. Do not edit files." --dry-run --chat-history-file (Join-Path $runDir 'aider.chat.md') --llm-history-file (Join-Path $runDir 'aider.llm.log')
~~~

Use dry-run for the initial sample. Histories can contain source/prompts. Distinguish conversation, proposed diffs and applied operations; “I will edit X” is not a write event.

Preserve text when no structured tool records exist. This Beta has no connected local-model backend that automatically fills the gap. Completion requires explicit source/version/workspace/confidence, without turning proposals into system FileWrite evidence.

### 12. ZCode

1. Identify the actual ZCode client. Claude/Codex configured with a Zhipu model still uses the original client's adapter.
2. Find export/log/hooks through that version's help. Fixed flags/directories were not verified.
3. Obtain a read request and result, mapping real arguments to read/path.
4. Record provider, client version and execution host independently.

Completion requires real samples establishing both identity and schema.

### 13. Qoder CN / CLI

Current official script documentation uses `qoder` and `--output-format stream-json`. Existing installations may expose different entry points; verify source/help.[Q1]

~~~powershell
Get-Command qoder,qodercli -ErrorAction SilentlyContinue | Select-Object Name, Source
# Use qoder below only if it is your verified executable.
qoder --version
qoder --help
qoder -p "Reply with the word READY. Do not use tools." --output-format stream-json 2> (Join-Path $runDir 'qoder.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'qoder.raw.jsonl')
~~~

This tests streaming only. Use ordinary permission flows or preauthorized read-only tasks for real tools. Headless operation may deny requests requiring confirmation; do not enable bypass mode for sampling.

Separate CN/international, IDE/CLI/cloud. A single final JSON object is not a continuous trace. International documentation does not certify a CN build. Completion requires fixtures for the actual channel.

### 14. CodeArts Agent CLI

1. Record exact product, channel, Huawei Cloud region and version. Do not confuse plugins or model APIs with this client.
2. Inspect installed help; use only supported structured output/diagnostic export.
3. Capture arguments, results, times and session ID from a read-only task; preserve unstructured logs as text.
4. Mark cloud workspaces/containers remote, not as the local CLI PID.

Universal flags were not verified across distributions. Completion requires versioned real protocol samples.

### 15. MiniMax Code CLI

1. Establish whether this is a standalone CLI or Coding Plan/API used by another client.
2. For Claude/Codex/Aider with MiniMax, follow the **client's section** and record model_provider separately.
3. For an actual standalone distribution, establish official source, executable and help before selecting its real stream/export.
4. Identify local, cloud or SDK execution.

Completion requires an identified client and parseable tool activity. Model-request logs are not system behavior.

### 16. GitHub Copilot CLI

Official documentation establishes standalone `copilot` and logging/monitoring help.[G1][G2]

~~~powershell
Get-Command copilot -ErrorAction Stop | Select-Object Name, Source
copilot --help
copilot help logging
copilot help monitoring
~~~

Record the version using installed help. Official `copilot version` may also check for updates; prefer a help-supported version flag for offline evidence.

1. Obtain supported logs/exports via that help, without inventing a JSON event flag.
2. Sample an authorized read-only task, retaining session/tool IDs and results separately from final conversation.
3. Verify legacy `gh copilot`, IDE and cloud coding agent independently.
4. Mark delegated cloud execution remote.

Completion requires traceable standalone tool records. Configuration JSON such as config --list --json is not execution telemetry.

### 17. Cursor CLI

Official headless guidance uses `agent`; older installs may expose `cursor-agent`. Verify Source because “agent” is generic; “cursor” is commonly the editor launcher.[U1]

~~~powershell
Get-Command agent,cursor-agent -ErrorAction SilentlyContinue | Select-Object Name, Source
# Continue only after confirming agent is Cursor's executable.
agent --version
agent --help
agent -p --output-format stream-json "Read README.md and summarize it. Do not edit files." 2> (Join-Path $runDir 'cursor.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'cursor.raw.jsonl')
~~~

Do not add `--force`. Separate assistant/call/result and retain correlation IDs. Distinguish answers/proposed patches from applied writes. Verify IDE and Background/Cloud execution locations independently.

Completion requires the right executable and correlated tool samples. Text-only or denied requests do not establish successful tool trace acquisition.

<a id="normalize"></a>
## 5. Converting records to HarnessEvent

### 5.1 Keep original and normalized data

~~~text
<runDir>\
  metadata.json
  <vendor>.raw.jsonl        # Or the original export / text / Markdown
  <vendor>.stderr.log
  normalized.harness.jsonl
  rejected.jsonl
  mapping-version.txt
~~~

**A ready-made converter for all 17 vendor formats is not included.** Manually normalize a real record first. Renaming a file or directly ingesting vendor JSONL is insufficient.

### 5.2 Minimum record

This is a **protocol example, not a real agent trace**:

~~~json
{"schema_v":1,"ts_utc_ms":1792886400000,"step":1,"phase":"tool_call","tool":{"name":"read","arguments":{"path":"C:\\demo\\config.yaml"}},"text":"DOCS_EXAMPLE_NOT_AGENT_OUTPUT","harness_log_id":20261025,"line_no":1}
~~~

| Field | Requirement |
|---|---|
| schema_v | Integer 1 |
| ts_utc_ms | Unix UTC milliseconds; label substituted receive time as estimated in sidecar metadata |
| step | Stable under the conversion rule |
| phase | reasoning / tool_call / tool_result / system / unknown |
| tool | {name,arguments} only for actual requests; otherwise null |
| text | Source text, not invented operations; label synthetic examples |
| harness_log_id / line_no | Stable identity for the normalized source |
| byte_start / byte_end | Optional byte ranges; zero does not establish precise provenance |
| raw_json | Optional **string**, not a nested object |

Do not infer exact execution times from mtime. Record cross-host clock uncertainty rather than adjusting timestamps to manufacture matches.

IDs currently use `(harness_log_id,max(line_no,step),slot)`. Keep normalized line_no stable and at least as large as step so several records do not collapse onto one larger step value. Split multiple tool blocks on one raw line into independently, stably numbered normalized records, retaining the raw line/block mapping in a sidecar. Internal slot numbers distinguish assertions generated from one normalized event; they do not solve source block identity automatically.

### 5.3 Rule coverage and limits

| Canonical tool | Arguments | Current extraction |
|---|---|---|
| read | path or target | Read |
| write | path or target | Write |
| edit | path or target | Read precondition + ordered Write |
| create | path or target | Create |
| delete | path or target | Delete |
| shell / exec | cmd or command | Limited rm/mv/cp/curl/ssh templates |
| connect / bind / listen | Target fields | Recognized names, incomplete endpoint normalization |
| Other | Retain source | May yield no assertion; does not establish completion |

Templates are not a complete PowerShell/bash parser. Complex quoting, pipes, aliases, redirection and spaced paths need separate verification.

**Important Beta boundaries:**

- RPC ingest deserializes HarnessEvent directly, bypassing generic parser quarantine, unknown-shape preservation and sentinel handling. Unknown phase values must be converted explicitly.
- Extraction is not sufficiently phase-gated. Repeating the tool on a result can duplicate assertions; use `tool=null` for results and retain linkage in source/sidecar records.
- “Wrote file” text is not system evidence. The extractor-llm SPI has no connected built-in ONNX backend to cover every client.
- Keep the complete RPC envelope under approximately 64 KiB. Store large outputs in source files with references; do not truncate JSON or silently discard errors.

<a id="rpc"></a>
## 6. A reproducible manual RPC walkthrough

This verifies **manual reasoning input only**. It launches no agent, ETW or UAC and generates no real file evidence. Run from the repository root; package-only users also need the helper below.

### 6.1 Start a standard-user daemon

Close your own GUI connection for this test without terminating someone else's processes. The current daemon serves a single client connection at a time; the helper closes after every request.

~~~powershell
$clensExe = (Get-Command clens.exe -ErrorAction Stop).Source
$daemonExe = Join-Path (Split-Path -Parent $clensExe) 'clens-daemon.exe'
if (-not (Test-Path -LiteralPath $daemonExe)) {
    throw 'clens-daemon.exe is missing next to clens.exe.'
}
if (-not (Get-Process -Name clens-daemon -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath $daemonExe -WindowStyle Hidden | Out-Null
}
~~~

For a repository-payload check, set $clensExe to the actual installer/payload/clens.exe path first. Label payload and installed-build evidence separately.

### 6.2 Load the helper and open a session

[harness-rpc.ps1](examples/harness-rpc.ps1) wraps the existing protocol with timeouts, length checks and errors. It provides neither automatic tail nor vendor conversion.

~~~powershell
. .\docs\guide\examples\harness-rpc.ps1
$hello = Invoke-CausativeLensRpc -Method 'hello' -Params @{
    protocol_version = 1
    mode = 'observe'
}
$hello | ConvertTo-Json -Depth 8
$ticket = Invoke-CausativeLensRpc -Method 'session.open' -Params @{}
$sessionId = $ticket.session_id
$sessionId
~~~

Expect product=CausativeLens, mode=observe, protocol_version=1 and a nonempty ticket. The current wire request is one line containing `{id,method,params}`. Do not invent subscription methods.

### 6.3 Send a labeled synthetic request, query and clean up

~~~powershell
$demoPath = 'C:\causativelens-docs-demo\config.yaml'
try {
    $event = [ordered]@{
        schema_v = 1
        ts_utc_ms = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        step = 1
        phase = 'tool_call'
        tool = @{
            name = 'read'
            arguments = @{ path = $demoPath }
        }
        text = 'DOCS_RPC_DEMO_NOT_AGENT_OUTPUT'
        harness_log_id = 20261025
        line_no = 1
    }
    $ingest = Invoke-CausativeLensRpc -Method 'session.ingest_harness' -Params @{
        session_id = $sessionId
        event = $event
    }
    $status = Invoke-CausativeLensRpc -Method 'session.status' -Params @{
        session_id = $sessionId
    }
    $why = Invoke-CausativeLensRpc -Method 'query.why' -Params @{
        session_id = $sessionId
        path = $demoPath
    }
    $ingest | ConvertTo-Json -Depth 10
    $status | ConvertTo-Json -Depth 10
    $why | ConvertTo-Json -Depth 20
} finally {
    Invoke-CausativeLensRpc -Method 'session.close' -Params @{
        session_id = $sessionId
    } | Out-Null
}
~~~

Expect a nonnull assertion_id and active=true, but events_seen=0, spool_frames=0, unobserved and empty evidence. **This is the honest expected result.** A read assertion does not prove the file exists or was read. Do not fabricate ingest_system records to make the result appear fulfilled.

Pass session_id explicitly: current CLI why lacks the ticket and may select another in-memory session; watch/timeline cannot reliably display this session either.

Closing releases memory and does not save a file. Restarting the daemon loses the state. Stop only a daemon you started and know is unused.

### 6.4 Finite import of normalized JSONL

This accepts only section 5's converted data. It is an **offline finite import, not a live vendor monitor**.

~~~powershell
$normalizedPath = Join-Path $runDir 'normalized.harness.jsonl'
if (-not (Test-Path -LiteralPath $normalizedPath)) {
    throw 'Normalize vendor records before importing them.'
}
$ticket = Invoke-CausativeLensRpc -Method 'session.open' -Params @{}
$sessionId = $ticket.session_id
$lineNo = 0
try {
    foreach ($line in [IO.File]::ReadLines($normalizedPath, [Text.Encoding]::UTF8)) {
        $lineNo++
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $event = $line | ConvertFrom-Json -ErrorAction Stop
        Invoke-CausativeLensRpc -Method 'session.ingest_harness' -Params @{
            session_id = $sessionId
            event = $event
        } | ConvertTo-Json -Depth 8 -Compress
    }
    Invoke-CausativeLensRpc -Method 'session.status' -Params @{
        session_id = $sessionId
    } | ConvertTo-Json -Depth 10
    # Query with query.why and the same session_id here, before closing.
} catch {
    throw ("Import stopped at normalized line {0}: {1}" -f $lineNo, $_.Exception.Message)
} finally {
    Invoke-CausativeLensRpc -Method 'session.close' -Params @{
        session_id = $sessionId
    } | Out-Null
}
~~~

Stop on errors and replay corrected records in a new session. RPC return values are not a persisted complete session.

<a id="acceptance"></a>
## 7. Acceptance criteria

| Layer | Required evidence | This review |
|---|---|---|
| Vendor source | Version, session ID, records and official flags | Some entry points checked; no 17-product agent execution |
| Encoding | UTF-8 JSON per line, stderr separate | Instructions supplied; each sample still needs validation |
| Conversion | Golden fixtures for results, subtasks, unknowns, timing and source identity | Complete version fixtures are missing |
| RPC | hello/ticket/ingest/status/why/close | **Packaged daemon smoke test passed locally** |
| Continuity | Append, rotation, truncate, partial lines, reconnect deduplication, visible errors | Some library primitives exist; automatic wiring not accepted |
| Collection | Known local operations yield real seq/pid/path and explained gaps | Complete resident pipeline unverified |
| Causality | Assertions, real events and why agree; denials are not fulfilled | Synthetic events cannot substitute |
| Persistence / GUI | Reopen consistency, shared ticket, restart recovery | Required end-to-end connection incomplete |
| Latency | Clock-anchored occurrence-to-queryable P50/P99 | Not measured; targets P50≤400ms, P99≤1.5s |

Actual smoke result: observe / 0.1.0, assertion received, events_seen=0, unobserved, zero evidence; `synthetic_harness_only=true`, `real_agent_test=false`, `etw_started=false`. The developer-machine report is in git-ignored `target/runtime-guide-smoke.json`.

The final guide's exact section 6.2–6.4 blocks were executed, including finite import, UTF-8 Chinese roundtrip, malformed-line reporting, the 64 KiB request limit, RPC error propagation and session cleanup. All passed; evidence is in local `target/runtime-guide-verification.json`. No vendor model task or ETW collection was started, and these checks do not certify 17 real vendor integrations.

Future real acceptance should use a dedicated test directory: read an existing file, approve a test write, deny a permission request, run one child process, then exit/recover. Compare each with genuine system metadata. Initial tutorial sampling stays read-only.

<a id="troubleshooting"></a>
## 8. Troubleshooting

| Symptom | Explanation / next step |
|---|---|
| run/attach offline with exit 0 | Current placeholder behavior; elevation/reinstallation cannot supply implementation |
| doctor ok without events | Check connected; offline fallback exists |
| Raw JSONL missing fields | Vendor schema is not HarnessEvent |
| Null assertion_id | Unsupported tool, text-only event or missing mapping; retain source |
| Duplicate assertions | Check call/result duplication, reconnects and block identity |
| Zero quarantine/schema/truncate | Direct RPC bypasses parser/sentinel; zeros prove no coverage |
| Unobserved / empty evidence | Expected without system events; success text is not evidence |
| RPC timeout | Missing daemon, conflicting process or occupied long connection |
| Unknown phase rejected | Explicitly convert and preserve raw source |
| GUI lacks the session | Ticket/subscription wiring is incomplete; verify by explicit RPC ticket |
| Data disappears on restart | In-memory sessions; open/close are not SQLite load/save |
| Cloud actions lack local events | Check executor location, not browser PID |
| Only Markdown/final answer | Preserve text; no proof of structured tools or system actions |

<a id="next"></a>
## 9. Implementation priorities

These are missing implementation tasks, not available commands. Retain one daemon, two frontends and observe-only operation:

1. Connect capture lifetime, event consumption, clocks/spool recovery and session save/reopen.
2. Connect automatic tail/stdout with shared quarantine, rotation, deduplication and schema sentinel; fix phase gating.
3. Prioritize Hermes observer, Claude CLI/hooks, Codex exec and OpenClaw transcript. Limit Manus input to actual exposed detail.
4. Add real vendor fixtures rather than treating aliases as support.
5. Bind GUI/CLI to the same ticket and disclose unverified compatibility and observation limits.
6. Display real event counts, gaps and update times, then measure latency and performance.

Leave missing evidence visible. A telescope does not paint a star into light that has not yet arrived.

<a id="sources"></a>
## 10. Official sources and code evidence

These establish entry points, not certification of every release:

- [H1 — Hermes CLI](https://hermes-agent.nousresearch.com/docs/user-guide/cli)
- [H2 — Hermes Sessions](https://hermes-agent.nousresearch.com/docs/user-guide/sessions)
- [H3 — Hermes Observer Hooks](https://hermes-agent.nousresearch.com/docs/developer-guide/observer-hooks)
- [C1 — Claude Code headless](https://code.claude.com/docs/en/headless)
- [C2 — Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [O1 — OpenAI Codex non-interactive](https://developers.openai.com/codex/noninteractive) and [CLI reference](https://developers.openai.com/codex/cli/reference/). Network returned 403; installed exec help supplied flag evidence.
- [W1 — OpenClaw logs](https://docs.openclaw.ai/cli/logs)
- [W2 — OpenClaw sessions](https://docs.openclaw.ai/concepts/session)
- [M1 — Manus Open Platform](https://open.manus.im/docs/)
- [A1 — Aider options](https://aider.chat/docs/config/options.html)
- [B1 — CodeBuddy quick start](https://www.codebuddy.cn/docs/cli/quickstart)
- [B2 — CodeBuddy headless](https://www.codebuddy.cn/docs/cli/headless)
- [Q1 — Qoder Run in Scripts](https://docs.qoder.com/cli/run-in-scripts)
- [G1 — Copilot CLI usage](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/overview)
- [G2 — Copilot CLI reference](https://docs.github.com/en/copilot/reference/cli-command-reference)
- [U1 — Cursor headless](https://cursor.com/docs/cli/headless)

Repository evidence: [CLI dispatch](../../crates/clens-cli/src/lib.rs), [runtime catalog/extraction](../../crates/clens-harnessd/src/lib.rs), [HarnessEvent](../../crates/clens-types/src/lib.rs), [daemon RPC](../../crates/clens-daemon/src/lib.rs), [capture entry](../../crates/clens-capture/src/main.rs), [open questions](../plan/open-questions.md).

This guide neither updates releases nor installs vendor plugins or sends vendor tasks. It delivers documentation and a manual protocol helper.
