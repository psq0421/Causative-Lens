# T0084 Harness 适配与断言提取复检

- 闸门：G2（并作为 G5 真实运行时接线前置）
- 范围：蓝图 8.1–8.2.3；JSONL tail、目录通知、未知 schema、quarantine、规则模板、语义层边界、断言池。
- 契约：`docs/spec/harness-8.md`
- 审计：`docs/plan/harness-audit-8.md`
- 状态：契约与隔离实现完成；daemon 已有 health 快照字段，真实运行时 fixture、ONNX 后端和常驻 tail 接线待 G5 门禁。

## 完成内容

1. `HarnessAdapter` 保持阻塞式拉取签名；`JsonlAdapter` 支持 truncate 回绕、partial line、Windows `ReadDirectoryChangesW` 唤醒和命名管道流式读取。
2. 增加 `QuarantineQueue`、`TailHealth`、`ingest_batch`，保留坏行原文和字节位置。
3. `HarnessEvent.raw_json` 保留未知 phase/tool；`SchemaSentinel` 检测字段类型形状变化。
4. 规则层覆盖基础工具和 `rm/mv/cp/curl/ssh`，`edit`/`cp` 产生先读后写 order edge。
5. 断言池支持去重计数、矛盾双保留、100,000 上限和语义降档。
6. 语义提取边界显式返回 unavailable，校验 raw quote 包含和置信度不超过 0.7。

## DoD 命令

```powershell
$cargo = "C:\Users\彭思齐\.cargo\bin\cargo.exe"
& $cargo fmt --all -- --check
& $cargo test -p clens-harnessd
& $cargo clippy -p clens-harnessd --all-targets -- -D warnings
```

## 完成记录

- 2026-10-25：16 个 harnessd 测试通过；clippy 通过；未宣称真实运行时版本兼容或 Qwen ONNX 已实现。
- 2026-10-25：v0.1.0 observe-only 边界未扩大，未添加任何干预/决策写路径。
- 2026-10-25：workspace `test --offline`、workspace clippy `-D warnings` 与 fmt check 全绿。


Language navigation: [English task-card catalog](README.en-US.md) · [简体中文任务卡](README.md) · [繁體中文任務卡](README_TW.md)。

