# 总体架构审计：蓝图第 3 章

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

审计对象是蓝图 3.1–3.5 与当前 x64 工作树。结论按“已落实 / 本次补齐 / 仍缺实证”区分。

## 逐项结果

| 条目 | 结论 | 证据或缺口 |
|---|---|---|
| 3.1 双进程与最小特权 | 本次补齐边界 | capture manifest 已声明 `requireAdministrator` 与 `longPathAware`；新增 `clens-daemon-client`，CLI/GUI 不再把 daemon 引擎库作为依赖。capture 的真实 ETW 热路径仍需 Windows UAC 实测。 |
| 3.2 进程拓扑 | 部分落实 | daemon 会话接线、Named Pipe、Unix socket、Harness 文件/进程 tail 已有；新增 capture 生命周期入口和 append-only spool writer。mmap ring、真实 TDH 解码、SQLite writer 尚未完成，不能宣称端到端已落盘。 |
| 3.3 时钟域 | 本次补齐可测基础 | 已有最小往返锚点选择；新增 64ms QPC 重排缓冲、持续漂移计算和超阈值刷新测试。Windows `QueryPerformanceCounter`/`GetSystemTimePreciseAsFileTime` 的 16 轮采样和 Harness 哨兵仍需实机接入。 |
| 3.4 典型会话 | 未完全落实 | daemon 可以 ingest Harness/System 并增量对齐；CLI 仅对 `hello/open/status` 尝试 RPC，`run` 尚未自动拉起 capture/Agent，最终 flush 与 `.cllens` 写入仍缺。 |
| 3.5 三种部署形态 | 部分落实 | `open`/离线查询契约和 observe 错误路径存在；`attach --pid` 的 rundown/进程树范围以及 `run` 的子进程生命周期尚未接通。 |

## 本次代码补充

- `crates/clens-daemon-client`：共享 JSON-RPC transport；Windows Named Pipe 与 Unix socket 分支一致。
- `clens-daemon`：非 Windows `run_unix_socket` 常驻入口；会话按系统事件 `seq` 去重并返回 `events_deduplicated`。
- `clens-capture`：`CaptureLifecycle`、64ms `ReorderBuffer`、`ClockCalibrator`；非 Windows 明确返回 ETW unsupported。
- `clens-storage`：`SpoolWriter` 与 `recover_file`，保证追加帧、CRC 和损坏尾部可审计。
- `clens-arch-tests`：新增 CLI 依赖边界、提权 manifest、跨平台 transport、spool 恢复契约。

## 未伪造的验收项

以下项目不能用静态检查替代：真实 ETW 四 provider 会话和 TDH oracle、mmap ring 跨进程崩溃恢复、QPC/UTC 16 轮同拍、SQLite `.cllens` 批量提交、`run/attach` 进程树、Windows clean VM 安装器 E2E。它们继续作为 G1/G3/G4/G7 的阻塞出口，直到产生 Windows 运行日志和可重放 fixture。

## 检查命令

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p clens-capture -p clens-storage -p clens-daemon -p clens-daemon-client -p clens-arch-tests
```

