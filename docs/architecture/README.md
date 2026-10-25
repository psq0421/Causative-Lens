# Architecture overview

> **The daemon is the heartbeat; the two frontends are its windows.**

CausativeLens uses a two-process observation topology:

```text
ETW providers ──> clens-capture (UAC, minimal privilege)
                      │ mmap ring + .clspool + control pipe
                      v
                 clens-daemon (standard user)
                    ├─ Harness adapters / assertion extraction
                    ├─ alignment / watermark / attribution
                    ├─ SQLite .cllens + query service
                    └─ causativelens-rpc
                       ├─ clens.exe       (CLI)
                       └─ clens-gui.exe   (Tauri 2 GUI)
```

The central promise is **one daemon, two frontends**. GUI and CLI cannot link the engine crates directly. They ask the same daemon for the same observe-only evidence. Capture is the only elevated component; no service, driver, injection, or intervention channel exists in v0.1.0.

## Data and time

`SysEvent`, `Assertion`, and `Verdict` are plain, versioned data in `clens-types`. Capture assigns a monotonic sequence number. ETW QPC, UTC wall time, and Harness UTC are anchored together; a watermark closes windows only after the reorder and assertion-lag budgets are safe.

The hot path uses a mmap ring with a `.clspool` extension. SQLite stores indexes, assertions, verdicts, and the thin evidence graph; high-rate event detail remains in compressed spool blocks. A crash resumes from the last acknowledged position and deduplicates by `seq`.

## Privilege and safety

The low-privilege process sends an allowlisted control protocol. Capture does not accept policy semantics, read user input, connect to the network, or execute commands. Decoders reject malformed records rather than panic. JSONL and model output are fuzzed and quarantined on failure.

Read the [architecture audit](../plan/architecture-audit-3.md), [IPC contract](../spec/ipc-security-12.md), and [data model](../spec/data-model-6.md) for normative details. Language: [English](README.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md).
