# T0207：存储明细边界与原子批量提交

状态：已完成

目标：恢复“SQLite 存索引与判定、spool 存完整明细”的架构边界，并让事务由 storage crate 强制拥有。

交付：

- 移除 `event_meta.payload_json` 写入路径；旧 `.cllens` 打开时重建表并清理 payload 派生的 FTS 行。
- 新增 `StorageBatch`/`SessionStore::write_batch`，在内部开启单一 SQLite 事务，覆盖事件元数据、断言、evidence 与 verdict。
- `event_meta` 记录 `block_id/block_off`，`event_search` 仅索引 kind/path/remote/image 等元数据。
- daemon 不再逐条调用 storage 写入；每个采集事件及其匹配证据作为一个批次提交，Harness 断言批次同样走该边界。

DoD：

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" fmt --all -- --check
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-storage --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-daemon --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" clippy -p clens-storage -p clens-daemon --all-targets --offline -- -D warnings
```

证据：`sqlite_wal_persists_graph_and_why_chain`、`legacy_payload_column_is_migrated_out_and_search_payload_is_cleared`、`write_batch_rolls_back_all_rows_when_one_operation_fails`。
