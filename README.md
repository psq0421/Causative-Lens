# CausativeLens

> **A small lantern for the moments when software stops being obvious.**

CausativeLens v0.1.1 Beta is a read-only agent trace observer for Windows. Choose your agent, pick a folder, review detection, and start listening. One `clens-daemon` owns the data; `clens.exe` and `clens-gui.exe` communicate through `causativelens-rpc`.

> **Current Beta limitation (2026-10-25):** the GUI now tails supported local traces, with 20 template choices covering the requested 17 products, both WorkBuddy channels, DeepSeek and Others. These are content-based integrations, not certification of every vendor version. The complete ETW/persistence pipeline is still unconnected; agent reports are labelled separately from kernel evidence. `run/attach` remain offline. Start with [your first observation](docs/guide/monitoring-start.md).

## Languages

The product supports **English (US)**, **Simplified Chinese**, and **Traditional Chinese**.

- CLI: `--lang en-US|zh-CN|zh-TW`, or `CAUSATIVELENS_LANG`.
- GUI: choose a language in the title bar; the choice is remembered per user.
- Installer: English, Simplified Chinese, and Traditional Chinese pages.
- JSON/RPC: field names and enum values stay English and stable for automation.

Language switch: [简体中文](README_CN.md) · [繁體中文](README_TW.md) · [documentation index](docs/README.md).

## Start here

- [Quick start](docs/quick-start.md) · [快速入门](docs/快速入门.md) · [快速入門](docs/快速入門.md)
- [v0.1.1 listening guide](docs/guide/monitoring-start.md) · [vendor trace preparation](docs/guide/runtime-integration.md)
- [Documentation index](docs/README.md)
- [Architecture and contracts](docs/architecture/README.md)
- [Release notes](docs/release/CHANGELOG.md)

## Local build

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.1.1
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.1
```

The installer is per-user and puts GUI, CLI, daemon, and capture in one directory. Only `clens-capture.exe` requests UAC. The GUI is the default desktop entry point and `.cllens` files open in it.

## Observe-only boundary

v0.1.0 deliberately observes. Intervention and decision commands are reserved for v0.2.0; the corresponding tables remain forward-compatible but have no write path. This boundary is a promise: the lens may explain a footprint, but it never reaches into the program it is watching.

## Release safety

Every version before `v1.0.0` is beta/test software. The `v0.1.0` prerelease uses an explicitly labelled x64 `-dev` installer; it is unsigned and ARM64 is deferred. A stable release requires signed x64 and ARM64 installers, SBOM, deterministic-build evidence, and the installer E2E matrix. See [the release gate](docs/release/g7-release-gate.md).

## Contributing

Read [CONTRIBUTING](docs/guide/contributing.md) and the bilingual [documentation index](docs/README.md) before changing a contract. Keep commits small, keep JSON stable, and leave a test where a future maintainer can hear your reasoning.
