# CausativeLens v0.2.5 installer and release line

The NSIS installer is per-user and installs `clens.exe` (CLI), `clens-gui.exe` (GUI), `clens-daemon.exe`, and `clens-capture.exe` in one payload. It does not write HKLM, install a service, load a driver, or inject into another process. Only `clens-capture.exe` carries the `requireAdministrator` manifest for real ETW capture.

## Local x64 build

```powershell
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

Without Azure Trusted Signing this produces `dist/CausativeLens-setup-x64-dev.exe`. A production-named asset requires verified Trusted Signing metadata:

```powershell
$env:AZURE_TRUSTED_SIGNING_DLIB = 'C:\path\Azure.CodeSigning.Dlib.dll'
$env:AZURE_TRUSTED_SIGNING_METADATA = 'C:\path\metadata.json'
powershell -ExecutionPolicy Bypass -File installer/build-release.ps1 -Arch x64 -Version 0.2.5
```

Self-signed certificates must not be used to imitate a production signature. SBOM generation uses the locked Cargo metadata and emits SPDX 2.3.

## Installer behavior

- `/S` silent install; `/D=<absolute path>` custom destination; `/NORESTART` suppresses automatic restart; `/LOG=<path>` writes an install log.
- Writes user PATH idempotently, HKCU Uninstall/App Paths, `.cllens` Classes, Start Menu entries, desktop shortcuts, and the GUI default entry.
- Detects WebView2 and guides the user to the official Evergreen bootstrapper when missing.
- The completion page explains why a restart may be needed for PATH, registry, and WebView2 registration.
- Locked files are moved to `.trash/pending` during uninstall. User data is removed only when selected or when `/CLEANUSERDATA` is passed.

## E2E

```powershell
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 `
  -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

The E2E script exercises silent install into a path containing spaces and Chinese characters, CLI/GUI version checks, doctor, PATH and registry checks, GUI cold-start smoke, five residue checks, uninstall, and reinstall. ARM64 uses the same scripts on an ARM64 runner.

All versions before `v1.0.0` are beta/test releases. See the [multilingual documentation index](../docs/README.md).
