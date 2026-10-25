# Changelog

## 0.1.0 Beta (2026-10-25)

This is a test release. Every version before `v1.0.0` is published as beta software.

- Delivered the observe-only G0–G7 release train: ETW capture, Harness intake, causal alignment, CLI, daemon, runtime adapters, and Tauri GUI.
- Added a GUI/CLI one-piece NSIS per-user installer with user PATH, HKCU registration, `.cllens` association, shortcuts, WebView2 guidance, upgrade/downgrade switches, and `.trash` uninstall fallback.
- Added x64 installer E2E for silent install, version/doctor, GUI cold start, five residual checks, and reinstall.
- Added SPDX SBOM generation and an Azure Trusted Signing release gate; without credentials local output remains `-dev`.
- Added English (US), Simplified Chinese, and Traditional Chinese UI/installer copy and documentation navigation.
- Deferred intervention (`advise`/`enforce`) and the Jev decision backend to v0.2.0.
- Published the tested x64 `CausativeLens-setup-x64-dev.exe` preview package with its SHA-256 record, SPDX SBOM, signature status, and installer E2E report.
- ARM64 is intentionally deferred from this beta; stable `v1.0.0` requires signed x64 and ARM64 installers plus the full clean-VM matrix.

The first release is a telescope, not a hand on the tiller: it records and explains, while leaving the observed process untouched.
