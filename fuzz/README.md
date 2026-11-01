# CausativeLens fuzz targets

These targets are intentionally outside the workspace build. They exercise the
three untrusted byte boundaries that can corrupt a session if they panic:

- `capture_json`: fixture/compatibility decoding for the four provider shapes;
- `harness_jsonl`: append-only runtime records and unknown schema shapes;
- `spool_frame`: CRC frame parsing and crash-tail recovery.

Run from a machine with `cargo-fuzz` installed:

```powershell
cargo fuzz run capture_json -- -max_total_time=900
cargo fuzz run harness_jsonl -- -max_total_time=900
cargo fuzz run spool_frame -- -max_total_time=900
```

The normal workspace remains offline-buildable because this directory is not a
workspace member. CI's fuzz job must install `cargo-fuzz`, build these targets,
and archive corpus/crash artifacts as evidence.
