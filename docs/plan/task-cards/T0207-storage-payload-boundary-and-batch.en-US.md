# T0207: storage payload boundary and atomic batches

Status: completed

Goal: restore the contract that SQLite stores indexes and verdicts while the spool stores complete event details, and make the storage crate own transaction boundaries.

Delivered:

- Removed the `event_meta.payload_json` write path; opening a legacy `.cllens` rebuilds the table and clears payload-derived FTS rows.
- Added `StorageBatch` and `SessionStore::write_batch`, which owns one SQLite transaction for event metadata, assertions, evidence, and verdict updates.
- `event_meta` records `block_id/block_off`; `event_search` indexes metadata such as kind/path/remote/image only.
- The daemon submits each captured event and its matches as one batch; Harness assertions use the same boundary.

DoD:

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" fmt --all -- --check
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-storage --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-daemon --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" clippy -p clens-storage -p clens-daemon --all-targets --offline -- -D warnings
```

Evidence: `sqlite_wal_persists_graph_and_why_chain`, `legacy_payload_column_is_migrated_out_and_search_payload_is_cleared`, and `write_batch_rolls_back_all_rows_when_one_operation_fails`.
