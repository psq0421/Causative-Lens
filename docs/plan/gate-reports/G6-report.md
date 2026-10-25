# G6 阶段闸门报告

## 已完成

- 新增 `crates/clens-gui` Tauri 2 壳，正式桌面构建使用 `--features tauri-shell`。
- Rust 侧仅通过 `clens-types` 与共享 `clens-daemon-client` 提供 `bootstrap` 与 `rpc_call`，不链接 `clens-core`、`clens-storage`、`clens-capture` 或 `clens-harnessd`。
- Windows 使用 `\\.\pipe\causativelens-rpc`，非 Windows 预留 `CAUSATIVELENS_RPC_SOCKET` Unix socket，便于跨平台迁移。
- 四工作区：Sessions、Timeline、Graph、Replay / Query。
- Fluent Design 语义 token：Segoe UI Variable、4/8px grid、Neutral/Accent 角色、Layer/Card、focus ring、浅深色主题和键盘可达性。
- Timeline 窗口化渲染最多 500 行；Graph Canvas 使用 requestAnimationFrame 和 100ms 增量合批。
- GUI 兼容性文案保留 Tier3 fallback，不强制拒绝未知 Agent 版本。

## 验证

| 检查 | 结果 |
|---|---|
| `node --check crates/clens-gui/frontend/app.js` | 通过 |
| GUI ↔ engine/storage 架构静态契约 | 已加入 arch-test |
| Fluent token/500 DOM/100ms 合批静态契约 | 已加入 arch-test |
| `cargo fmt --all -- --check` | 通过（已执行） |
| Tauri 2 `cargo check -p clens-gui --features tauri-shell` | 未完成：本机 Cargo 在 crates.io 索引/依赖解析阶段超时 |
| 冷启动/RSS/10 万事件帧率/graph.delta | 未测，必须在 Tauri 编译成功后执行 |
| ARM64 | 按用户要求暂缓 |

## 风险与下一步

Tauri 2 工程文件完整，但当前机器尚未缓存 `tauri-build` 依赖链（离线首个缺口为 `anyhow`，在线索引更新超过 2 分钟无输出）。因此 G6 不能标记最终放行；需要恢复 Cargo registry 后执行正式构建、WebView2 冷启动、Canvas 帧率和 RSS 实测。已有 Rust workspace 的 G0–G5 代码在加入 GUI 前均已通过全量测试。

## 结论

G6 前端和 Tauri 壳已启动并完成静态契约；候选未放行，性能指标与正式 Tauri 编译证据待依赖解析恢复后补齐。
