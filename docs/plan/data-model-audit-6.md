# 核心数据模型第 6 章审计记录

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

日期：2026-10-25
范围：CL-BLUEPRINT-001 第 6.1–6.5；只处理 v0.1.0 observe 数据面。

## 逐项结论

| 蓝图要求 | 当前结论 | 落点 |
|---|---|---|
| SysEvent 三元字段、schema_v、seq | 已补齐并加兼容 rundown 标记 | `crates/clens-types/src/lib.rs`、`clens-capture::SequenceAllocator` |
| QPC/UTC/Harness 三时钟 | 已补 `ClockAnchor` 的饱和 QPC→UTC 换算；daemon 对齐接线仍需在 G3 关闭 | `clens-capture::ClockAnchor`、OQ-017 |
| PathIdx 驻留 | 已补 `PathInterner`，可 snapshot 给 spool block | `crates/clens-capture/src/lib.rs` |
| Assertion HarnessSpan | 已补 `HarnessSpan` 与 `HarnessEvent` 来源偏移 | `crates/clens-types/src/lib.rs` |
| order_key | 已补到 Assertion，默认 `None` 保持旧日志可读 | `crates/clens-types/src/lib.rs` |
| 稳定 assertion_id | 已由三元组确定性 FNV-1a 生成；池内矛盾状态单独维护 | `stable_assertion_id`、`AssertionPool` |
| Fulfilment/Verdict | 已下沉到 clens-types；候选 Unrelated 仍是 core 内部状态 | `verdict_from_match` |
| SQLite 履约图 | DDL 已为 assertions/verdicts/event_meta 增加 schema_v；干预表仍只建表 | `clens-storage::GRAPH_SCHEMA_V1` |
| session ID UTC+随机后缀 | RPC ticket 仍用 UUID，尚未切换 `.cllens` 文件主键 | OQ-018，避免破坏 RPC 兼容 |

## 验证记录

```powershell
cargo test -p clens-types -p clens-capture -p clens-core -p clens-harnessd -p clens-daemon -p clens-storage --offline
cargo fmt --all -- --check
```

本机 Cargo 离线解析受 GUI Tauri 缓存缺少 `redox_users`/`windows_i686_gnu` 影响时，测试命令以临时排除 GUI 成员的方式验证核心 crate，测试后恢复 workspace 清单；这不是删除 GUI 依赖的实现变更。

## 未静默改变的差异

- `SysEvent.is_rundown` 是现有实现字段，保留并用 serde 默认兼容旧 spool；
- `TargetPattern` 保存规范化 glob 字符串，不把不可序列化的 `GlobMatcher` 放进 clens-types；
- RPC `SessionTicket.session_id` 保留 UUID，文件主键迁移需要同时升级 CLI/daemon/存储协议；
- 对齐层的时间单位转换尚未声称完成，必须在 G3 锚点接线后再关闭 OQ-017。

