# Changelog

## 0.2.5 Beta (2026-11-01)

> **This is the largest update in the project's history so far.**

- Expanded the GUI into the seven-stage Sessions → Observe → Investigate → Replay → Compare → Assist → Intervention journey.
- Added menu groups, keyboard shortcuts, breakpoint-first investigation, replay controls, comparison, evidence-grounded assistance, and persistent health indicators.
- Added architecture contracts for workspace coverage, observe-only locking, and RPC-only GUI boundaries.
- Unified all crate, CLI, GUI, Tauri, installer, payload, and lockfile versions at `0.2.5`.
- Refreshed the trilingual quick-start, monitoring guide, and release notes.
- Kept the pre-`v1.0.0` beta/test status and observe-only boundary explicit.
- Added a reproducible code-size report plus a long-form Chinese mechanism article and a plain-language sharing version.

Full notes: [English](v0.2.5.en-US.md) · [简体中文](v0.2.5.md) · [繁體中文](v0.2.5_TW.md).

## 0.2.0 Beta (2026-10-31)

- Added the Windows ETW bridge, four provider consumers, TDH decoding, path interning, FileObject tracking, and bounded loss accounting.
- Added WAL-backed SQLite fulfilment persistence, FTS5 indexes, offline `.cllens` reopen, and persisted `why` chains.
- Added Criterion baselines for alignment and spool recovery.
- Added the evidence-loop plan and explicit capture limitations.

## 0.1.2 Beta (2026-10-26)

- Added the optional evidence-grounded assistant for Jev, OpenAI-compatible, Anthropic-compatible, and Ollama endpoints.
- Added GUI and CLI settings for endpoint, model, privacy, timeout, retention, and language.
- Documented the real ETW, mmap ring, and persistence gaps instead of presenting demo traces as kernel evidence.

## 0.1.1 Beta (2026-10-25)

- Added the first listening wizard, 17 vendor templates plus Others, timeline filters, search, and GUI/CLI parity checks.
- Added x64 installer E2E evidence, uninstall residue checks, and multilingual documentation.

## 0.1.0 Beta (2026-10-25)

- Established the observe-only architecture: one daemon, two RPC frontends, and a least-privilege capture boundary.
- Added the initial CLI, GUI shell, contracts, fixtures, and release gate documentation.
