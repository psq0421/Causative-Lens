# T0205：daemon ring reader 与兼容桥

状态：计划

目标：让 daemon 默认从 shared-memory ring 读取二进制帧，JSON 文件桥只能通过显式 fallback 使用。

范围：映射打开、ACL、SPSC reader、CRC/seq/gap、spool 续读、capture health、JSON fallback 状态、daemon 崩溃恢复。

DoD：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --workspace --target x86_64-pc-windows-msvc --offline
```

架构测试必须证明 ring 是默认 transport；fallback 必须返回 `transport=json-fallback` 和原因，不得静默降级。
