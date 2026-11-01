# T0210：GUI 用户旅程工作区与命令对等

状态：已实现（本地，等待人工验收）  
范围：v0.2.x GUI；observe-only；不新增引擎或干预通道。

## 目标

将 GUI 从 Graph/Settings 壳扩展为 Sessions → Observe → Investigate → Replay → Compare → Assist → Intervention 的完整用户旅程。菜单栏提供 Windows 惯例的文件、会话、分析、回放、视图、工具、帮助入口；断点卡成为 Why、AI 分析、时间轴和引用复制的共同跳板。

## 实现边界

- 仅复用现有 `causativelens-rpc`：`monitor.list/status/events/start/stop/templates/probe`、`query.why`、`assist.analyze`。
- GUI Rust 侧仍只有 RPC bridge；前端不链接 `clens-core`、`clens-storage`、`clens-capture` 或 `clens-harnessd`。
- 没有 RPC 的 `open`、`doctor`、`audit` 等入口显示 CLI 权威提示，不伪造结果。
- Intervention 工作区固定为 observe-only 锁定态，不注册 advise/enforce/decision 命令。
- 顶部健康度胶囊暴露 EventsLost、TDH fallback、quarantine、dropped、reorder late；没有数据时显示 0，而不是推断“未发生”。

## 变更

- `crates/clens-gui/frontend/index.html`：七工作区、菜单栏、断点详情、回放/对比/辅助占位。
- `crates/clens-gui/frontend/app.js`：工作区路由、会话卡、断点选择、回放步进、菜单动作、快捷键和健康度更新。
- `crates/clens-gui/frontend/styles.css`：Fluent 语义令牌下的菜单、卡片、双泳道和状态条样式。
- `crates/clens-gui/frontend/i18n.js`：`zh-CN`、`zh-TW`、`en-US` 文案。
- `crates/clens-arch-tests/tests/architecture_contract.rs`：七工作区、七菜单、健康度和干预锁定契约。

## DoD 命令

```powershell
node --check crates/clens-gui/frontend/app.js
node --check crates/clens-gui/frontend/i18n.js
cargo fmt --all -- --check
cargo test -p clens-arch-tests --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
```

## 已知边界

会话列表当前反映 daemon 内存中的 monitor ticket；完整 `.cllens` 历史浏览、文件选择对话框、stats/find/grep/diff 的专用 RPC、Replay 的持久化时间轴和 Adapter Doctor 仍需对应后端契约。UI 会明确提示这些边界，不能把临时卡片当作持久化事实。

