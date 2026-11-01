# T0210: GUI journey workspaces and command parity

Status: implemented locally, awaiting human review. Observe-only; no engine or intervention channel added.

The GUI now exposes the seven-step journey: Sessions, Observe, Investigate, Replay, Compare, Assist, and the locked Intervention placeholder. A Windows-style menu bar covers file, session, analysis, replay, view, tools, and help actions. Breakpoints are first-class: a selected breakpoint offers Why, evidence-grounded assist, timeline jump, and reference copy.

The frontend uses existing RPC methods only. The Rust shell remains an RPC bridge and does not link engine, storage, capture, or harness crates. Entries without a backend RPC (for example `open`, `doctor`, and `audit`) disclose the CLI-authoritative boundary rather than inventing results. Health capsules show loss, TDH fallback, quarantine, dropped, and late-reorder counters. Intervention remains visibly observe-only and locked.

## DoD

```powershell
node --check crates/clens-gui/frontend/app.js
node --check crates/clens-gui/frontend/i18n.js
cargo fmt --all -- --check
cargo test -p clens-arch-tests --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
```

The current session cards reflect daemon-memory monitor tickets. Persistent `.cllens` browsing, dedicated stats/find/grep/diff RPCs, replay persistence, and Adapter Doctor remain backend contract work and are stated in the UI.

