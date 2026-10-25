# 推理侧适配与断言提取契约（蓝图第 8 章）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

本契约适用于 v0.1.0 的 observe-only 推理侧。输入是 append-only JSONL；适配器只负责读取与标准化，规则提取器只产生 `Assertion`，任何适配器都不得调用干预、决策或 Jev 后端。

## 8.1 适配器边界

`HarnessAdapter` 保持蓝图签名，并提供 `name()`、`mode()` 和 `health()` 诊断扩展：

```rust
pub trait HarnessAdapter: Send {
    fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError>;
    fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError>;
    fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError>;
}
```

`JsonlAdapter` 在 Windows 上打开父目录并使用 `ReadDirectoryChangesW` 作为唤醒机制，文件句柄负责从上次 offset 追加读取。未完成的尾行保留到下一批；检测到文件长度小于读指针时清空尾行、回绕到 0，并递增 `TailHealth.truncate_count`。无法创建目录通知句柄时才回退为轮询，`directory_watch_active=false` 必须在健康度中可见。

当 `AttachSpec.source` 是 `\\.\\pipe\\...` Windows 命名管道时，适配器以阻塞流方式读取，不执行 seek 或目录通知，也不接受非零恢复 offset；进程 stdout 由 `ProcessTail` 提供同样的逐行边界。这样文件、命名管道和子进程管道都经过同一 `HarnessEvent` 解析契约。

`ingest_batch` 按物理行号和字节偏移调用适配器解析。单行解析失败进入有界 `QuarantineQueue`，后续行继续处理；队列溢出计数通过 `dropped()` 暴露。未知 `phase` 映射为 `Phase::Unknown`，未知或形状不完整的 `tool` 不丢弃，完整原始记录保存在 `HarnessEvent.raw_json`。

`SchemaSentinel` 对字段名和 JSON 类型形状计算指纹，并记录 `schema_v`。指纹变化不阻断会话，但递增 `TailHealth.schema_change_count`，供 doctor 和会话健康页展示。运行时版本探针缺失时维持 Tier3 fallback，不宣称字段级兼容。

## 8.2 规则层

规则层优先处理结构化 tool call，置信度固定为 `1.0`：

| tool | 断言 |
|---|---|
| `read(path)` | `Read + ReadOnly` |
| `write(path)` | `Write + ReadWrite` |
| `edit(path)` | `Read` 前置断言 + `Write`，后者带 `order_key` |
| `create(path)` | `Create + CreateNew` |
| `delete(path)` | `Delete + ReadWrite` |
| `connect`/`bind`/`listen` | 对应网络断言 |
| `shell`/`exec` | 使用内置模板 `rm/mv/cp/curl/ssh`；未命中则交语义层 |

断言 ID 使用 `(harness_log_id, line_no, slot_no)` 稳定生成。一个 tool 产生多条断言时 slot 固定且可重放。路径先规范化为跨平台字符串；Shell 模板只解析元数据，不读取或记录文件内容。

## 8.3 语义层边界

`extractor-llm` 只表示可插拔边界，不代表仓库内置模型。Qwen 0.5B–1.5B ONNX int4 后端接入前，调用返回 `SemanticExtractorUnavailable`，不得伪造结果。真正后端必须满足：显式启用、离线无遥测、单步目标 `<300ms`、断言置信度不超过 `0.7`、`raw_quote` 逐字包含在输入文本中，并在 grammar/post-check 失败后最多重试一次。

`validate_semantic_assertion` 是后端和池之间的契约检查；不通过的结果不得入池。

## 8.4 断言池

池按 `(span, op, target, io_mode)` 建索引。同期待相同的重复项只保留一个对象并增加 `duplicate_count`；同一索引下不同 `Expectation` 保留两条并将两条 ID 标记为 contradiction。默认最多保存 100,000 条独立断言；达到上限后 `semantic_enabled=false`，规则层仍可报告重复或已存在对象，新的语义候选被拒绝。

## 8.5 可测指标

| 指标 | 测量方式 |
|---|---|
| truncate 回绕 | fixture 将文件缩短后检查 `truncate_count` 与 offset=0 |
| quarantine | 混入坏 JSONL，检查队列条数、原文和健康计数 |
| schema 哨兵 | 改变字段类型或增加字段，检查 `changed=true` |
| 规则延迟 | criterion 对结构化 tool call 批量提取，记录 p50/p95 |
| 池降档 | 以 100,001 个唯一断言插入，检查语义候选拒绝且规则候选仍可见 |
| 版本兼容 | 每个 runtime fixture 记录 `reported_version`、采样日期和原始 JSONL；无 fixture 只标 `tier3-jsonl-unverified` |

