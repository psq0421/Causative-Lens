# CausativeLens

> **A small lantern for the moments when software stops being obvious.**

**CausativeLens v0.2.5 is the largest update in this project's history so far.** It is a read-only Windows observer for agent traces: it compiles reasoning steps into causal assertions, then checks those assertions against available system evidence. One `clens-daemon` owns the evidence; `clens.exe` and `clens-gui.exe` are two RPC frontends.

> **Evidence boundary:** v0.2.5 keeps the observe-only contract. The GUI now follows the complete journey from Sessions to Observe, Investigate, Replay, Compare and Assist. Kernel evidence still requires an elevated capture host; when it is unavailable, the product says so instead of treating a harness log as an ETW fact.

## Languages

The product supports **English (US)**, **Simplified Chinese**, and **Traditional Chinese**.

- CLI: `--lang en-US|zh-CN|zh-TW`, or `CAUSATIVELENS_LANG`.
- GUI: choose a language in the title bar; the choice is remembered per user.
- Installer: English, Simplified Chinese, and Traditional Chinese pages.
- JSON/RPC: field names and enum values stay English and stable.

Language switch: [简体中文](README_CN.md) · [繁體中文](README_TW.md) · [documentation index](docs/README.md).

## Start here

- [Quick start](docs/quick-start.md) · [简体中文快速入门](docs/快速入门.md) · [繁體中文快速入門](docs/快速入門.md)
- [First observation](docs/guide/monitoring-start.md) · [runtime integration](docs/guide/runtime-integration.md)
- [Documentation index](docs/README.md)
- [Architecture and contracts](docs/architecture/README.md)
- [v0.2.5 release notes](docs/release/v0.2.5.en-US.md)

## Local build

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

The per-user installer contains GUI, CLI, daemon and capture. Only `clens-capture.exe` requests UAC. The GUI is the default desktop entry point and `.cllens` files open in it.

## Observe-only boundary

CausativeLens observes and explains. Intervention and decision commands remain unregistered; their schema remains forward-compatible, but there is no write path. The lens may explain a footprint, never reach into the process it watches.

## Release safety

Every version before `v1.0.0` is beta/test software. v0.2.5 is published as a normal GitHub test release. This local build is x64-first; if Azure Trusted Signing is unavailable, the uploaded installer is explicitly marked `unsigned-dev`. Stable `v1.0.0` still requires signed x64 and ARM64 installers, SBOM, deterministic-build evidence, and the clean-VM installer matrix. See the [v0.2.5 release notes](docs/release/v0.2.5.en-US.md).

## Contributing

Read [CONTRIBUTING](docs/guide/contributing.md) and the [bilingual documentation index](docs/README.md) before changing a contract. Keep commits small, keep JSON stable, and leave a test where a future maintainer can hear your reasoning.