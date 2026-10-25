# 第 11 章查询与回放复检

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

## 已补齐

- `clens-cli::query`：确定性 why/when/what-if 路由，路径/端点/step 实体抽取，未知输入固定提示。
- `clens-daemon::query.why`：observe-only 查询 RPC，返回五段式因果链和采集健康度。
- `clens-cli::replay`：稳定 assertion ID 的只读 baseline/branch 叠加；显式关闭 v2 反事实模拟。
- `clens-cli::terminal`：双泳道输出、偏差 ANSI 高亮、老 conhost/NO_COLOR 纯文本降级。
- GUI 前端修复乱码，并在单 daemon 会话上调用 `query.why`。

## 未宣称完成

- `ratatui` 依赖和完整交互事件循环尚未冻结，当前只提供可测渲染边界。
- SQLite/`.cllens` 的持久化 query index 尚未由 CLI 打开路径接线；内存 daemon 查询用于 RPC 契约测试。
- 回放仅为观察叠加，不执行反事实模拟。

## DoD 证据

```powershell
C:\Users\彭思齐\.cargo\bin\cargo.exe fmt --all -- --check
C:\Users\彭思齐\.cargo\bin\cargo.exe test -p clens-cli -p clens-daemon --offline
node --check crates/clens-gui/frontend/app.js
```

