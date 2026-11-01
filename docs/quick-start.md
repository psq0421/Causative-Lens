# CausativeLens v0.2.5: quick start

Language switch: [English (US)](quick-start.md) · [简体中文](快速入门.md) · [繁體中文](快速入門.md).

> A session is a message from a running program to its future self. CausativeLens keeps the envelope, timestamp, and fingerprint.

v0.2.5 is a read-only Windows observer for Agent traces. It compiles stated reasoning into causal assertions and compares those assertions with the evidence that is actually visible. Every release before `v1.0.0` is beta/test software. Observe-only is deliberate: CausativeLens does not start an Agent, change its configuration, or execute intervention.

## Install and check

Run the x64 `-dev` installer. It installs CLI, GUI, daemon, and capture per user, updates user PATH idempotently, registers `.cllens`, creates shortcuts, and checks WebView2. Only real ETW capture can request UAC.

```powershell
clens --version
clens doctor
clens doctor --json
clens --lang en-US doctor
clens --lang zh-CN doctor
clens --lang zh-TW doctor
```

## Start listening in the GUI

1. Open **Listen → Choose your software** and select OpenClaw, Claude Code, Codex, or another template.
2. Click **Browse** and choose a project, log, or installation folder. Disable “read existing recent traces” when you only want new records.
3. Click **Check folder**, inspect roots and candidate files, then click **Start listening**.

A selected folder is not proof of ingestion. Watch the recognized-record counter and health strip. Unknown schemas, malformed JSON, rotations, and truncations remain visible. The listener does not launch the Agent or call cloud APIs.

## Twenty templates

The 17 requested products include separate WorkBuddy CN and International templates, plus DeepSeek Harness and Others. Detection is content-based; there is no hard vendor-version allowlist. Cloud, WSL, container, and remote gateway activity can enter through exported local traces, but cannot be called local ETW evidence.

The detailed preparation table is in the [monitoring guide](guide/monitoring-start.md), with the [17-product integration tutorial](guide/runtime-integration.md) for per-product caveats.

## Timeline, why, and replay

- **All / Assertions / Files / Network** filter actual rows; search combines with the selected filter.
- **Source** distinguishes Agent reports, extracted intentions, and system evidence. Without ETW, verdicts remain unobserved.
- Select a row or press Enter/Space for JSON details; Escape closes the detail panel; Export saves the current JSON view.
- **Graph** shows real assertion/evidence relationships only. **Query** requests the daemon's five-part `why` chain for a full path.
- `replay` is read-only overlay/diff. Counterfactual simulation is a future direction, not a v0.2.5 claim.

## Generic Others input

Create `trace.jsonl` in the selected folder:

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

This validates trace ingestion only; it does not prove that the file or network operation occurred. JSONL waits for complete lines; plain text and Markdown are displayed as text.

## Optional model assistance

Jev, OpenAI-compatible, Anthropic-compatible, and Ollama providers can explain an existing evidence chain. They cannot change a Verdict, start a process, or execute intervention. Prefer an environment variable or stdin for API keys; never commit them to settings.

```powershell
clens assist --enable --provider jev --endpoint https://your-jev-endpoint.example/v1/chat/completions --model jev-causal
$env:CAUSATIVELENS_LLM_API_KEY = 'use only in this terminal'
clens why --path config.yaml --assist
```

## Limits

The current listening flow is primarily a runtime-trace ingress path. Real ETW capture, clean-VM proof, ARM64 runtime proof, and production signing are separate evidence gates. Scanning is bounded to depth 6, 8192 entries, 128 files, 256 KiB per file, a 4096-event queue, and a 5000-row view.

Read next: [documentation index](README.md) · [architecture](architecture/README.md) · [contracts](reference/contracts.md) · [release notes](release/v0.2.5.en-US.md).
