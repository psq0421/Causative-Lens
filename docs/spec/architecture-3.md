# 第 3 章总体架构契约（v0.1.0）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

本契约把蓝图 3.1–3.5 落到可检查的边界。v0.1.0 只有 `observe`，不增加干预通道。

## 进程边界

- `clens-capture` 是唯一可请求 `requireAdministrator` 的进程。它负责 ETW 会话计划、事件解码、rundown 标记和顺序化输出。
- `clens-daemon` 是标准用户常驻主进程，负责 Harness tail、断言池、对齐、存储和查询 RPC。
- `clens-cli` 与 `clens-gui` 都是 daemon JSON-RPC 客户端。它们不能依赖 `clens-core`、`clens-storage` 或 `clens-capture`；共享 transport 位于 `clens-daemon-client`。
- daemon 与 GUI/CLI 的 Windows 传输是 `\\.\pipe\causativelens-rpc`；Unix 预留使用 `CAUSATIVELENS_RPC_SOCKET`。

## 采集通道

采集事件以全局 `seq` 标识。内存 ring 是热路径，`.clspool` 是崩溃恢复和落盘路径。`.clspool` 帧必须包含 magic、schema、flags、长度和 CRC；恢复扫描不得跳过损坏偏移。daemon 对重复 `seq` 做去重并暴露 `events_deduplicated`。

## 时序契约

- QPC、UTC 和 Harness UTC 通过 16 轮同拍样本选择最小往返间隔锚点。
- 采集输出使用 64ms QPC 重排尾窗；会话结束必须 flush 尾窗。
- 持续校准漂移阈值是 `200µs/s`，触发刷新时写入校准事件。
- 对齐水位线必须单调不减；跨 provider 主序是 `(ts_qpc, seq)`。

## 线程预算契约

- capture：四个 provider 消费线程、一个 spool writer、一个控制线程。
- daemon：一个 spool reader、一个 Harness tail、`clamp(physical_cpus / 2, 1, 4)` 个断言 worker、一个对齐线程、一个 SQLite writer。
- 线程预算由 `capture_thread_plan()` 和 `daemon_thread_plan()` 暴露给架构测试；计划不等于运行态完成，未接线线程不得写成已启动。

## 部署形态

`run`、`attach`、`open` 的语义由 CLI/daemon 契约承载：`run` 覆盖子进程生命周期，`attach` 从附加时刻开始，`open` 只读离线。若本机没有 daemon 或 ETW 权限，客户端必须返回可执行的诊断，不得伪造实时采集已启动。

## 验收命令

```powershell
cargo test -p clens-capture -p clens-storage -p clens-daemon -p clens-daemon-client -p clens-arch-tests
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

