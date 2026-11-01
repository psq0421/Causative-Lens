# T0208: reorder, templates, audit chain, and real attach

Status: code and contract work completed in this pass; the elevated Windows runtime DoD remains required.

Goal: close the correctness gaps found in review without treating fixtures or the compatibility bridge as production evidence.

Delivered:

- `ReorderBuffer` fixes the 64 ms QPC window, sequence-set convergence, late-event health counters, and non-monotonic release counters without rewriting an original late timestamp.
- Windows capture uses `StartTraceW`/`EnableTraceEx2`/`ProcessTrace`/`ControlTraceW`; `monitor.start` accepts `attach_pid` and sends capture-state rundown for Process/File providers.
- The rule catalog covers file, process, network, registry, resource-budget, network-allowlist, data-flow, and idempotency declarations. Single-event semantic boundaries are tested; idempotency cannot be falsely fulfilled by one event.
- The session manifest stores a payload hash and recomputes the chain during verification. Legacy rows without that hash fail verification explicitly.
- Added three `cargo-fuzz` targets for capture JSON, Harness JSONL, and spool frames.

DoD:

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" fmt --all -- --check
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-capture -p clens-core -p clens-harnessd -p clens-storage -p clens-daemon --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" check -p clens-capture -p clens-daemon --target x86_64-pc-windows-msvc --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" clippy -p clens-capture -p clens-core -p clens-harnessd -p clens-storage -p clens-daemon --all-targets --offline -- -D warnings
cargo fuzz run capture_json -- -max_total_time=900
cargo fuzz run harness_jsonl -- -max_total_time=900
cargo fuzz run spool_frame -- -max_total_time=900
```

Runtime evidence must come from an elevated Win10 1809+ x64 runner: `attach_pid` captures only the target PID, rundown events have `is_rundown=true`, and the report records `EventsLost`, `RealTimeBuffersLost`, late counters, PID-filter errors, and Win32 return codes. Without that evidence the attach/rundown gate is not green.
