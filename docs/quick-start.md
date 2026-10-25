# Quick start

> **A session is a letter from a running program to your future self. CausativeLens keeps the envelope, the timestamp, and the fingerprints.**

This guide covers the complete v0.1.0 observe-only workflow on Windows.

## 1. Install

Download the architecture-matched installer and double-click it. The installer:

1. lets you choose English, Simplified Chinese, or Traditional Chinese;
2. installs GUI, CLI, daemon, and capture for the current user;
3. adds a de-duplicated user PATH entry;
4. registers `.cllens`, HKCU App Paths, and Start Menu/Desktop shortcuts;
5. checks WebView2 and opens the official bootstrapper page when needed;
6. offers **Restart now** or **Later**, because Windows gives PATH and shell registration a new life after restart.

No administrator prompt is expected during installation. Only capture requests UAC when a live ETW session needs it.

## 2. Verify the installation

```powershell
clens --version
clens doctor
clens doctor --json
```

A healthy installation reports `mode: observe` and `installation: installed`. JSON is intentionally English and stable for scripts.

Choose a language for human output:

```powershell
clens --lang en-US doctor
clens --lang zh-CN doctor
clens --lang zh-TW doctor
$env:CAUSATIVELENS_LANG = 'en-US'
```

## 3. Start a live observation

Embedded mode launches the agent as a child process and observes its full lifetime:

```powershell
clens run -- your-agent.exe --config config.yaml
```

Use attach mode for an existing process tree:

```powershell
clens attach --pid 12345
```

The v0.1.0 boundary is read-only. CausativeLens records metadata and traces; it does not inject, modify, enforce, or make decisions for the agent.

## 4. Watch the session

```powershell
clens watch
clens timeline
clens timeline --json
clens export --json
```

The GUI can be opened from the desktop shortcut or by launching `clens-gui.exe`. Its four workspaces mirror the CLI:

- Sessions — open/import a `.cllens` session and inspect health.
- Timeline — follow assertions, ETW events, and the watermark.
- Graph — inspect evidence edges and causal breakpoints.
- Replay / Query — ask a restricted `why`, overlay two observed runs, or export JSON.

The GUI never links the engine crates directly; it asks the same daemon that serves the CLI. That is how two windows can disagree about typography while agreeing about evidence.

## 5. Ask why

The query router is deterministic and offline-friendly. It recognizes English, Simplified Chinese, and Traditional Chinese templates:

```powershell
clens why --path config.yaml
clens why "why did it modify config.yaml?"
clens why "为什么它修改了 config.yaml？"
clens why "為什麼它修改了 config.yaml？"
clens why "when did step 12 connect api.example.com:443"
clens why --path config.yaml --json
```

The response has five stable sections: trigger assertion, evidence events, verdict, reverse attribution, and confidence/health. Unsupported natural language falls back to `clens why --path <p>` instead of inventing a query.

## 6. Open and replay sessions

```powershell
clens open path\to\session.cllens
clens replay --baseline baseline.json --branch branch.json
clens replay --baseline baseline.json --branch branch.json --json
```

v0.1.0 replay is a read-only overlay. It compares observed runs and highlights differences; counterfactual simulation remains a v2 idea.

## 7. Harness logs and adapters

Harness input is append-only JSONL. The built-in adapter accepts DeepSeek-style events and a generic JSONL Tier3 fallback. Runtime descriptors cover the planned Agent ecosystem without hard-blocking unknown versions. Use the daemon doctor and adapter fixtures to see whether a runtime has a verified schema sample or is using fallback parsing.

A log rotation or truncation is reported as health data rather than silently skipped. Unknown fields are retained; malformed lines enter quarantine.

## 8. Troubleshooting

| Symptom | Action |
|---|---|
| `clens` is not found after installation | Restart the shell or choose **Restart now** in the installer. Check the user PATH. |
| GUI says offline | Start `clens-daemon.exe`, then retry; offline mode still opens local/read-only surfaces. |
| `CL-1001` | Run as a standard user for GUI/CLI; only the capture step should request UAC. Follow the repair text. |
| `CL-2001` | Capture missed three heartbeats. Preserve the session spool and attach the health report. |
| WebView2 missing | Run the official Evergreen bootstrapper, then relaunch the GUI. |
| A locked file survives uninstall | Reboot; the uninstaller moves it to `.trash` and retries cleanup. |

## 9. Build from source

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.1.0
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.0
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

Formal Release also requires Azure Trusted Signing, SBOM, deterministic-build evidence, and a clean VM matrix. Local credentials are never replaced with a self-signed certificate.

## 10. Uninstall cleanly

Use **Apps > Installed apps > CausativeLens > Uninstall**, or:

```powershell
& "$env:LOCALAPPDATA\causativelens\uninstall.exe" /S /NORESTART /CLEANUSERDATA
```

The E2E gate checks that files, registry keys, user PATH, `.cllens` association, and shortcuts are all gone. Session data is deleted only after the explicit checkbox or `/CLEANUSERDATA`.

For deeper details, continue to the [documentation index](README.md), [architecture](architecture/README.md), [contracts](reference/contracts.md), and [release gate](release/g7-release-gate.md).
