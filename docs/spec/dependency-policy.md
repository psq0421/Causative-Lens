# 第三方依赖与构建策略契约

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

核心路径保持 Rust。新增 crate 必须在 PR 中说明用途、维护状态、许可证、unsafe 面积和 Windows CI 结果。

允许许可证：MIT、Apache-2.0、BSD-2-Clause、BSD-3-Clause、ISC、MPL-2.0。未标注许可证、AGPL、SSPL 和其他强 copyleft 依赖拒绝进入 v0.1.0。

首选白名单：`windows`、`rusqlite`（`bundled`）、`zstd`、`crossbeam-channel`/`crossbeam-queue`、`clap`、`serde`/`serde_json`、`tracing`、`thiserror`、`bytemuck`、`dashmap`、`globset`、`ariadne`。`tokio` 只允许适配层 feature，不能进入 capture/core/storage 热路径。

每次依赖变更执行：

```powershell
cargo deny check advisories licenses bans sources
cargo audit --locked
cargo metadata --locked --format-version 1 --no-deps
```

仓库根目录的 `deny.toml` 固化许可证、来源和通配版本策略；`scripts/dependency_audit.ps1` 在工具未安装时只做明确降级提示，不能把降级结果标成发布通过。

