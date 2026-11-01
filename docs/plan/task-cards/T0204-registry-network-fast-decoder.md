# T0204：Registry/Network 手工布局与 TDH oracle

状态：计划

目标：完成 Registry 与 Network 第一批布局，建立跨 Windows manifest 版本的 TDH oracle 差分目录。

范围：Open、Query、Set、Delete；Connect、Disconnect、Send、Receive、Listen、Bind；输出注册表键、值、协议、端点、方向和长度。

DoD：

```powershell
cargo fmt --all -- --check
cargo clippy -p clens-capture --all-targets --offline -- -D warnings
cargo test -p clens-capture --offline
cargo test --workspace --offline
```

必须报告 `manual_fast_path_events`、`tdh_fallback_events`、`unknown_layout_events` 和 oracle mismatch 数量；未知版本只能 fallback。
