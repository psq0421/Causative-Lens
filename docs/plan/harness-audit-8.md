# 蓝图第 8 章复检记录

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

## 结论

第 8.1–8.2.3 的数据边界、追加读取、未知字段保留、坏行隔离、规则断言和断言池契约已补齐。DeepSeek 及用户点名的 18 个运行时仍统一落在 JSONL Tier3 fallback；没有真实近三个月版本 fixture 的运行时不宣称字段级兼容。

## 已补齐

1. `JsonlAdapter` 保留 partial line，检测 truncate 并回绕；Windows 文件 tail 使用 `ReadDirectoryChangesW` 唤醒追加读取，命名管道走阻塞流，回退状态写入 `TailHealth`。
2. 增加 `QuarantineEntry`、有界 `QuarantineQueue`、`ingest_batch` 和 `TailHealth`。坏行不阻断后续事件。
3. `HarnessEvent.raw_json` 保留未知 phase/tool 的原始 JSON；`SchemaSentinel` 从字段名指纹升级为字段名+JSON 类型形状指纹，并统计 schema 变化。
4. 规则层新增 Shell 模板 `rm/mv/cp/curl/ssh`；`edit` 与 `cp` 产生先读后写的显式 order edge。
5. `AssertionPool` 按 span/op/target/io_mode 去重，记录重复计数，保留矛盾断言并标记两侧；达到 100,000 条时关闭语义层。
6. 增加语义层 SPI 和 raw quote/置信度校验；没有已审计 ONNX 后端时显式返回 unavailable。

## 当前仍是门禁项

- 需要真实 Harness 日志的字段级 fixture，才能把任一运行时从 Tier3 fallback 升为 verified。
- daemon 已增加 `TailHealth` 快照和 session status 计数位；仍需把真实 tail/quarantine/sentinel 管线接入常驻运行时 doctor。
- 需要真实运行 Qwen ONNX 后端并用 criterion 证明单步 `<300ms`；在此之前禁止打开 `extractor-llm` 的生产默认值。
- `ReadDirectoryChangesW` 的异步 overlapped 版本、目录重命名竞态和日志轮换行为仍需 Windows runner E2E 验证。

## 验证记录

```powershell
cargo fmt --all -- --check
cargo test -p clens-harnessd
cargo clippy -p clens-harnessd --all-targets -- -D warnings
```

本轮结果：16 个 harnessd 单元测试通过；相关 crate clippy `-D warnings` 通过；随后执行 `cargo test --workspace --offline`、`cargo clippy --workspace --all-targets --offline -- -D warnings` 和 `cargo fmt --all -- --check` 均通过（含 GUI 与架构契约测试）。这只证明当前代码与契约门禁通过，不等同于真实运行时版本兼容或 Qwen ONNX 性能达标。

