# v0.1.0 implementation plan and task cards

This directory turns `CL-BLUEPRINT-001` chapters M0–M7 into one G0–G7 release train. v0.1.0 is observe-only; intervention and Jev decision work is deferred to v0.2.0.

| Gate | Focus | Exit evidence | Cards |
|---|---|---|---|
| G0 | workspace, CI, contracts, empty installer | checks green; empty package installs/uninstalls | T0001–T0005 |
| G1 | ETW capture and spool | providers, decoders, rundown, recovery, x64 package | T0010–T0015 |
| G2 | Harness and adapters | JSONL tail, assertion rules, runtime fixtures | T0020–T0023 |
| G3 | alignment and graph | score/watermark/SQLite contract and 12 fixtures | T0030–T0032 |
| G4 | CLI and query | observe command subset, JSON, replay, terminal output | T0040–T0041 |
| G5 | daemon and runtimes | named pipe, tickets, tail/sentinel/doctor | T0020–T0023 |
| G6 | Tauri GUI | four workspaces, RPC-only bridge, performance red lines | T0050–T0052 |
| G7 | packaging and release | signing, SBOM, E2E, uninstall, release artifacts | T0060–T0072 |

- [Chinese plan index](README.md) · [简体中文 mirror](README_CN.md) · [繁體中文 mirror](README_TW.md)
- [Implementation plan](implementation-plan.md)
- [Installer specification](installer-spec.md)
- [GUI UX specification](gui-ux-spec.md)
- [Test plan](test-plan.md)
- [Task cards](task-cards/)
- [Gate reports](gate-reports/)

Every card keeps a copyable DoD command and a completion record. A gate report is the logbook entry; no gate advances on optimism alone.
