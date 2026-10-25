# G7 release gate

Every version before `v1.0.0` is a beta/test release. A beta may be published as a GitHub prerelease with an explicitly labelled development package and an honest limitation report. The requirements below are the stable-release gate for `v1.0.0` and later.

A formal release is allowed only when the following evidence is present:

- x64 and ARM64 installers named `CausativeLens-setup-x64.exe` and `CausativeLens-setup-arm64.exe`;
- Azure Trusted Signing verification and timestamp for every executable;
- SPDX SBOM, locked dependencies, reproducible-build metadata, and hashes;
- clean Windows VM E2E: silent install, restart, `clens doctor`, minimal session, GUI cold-start/RSS/frame-rate, uninstall five checks, and reinstall;
- documentation links resolve in English, Simplified Chinese, and Traditional Chinese.

A machine without signing credentials may build and test `-dev` packages, but cannot create a formal Release. The gate is deliberately boring: software earns trust one observable check at a time.

Language: [English](g7-release-gate.md) · [简体中文](g7-release-gate_CN.md) · [繁體中文](g7-release-gate_TW.md).
