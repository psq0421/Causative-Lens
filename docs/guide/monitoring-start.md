# v0.1.2: your first observation

[English](monitoring-start.md) · [简体中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> Leave a thread for the program to find its way home. Traces first; evidence follows.

## Install and open

Run the x64 `-dev` test installer. Installation is per-user. Open the desktop shortcut; the Start Menu CLI shortcut now keeps its terminal open after help. Use the title bar for Simplified Chinese, Traditional Chinese, English (US), or light/dark appearance. ARM64 is deferred and the test package does not carry a production Authenticode signature.

## Three steps

1. Open **Listen → Choose your software**, select OpenClaw (for example), then Next.
2. **Browse** to its project, log or installation folder. Profile scanning may include other sessions: uncheck it if unwanted. Uncheck existing traces to read only new content.
3. **Check folder**, review actual roots/files, then **Start listening**. An empty folder can wait for logs; perform a small agent task or follow the export guidance below.

An installation directory may contain no traces. A configured folder is not proof of ingestion: look for the recognized-record counter to grow. Unknown schemas and malformed JSON produce visible health counters. The observer does not launch the agent, alter its settings or call cloud APIs.

## Templates and input preparation

There are 20 choices: 17 products with two WorkBuddy channels, plus DeepSeek and Others. Detection uses content rather than an enforced version allowlist. Unknown formats are reported explicitly. These templates are not a claim that every real vendor version from the last three months has been tested.

| Software | Input preparation |
|---|---|
| Hermes Agent | Select Hermes session exports or observer logs. Export sessions first if only state.db exists. |
| Claude Code | Detects Claude messages and tool_use. Use CLI stream-json or supported transcripts; remote tools need separate verification. |
| OpenAI Codex | Detects Codex exec JSONL and session traces; private reasoning is not exposed. |
| Google Antigravity CLI | Use structured traces exported by your installed version. Gemini CLI flags are not assumed. |
| OpenClaw | Select gateway session transcripts. Gateway health logs alone do not create tool assertions; remote activity is not local ETW. |
| Manus | Select task activity exports. A cloud task report is not a per-tool trace or local system evidence. |
| AutoGPT | Select the actual worker's task logs. Verify Classic, Platform and container variants separately. |
| CodeBuddy | Read CodeBuddy stream-json or session exports while keeping normal permissions. |
| WorkBuddy（国内版） | Select CN task activity exports. This is independent of the international client; a same-name CLI is not assumed. |
| WorkBuddy International | Select international task activity exports; keep CN sessions separate. |
| Trae Agent / TRAE CLI | Select executor traces; IDE, open-source Agent and CLI protocols may differ. |
| Aider | Read chat-history Markdown or structured exports. Prose is never converted into observed file writes. |
| ZCode | Select ZCode client traces. When another client uses a Zhipu model, select that actual client. |
| Qoder CN / Qoder CLI | Select CLI stream-json or channel-specific exports. Verify CN and international variants separately. |
| CodeArts Agent CLI | Select distribution-specific structured activity exports; record region and execution host. |
| MiniMax Code CLI | For Claude/Codex using MiniMax, choose that client. For a standalone CLI select its trace directory. |
| GitHub Copilot CLI | Select standalone Copilot session events. Legacy gh, IDE and cloud agents need independent verification. |
| Cursor CLI | Select Cursor agent stream-json output, not an editor launcher. |
| DeepSeek Harness | Select normalized Harness JSONL or structured tool traces. |
| Others / 其他软件 | Auto-detect Harness, OpenAI/Anthropic tools and text. Unknown JSON is counted, not presented as a successful adapter. |

## Timeline, details and queries

- **All / Assertions / Files / Network** filter actual records; counts cover the latest 5,000 rows. Search combines with the chosen filter.
- **Source** separates agent reports, extracted intentions and kernel evidence. This listening flow has no ETW connection; verdicts remain unobserved.
- Click a row or press Enter/Space for JSON details, Escape to dismiss. Follow toggles automatic scrolling; Export saves the current JSON view.
- **Graph** displays intentions without invented evidence edges. **Query** takes a full path and requests the daemon's five-part chain.
- Stop releases the reader but keeps rows. Closing the GUI leaves the daemon reading; reopening restores its in-memory view. Exiting the daemon loses that view. Export early: JSON is not a persisted `.cllens` session.

## Others: generic input

Create `trace.jsonl` in the selected folder, with one object per line:

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

This validates trace ingestion, not actual file/network IO. JSONL waits for complete lines; JSON exports are read as whole snapshots; plain text/Markdown is displayed as text only.

## Troubleshooting and limits

Choose an accessible folder for directory errors. Export logs or explicitly include profile logs when nothing is found; SQLite `state.db` is not parsed as text. Check health details for unknown schemas, rotation, truncation and limits. Cloud, WSL and container activity is visible only through exported reports in this flow. Reinstall to repair an old CLI shortcut, or run `clens --help` in a terminal.

Scanning is bounded to depth 6, 8192 entries, 128 files, 256KiB per file/turn, 256KiB JSON snapshots, a 4096-event queue and a 5000-row view. Limits/skipped bytes are visible in health reporting.

[Detailed vendor trace preparation (v0.1.0 capability audit)](runtime-integration.md) · [Changelog](../release/CHANGELOG.md)
