# 第 6 章核心数据模型契约（v0.1.0）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

本文件把 CL-BLUEPRINT-001 第 6.1–6.5 落成可序列化、可离线重放的 Rust 数据契约。跨模块对象只携带数据；平台句柄、SQLite 连接、glob 编译器和线程状态留在实现 crate。

## 版本和 observe 边界

`schema_v` 是跨模块结构的版本字段。破坏性字段变化升 major；新增可选字段使用 `serde(default)` 保持旧 JSON 可读。v0.1.0 只产生 observe 数据。`interventions` 与 `decisions` 仍建表，但没有写入路径。

## SysEvent

`clens-types::SysEvent` 包含 `schema_v`、capture 进程分配的单调 `seq`、`ts_qpc`、pid/tid、`SysKind` 和 `EventPayload`。`is_rundown` 是现有实现的兼容扩展，用于区分会话建立时的补发事件；旧输入缺少该字段时按 `false` 解码。

`ts_qpc` 的契约单位是 ETW QPC 的 100 ns 坐标。UTC 对齐必须先应用 capture 的 `ClockAnchor`，不能把原始 QPC 值直接当作 Unix 毫秒。`ClockAnchor::qpc_to_utc_100ns/qpc_to_utc_ms` 提供饱和换算；当前对齐函数仍保留毫秒参数以兼容 G3 旧 fixture，真实 daemon 接入锚点前不得宣称跨时钟精度，见 `open-questions.md` 的 OQ-017。

`clens-capture::SequenceAllocator` 在事件离开解码热路径前分配 `seq`；daemon 仍按 `seq` 去重。`PathInterner` 将重复路径压成 `PathIdx`，按 spool block snapshot 物化路径。FileRead/FileWrite 只保存长度和属性，不保存用户数据内容。

## Assertion

`Assertion` 现在使用：

- `span: HarnessSpan { step, byte_start, byte_end }`，保存 JSONL 原文位置；
- `order_key: Option<(u64, u64)>`，表达前置/后置断言关系；
- `TargetPattern::Path` 和 Endpoint host 的规范化 glob/文本。`normalize_path_pattern` 统一 Windows 分隔符；IR 不保存 `globset::GlobMatcher`，由 `clens-core` 编译，保证纯数据和 serde 稳定；
- `confidence`、`raw_quote` 作为审计证据。

断言 ID 由 `stable_assertion_id(harness_log_id, line_no, slot_no)` 的确定性 FNV-1a 实现生成。相同 JSONL 重放得到相同 ID，不再依赖进程内递增计数器。`AssertionPool` 将矛盾状态放在池的索引中，不污染跨模块 Assertion 数据。

`HarnessEvent` 增加可选来源字段 `harness_log_id`、`line_no`、`byte_start`、`byte_end`，以及可选 `raw_json`；旧适配器未提供时，规则层以 step 和当前行长度作回退。tail 适配器应在能取得文件偏移时填入真实值。`raw_json` 用于保留未知 phase/tool 或未来字段形状，缺失时按 `None` 解码并且不改变旧 JSON 的序列化形状。

## Verdict 与履约图

公共模型新增 `Fulfilment`（Fulfilled、Deviated、Unobserved、Partial）、`ScoreComponents`、`EvidenceEdge`、`Deviation`、`Watermark` 和 `Verdict`。`clens-core` 的 `VerdictState::Unrelated` 仅是候选评分的内部状态，不写入履约图；已命中的候选可以通过 `verdict_from_match` 转为公共 `Verdict`。

履约图仍是 SQLite 的断言节点、事件节点和两类边：`evidence(assertion_id,event_seq)` 与 `Assertion.order_key`。`assertions`、`verdicts`、`event_meta` 均保存 `schema_v`；`interventions`/`decisions` 只建表。

## 测试契约

`clens-types` 覆盖稳定 ID、source span、order key、Verdict edge 的 serde round-trip；`clens-capture` 覆盖路径驻留和 seq 分配；核心、daemon、storage 既有 fixture 与 observe 契约继续运行。完整 workspace 的离线测试若因本机 Tauri 缓存缺少 `redox_users`/`windows_i686_gnu` 失败，必须报告环境原因，不得删掉 GUI 成员。

