# Contract index

CausativeLens treats these files as executable contracts:

| Area | Primary contract | What it fixes |
|---|---|---|
| Identity | `clens-types` and `docs/spec/rpc-observe.md` | product slug, version, observe mode, pipe name |
| Capture | `docs/spec/etw-7.md` | four ETW sessions, decoders, rundown, spool health |
| Harness | `docs/spec/harness-8.md` | JSONL tail, quarantine, assertion extraction |
| Alignment | `docs/spec/alignment-9.md` | score dimensions, windows, watermarks, attribution |
| Query | `docs/spec/query-11.md` | deterministic why/when/what-if routing and five-part chain |
| IPC | `docs/spec/ipc-security-12.md` | ring, control allowlist, ACL, heartbeat failure |
| Runtime | `docs/spec/runtime-adapters.md` | descriptor, fixture, sentinel, doctor compatibility |
| Packaging | `docs/plan/installer-spec.md` | per-user installation, signing, E2E, clean uninstall |
| GUI | `docs/plan/gui-ux-spec.md` | four workspaces, RPC-only bridge, performance budget |

The blueprint remains the historical source (`CL-BLUEPRINT-001`). When a new requirement conflicts with it, record the conflict in `docs/plan/open-questions.md` and update the contract test before editing implementation. A contract is a lighthouse: it does not steer the ship, but it makes the rocks visible.

Language: [English](contracts.md) · [简体中文](contracts_CN.md) · [繁體中文](contracts_TW.md).
