# T0202：采集数据面与解码快路径重构

状态：计划已细化，待实现

目标：把 capture→daemon 从 JSON 文件桥迁移到 mmap ring，并为四个 ETW provider 建立可由 TDH oracle 背书的手工布局快路径。

权威契约：`docs/spec/etw-7.md`、`docs/spec/ipc-security-12.md`、`docs/plan/capture-fast-path.md`、`CL-BLUEPRINT-001` 第 7、12、13、14 章。

范围：

- SPSC shared-memory ring、固定二进制帧、CRC、seq、gap 和 crash resume；
- ring 95% spill 到 `.clspool`，ETW 消费线程不阻塞；
- `LayoutKey`/`LayoutSpec` 目录和 Process/File/Registry/Network Top-event 快路径；
- TDH metadata cache、fallback worker、oracle golden 对照；
- daemon ring reader 与显式 JSON fallback；
- capture→Verdict 基准和健康指标。

禁止：修改 workspace version；引入干预层；用 JSON 或 Harness 日志冒充内核事实；凭 opcode 猜测字段偏移；在 ETW callback 中同步执行 TDH fallback。

DoD：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --workspace --target x86_64-pc-windows-msvc --offline
cargo bench -p clens-core --bench alignment --offline
cargo bench -p clens-storage --bench spool_decode --offline
```

Windows 提权验收还必须输出：

```powershell
clens-capture.exe --live --output "$env:TEMP\causativelens-live.jsonl"
```

并保存 `manual_fast_path_events`、`tdh_fallback_events`、`decode_failures`、`EventsLost`、`RealTimeBuffersLost`、`ring_dropped`、`spool_gaps`、吞吐、P50/P99 延迟和提交 hash。JSON 输出只允许作为显式兼容模式。

完成记录：

- 实现提交：
- oracle 报告：
- benchmark 报告：
- Windows build 矩阵：
- 已知限制：
