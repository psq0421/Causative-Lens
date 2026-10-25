# Changelog

## 0.1.2 Beta (2026-10-26)

- Added the optional `clens-assist` layer for evidence-grounded Jev, OpenAI-compatible, Anthropic-compatible, and local Ollama analysis.
- Added the GUI Settings workspace with provider, endpoint, model, privacy, timeout, timeline, language, and theme controls.
- Added `clens assist` and `clens why --assist`; API keys stay in process memory and never enter `settings.json`.
- Enforced citation grounding: findings must reference assertion IDs or event sequences present in the submitted causal chain.
- Hardened endpoint and header validation, repeated Windows settings saves, relative-path redaction, and human-readable CLI output.
- Kept the observe-only boundary: Jev decision, intervention, ETW production closure, and ARM64 runtime remain outside this beta's verified scope.
- This GitHub release is intentionally published as a normal (non-prerelease) release at the maintainer's request, while the product remains test software before `v1.0.0`; the uploaded x64 installer is explicitly unsigned because Azure Trusted Signing was unavailable on the build host.
- [Full v0.1.2 notes](v0.1.2.md).
## 0.1.1 Beta (2026-10-25)

- Rebuilt the Fluent listening wizard and added 20 content-based runtime template choices.
- Fixed real Timeline filters, search/details/follow/export, and persistent CLI shortcuts.
- Added bounded native trace readers and daemon monitoring RPC; reports remain distinct from system evidence.
- Added browser E2E, monitoring coverage measurement, Tauri build and safer installer E2E.
- [Full bilingual notes and remaining release gates](v0.1.1.md).

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
