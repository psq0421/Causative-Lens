# v0.2.5: your first observation

[English](monitoring-start.md) · [简体中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> Leave a thread for the program to find its way home. Traces first; evidence follows.

## Install and open

Run the x64 `-dev` test installer. Installation is per-user. Open the desktop shortcut; the Start Menu CLI shortcut keeps its terminal open after help. Use the title bar to switch between English (US), Simplified Chinese, Traditional Chinese, and light/dark appearance. ARM64 is deferred and the test package has no production Authenticode signature.

## Three steps

1. Open **Listen → Choose your software**, select OpenClaw, Claude Code, Codex, or another template, then choose **Next**.
2. Click **Browse** and select the project, log, or installation folder. Uncheck “read existing recent traces” when you only want newly appended records.
3. Click **Check folder**, review the actual roots and candidate files, then click **Start listening**. An empty folder can wait for logs; perform a small agent task or use the generic JSONL example below.

A configured folder is not proof of ingestion. Watch the recognized-record counter and health strip. Unknown schemas and malformed JSON remain visible. The observer does not launch the agent, alter its settings, or call cloud APIs.

## Templates and input preparation

There are 20 choices: 17 products with two WorkBuddy channels, plus DeepSeek Harness and Others. Detection uses content rather than an enforced version allowlist. Unknown formats are reported explicitly. These templates do not claim that every real vendor version from the last three months has been tested.

| Software | Input preparation |
|---|---|
| Hermes Agent | Select session exports or observer logs. Export sessions first if only `state.db` exists. |
| Claude Code | Use message/tool_use, CLI stream-json, or supported transcripts. |
| OpenAI Codex | Use exec JSONL and session traces; private reasoning is not exposed. |
| Google Antigravity CLI | Use structured traces exported by your installed version. |
| OpenClaw | Select gateway session transcripts; health logs alone do not create tool assertions. |
| Manus | Select task activity exports; a cloud report is not local system evidence. |
| AutoGPT | Select the actual worker task logs and verify the Classic/Platform/container variant. |
| CodeBuddy | Use stream-json or session exports. |
| WorkBuddy (CN) | Use CN task activity exports; keep them separate from the international client. |
| WorkBuddy International | Use international task activity exports. |
| Trae Agent / TRAE CLI | Use executor traces; IDE, open-source Agent, and CLI protocols may differ. |
| Aider | Use chat-history Markdown or structured exports. Prose is never converted into observed file writes. |
| ZCode | Use ZCode client traces; choose the actual client when another client uses a Zhipu model. |
| Qoder CN / Qoder CLI | Use CLI stream-json or channel exports; verify CN and international variants separately. |
| CodeArts Agent CLI | Use structured activity exports and record region and execution host. |
| MiniMax Code CLI | Use the standalone CLI trace directory; using a MiniMax model does not imply MiniMax CLI. |
| GitHub Copilot CLI | Use standalone Copilot session events. |
| Cursor CLI | Use Cursor agent stream-json, not an editor launcher. |
| DeepSeek Harness | Use normalized Harness JSONL or structured tool traces. |
| Others | Use generic Harness/OpenAI/Anthropic tool messages and text; unknown JSON is counted, not declared compatible. |

## Timeline, details, and queries

- **All / Assertions / Files / Network** filter actual records; search combines with the selected filter.
- **Source** separates agent reports, extracted intentions, and system evidence. This listening flow has no ETW connection; verdicts remain unobserved.
- Click a row or press Enter/Space for JSON details; Escape dismisses it. Export saves the current JSON view.
- **Graph** displays intentions without invented evidence edges. **Query** takes a full path and requests the daemon’s five-part `why` chain.
- Stopping releases the reader but keeps rows. Exiting the daemon loses its in-memory view, so export early; JSON is not a persisted `.cllens` session.

## Others: generic input

Create `trace.jsonl` in the selected folder, with one object per line:

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

This validates trace ingestion, not actual file or network I/O. JSONL waits for complete lines; plain text and Markdown are displayed as text only.

## Troubleshooting and limits

Choose an accessible folder for directory errors. Export logs or explicitly include profile logs when nothing is found; `state.db` is not parsed as text. Check health details for unknown schemas, rotation, truncation, and skipped bytes. Cloud, WSL, container, and remote gateway activity is visible only through exported reports in this flow.

Scanning is bounded to depth 6, 8192 entries, 128 files, 256 KiB per file, a 4096-event queue, and a 5000-row view. Limits and skipped bytes are visible in health reporting.

[Detailed vendor trace preparation](runtime-integration.md) · [Changelog](../release/CHANGELOG.md)
