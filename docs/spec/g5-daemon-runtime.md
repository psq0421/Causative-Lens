# G5 常驻接线与运行时哨兵

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

## daemon 会话闭环

`clens-daemon` 的 `DaemonState` 为每个 `session.open` 创建 `ObservationSession`。会话内固定串联：

1. `session.ingest_harness` 接收 `HarnessEvent`，调用 `clens-harnessd` 规则层和 `AssertionPool`；
2. `session.ingest_system` 接收 `SysEvent`，更新 `clens-capture::CaptureHealth`，编码为 `.clspool` `SpoolFrame`，并调用 `clens-core::align_candidate`；
3. `session.status` 返回事件、去重、匹配和 spool 计数；`session.close` 释放会话；
4. `interventions`/`decisions` 没有任何 RPC 写入方法。

这个接线是 observe-only 的实时路径。系统事件按全局 `seq` 去重；`clens-storage::SpoolWriter` 可将 CRC 帧追加到 `.clspool`，后续 writer 可批量落盘到 `.cllens`，不会改变前端协议。

## Named Pipe / Unix socket

Windows daemon 使用 `\\.\pipe\causativelens-rpc`，Unix daemon 使用 `CAUSATIVELENS_RPC_SOCKET`（默认 `/tmp/causativelens-rpc`）；两者都是单行 JSON 请求/响应，state 在客户端连接之间保持。已在本机通过 .NET `NamedPipeClientStream` 验证 Windows 的 `hello`、`session.open`、`session.ingest_harness`、`session.status`。

## 真实进程/文件 tail

`JsonlAdapter` 现在打开真实 JSONL 文件，按追加读取、保留 partial line、仅输出完整行，并在文件长度小于读位置时自动 rewind，覆盖 truncate/restart。`ProcessTail::spawn` 可启动任意运行时 CLI，读取 stdout 的 JSONL 行并检查子进程状态。运行时目录通过同一 Tier3 解析器复用；未知版本不阻断，只登记 fallback。

## 上游变更哨兵

`SchemaSentinel` 对每个 `RuntimeId` 保存 JSON 顶层字段指纹；字段集合变化返回 `schema-changed`，首样本/稳定样本返回 `stable-or-first-sample`。它不阻断运行，供 doctor、日志和发布兼容性报告使用。

