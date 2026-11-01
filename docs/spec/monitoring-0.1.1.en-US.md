# v0.2.5 observe monitoring contract

This contract defines the GUI listening wizard and the daemon's read-only RPC. It accepts existing traces; it does not launch an Agent, change vendor configuration, or turn a selected folder into ETW evidence.

## RPC methods

- `monitor.templates`: no parameters; returns data-driven templates with stable IDs and localized labels.
- `monitor.probe`: `{runtime_id,folder,include_profile}`; the folder must exist; returns scan roots, candidate files, detected formats, and export guidance.
- `monitor.start`: probe fields plus `read_existing` (default `true`); returns `session_id`. The reader never executes Agent commands or changes vendor settings.
- `monitor.status`: `{session_id}`; returns state, read/recognized/rejected lines, rotation, truncation, dropped rows, recent errors, and `capture_connected`. Without real ETW it must be `false`.
- `monitor.events`: `{session_id,after,limit}`; `after` is a monotonic Timeline cursor; bounded responses return `rows`, `next_cursor`, `has_more`, and `retained_from`.
- `monitor.stop`: `{session_id}`; stops the file reader and retains the in-memory Timeline; repeated stop is idempotent.
- `monitor.list`: returns listener sessions still held by the daemon so the GUI can reopen them.
- `session.close`: stops the reader and releases memory; closing never fabricates a `.cllens` file.

## Row categories

Timeline rows use `assertions`, `files`, `network`, and `other`. Sources `assertion`, `agent`, and `system` stay separate: a file tool request is client intent, not an ETW `FileRead`; `tool_result` never creates a duplicate assertion. Each row includes `seq`, `ts_utc_ms`, `category`, `source`, `kind`, `target`, `text`, and `state`.

## Input formats

Accepted sources include normalized Harness JSONL, Anthropic messages, OpenAI tool calls, Codex exec/session, OpenClaw transcripts, Hermes session exports/observer records, and generic structured tool events. Markdown/plain text is text only; `state.db` is not scanned as text. Parse failures and unknown objects remain visible in health counters. A `synthetic` fixture is not vendor-version certification.

## Security and boundary

Scanning is read-only and does not follow symlinks or read credential files. User-directory scans require an explicit UI choice. Prompts and tool traces may be sensitive, so the GUI states that processing is local. The listener does not automatically open UAC and does not claim visibility into cloud, WSL, or container internals.

Language switch: [English](monitoring-0.1.1.en-US.md) · [简体中文](monitoring-0.1.1.md) · [繁體中文](monitoring-0.1.1_TW.md).
