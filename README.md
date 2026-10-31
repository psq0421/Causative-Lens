# CausativeLens

> **A small lantern for the moments when software stops being obvious.**

CausativeLens v0.2.0 Beta is a read-only agent trace observer for Windows. Choose your agent, pick a folder, review detection, and start listening. One `clens-daemon` owns the data; `clens.exe` and `clens-gui.exe` communicate through `causativelens-rpc`.

> **Evidence boundary:** v0.2.0 adds the real `OpenTraceW`/`ProcessTrace` capture bridge and WAL-backed fulfilment graph. An elevated capture host is required for kernel facts; without it the UI reports evidence as unavailable. Start with the [v0.2.0 plan](docs/plan/v0.2.0.en-US.md).

## Languages

The product supports **English (US)**, **Simplified Chinese**, and **Traditional Chinese**.

- CLI: `--lang en-US|zh-CN|zh-TW`, or `CAUSATIVELENS_LANG`.
- GUI: choose a language in the title bar; the choice is remembered per user.
- Installer: English, Simplified Chinese, and Traditional Chinese pages.
- JSON/RPC: field names and enum values stay English and stable for automation.

Language switch: [简体中文](README_CN.md) · [繁體中文](README_TW.md) · [documentation index](docs/README.md).

## Start here

- [Quick start](docs/quick-start.md) · [快速入门](docs/快速入门.md) · [快速入門](docs/快速入門.md)
- [v0.1.2 listening guide](docs/guide/monitoring-start.md) · [vendor trace preparation](docs/guide/runtime-integration.md)
- [Documentation index](docs/README.md)
- [Architecture and contracts](docs/architecture/README.md)
- [Release notes](docs/release/CHANGELOG.md)

## Local build

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.0
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.0
```

The installer is per-user and puts GUI, CLI, daemon, and capture in one directory. Only `clens-capture.exe` requests UAC. The GUI is the default desktop entry point and `.cllens` files open in it.

## Observe-only boundary

v0.2.0 deliberately observes. Intervention and decision commands remain unregistered; the corresponding tables are forward-compatible but have no write path. This boundary is a promise: the lens may explain a footprint, but it never reaches into the program it is watching.

## Release safety

Every version before `v1.0.0` is beta/test software. The `v0.2.0` GitHub release is a normal non-prerelease test release; its x64 local package is unsigned until Azure Trusted Signing credentials are supplied. ARM64 is deferred. A stable release requires signed x64 and ARM64 installers, SBOM, deterministic-build evidence, and the installer E2E matrix. See the [v0.2.0 release notes](docs/release/v0.2.0.en-US.md).

## Contributing

Read [CONTRIBUTING](docs/guide/contributing.md) and the bilingual [documentation index](docs/README.md) before changing a contract. Keep commits small, keep JSON stable, and leave a test where a future maintainer can hear your reasoning.
