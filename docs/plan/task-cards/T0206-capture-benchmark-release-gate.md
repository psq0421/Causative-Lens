# T0206：采集端到端基准与发布门

状态：计划

目标：用实际 x64 Windows 数据证明或否定采集层吞吐预算，不再只列指标名称。

DoD：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo bench -p clens-core --bench alignment --offline
cargo bench -p clens-storage --bench spool_decode --offline
```

Windows 提权测试还必须保存 manual/TDH 吞吐、ring 写读吞吐、端到端 P50/P99、CPU/RSS、`EventsLost`、`RealTimeBuffersLost`、ring dropped、spill、gap 和 commit hash。报告未完成时不得把 200k events/s 标为达标。
