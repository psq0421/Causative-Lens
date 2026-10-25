# 查询与回放契约（第 11 章复检）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

## `why` 受限 NL 路由

v0.1.0 查询入口只接受确定性模板，不调用 LLM：`why/为何/为什么`、`when/何时/什么时候`、`what-if/如果/假如`。实体规则抽取规范化路径、`host:port` 端点和 `step N`。未同时识别意图与实体时固定降级为 `use clens why --path <p>`，不会自由生成 SQL 或自然语言答案。

结构化结果固定包含五段：`trigger_assertion`（raw quote、step）、`evidence_events`（seq、时间、pid、操作）、`verdict`（履约/断点）、`reverse_attribution`、`confidence_health`。CLI 与 GUI 都通过 daemon 的 `query.why` 使用相同形状；GUI 不链接引擎 crate。

## 回放叠加

`replay_overlay` 使用事件公共前缀表示分叉前事件复用，分叉后按稳定 `assertion_id` 与 fingerprint 对齐并输出 baseline/branch diff。`counterfactual_simulation=false` 是 v0.1.0 兼容性标记；第 18 章反事实模拟留给 v0.2.0。

CLI 的离线验证入口是 `clens replay --baseline <events.json> --branch <events.json> --json`；输入为 `ReplayEvent[]`，缺少任一输入或解析失败会返回 observe-only 错误对象，不会模拟事件。

## 终端渲染

CLI 提供双泳道 timeline renderer，偏差行在 ANSI 可用时红色高亮；`NO_COLOR`、重定向或老 conhost 走纯文本。`--json` 始终输出机读结构，不包含终端控制序列。当前 renderer 是稳定渲染边界；完整 ratatui 交互循环仍需依赖冻结后在 G4 发布门接入，不能把边界测试当成交互 TUI 已完成。

## 证据与限制

当前 daemon 查询对内存中的 observe session 执行索引扫描，SQLite 持久化查询与真实 `.cllens` 打开仍属于 G4/G7 接线项。没有会话记录时五段结果为空并诚实报告健康度；不会伪造证据。

