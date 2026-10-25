# G5 阶段闸门报告

## 已完成

- daemon `ObservationSession` 常驻内存接线：Harness → AssertionPool、SysEvent → CaptureHealth/SpoolFrame、对齐 → AlignmentMatch。
- RPC 方法：`session.open`、`session.status`、`session.close`、`session.ingest_harness`、`session.ingest_system`。
- Windows Named Pipe `\\.\pipe\causativelens-rpc` server，单行 JSON，跨客户端保持会话状态。
- 真实 JSONL 文件 tail：追加读取、partial line、truncate rewind。
- `ProcessTail`：启动任意 CLI 并读取 stdout JSONL。
- `SchemaSentinel`：按 RuntimeId 记录字段指纹，报告 schema 变化但不强制阻断。
- 19 项运行时继续使用 Tier3 fallback；发布时透明报告版本/采样日期/兼容级别。
- 第 12 章 IPC 安全契约：ring header/95% spool fallback、capture 控制白名单/4 KiB 上限、2 秒心跳三次失联 `CL-2001`、daemon owner-only Named Pipe ACL 和 64 KiB 有界请求。

## 测试矩阵

| 检查 | 结果 |
|---|---|
| cargo fmt --all -- --check | 通过 |
| cargo clippy --workspace --all-targets --offline -- -D warnings | 通过 |
| cargo test --workspace --offline | 通过（全部测试） |
| 文件 tail partial line/truncate | 通过 |
| schema sentinel 变化检测 | 通过 |
| daemon session pipeline 单测 | 通过 |
| Windows Named Pipe hello/open/ingest_harness/ingest_system/status | 本机手工通过 |
| IPC 控制协议、ring header、ACL/长度门禁单测 | 通过 |
| x64 | 已验证 |
| ARM64 | 按用户要求暂缓 |

## 仍需后续接线

- `clens-capture` 真实 ETW 采集仍受 UAC 限制；daemon 不绕过权限，也不直接把标准用户进程变成管理员。
- `ProcessTail` 已提供通用进程出口；每个运行时的真实启动参数、日志路径和版本探针仍需真实安装样本逐项登记。
- 当前 spool 在会话内存中编码；正式 `.cllens` writer 和 SQLite 批量提交仍属于后续存储落盘卡。
- `CreateFileMappingW(SEC_COMMIT)`、命名事件、跨进程 Release/Acquire 进度和 capture 崩溃自动收尾仍未接入运行时；详见 `docs/plan/ipc-security-audit-12.md`。

## 结论

G5 三项请求已完成可运行的基础接线和 x64 验证；真实 ETW 权限、各厂商运行时实例和持久化 writer 的环境性验证保持透明记录，没有伪报完成。

## 架构复检补充

蓝图第 3 章复检后补齐了 `clens-daemon-client` 共享 transport、Unix socket server、系统事件 `seq` 去重、`SpoolWriter` 文件恢复、capture 生命周期入口、64ms 重排缓冲和持续校时计算。补充审计见 [`docs/plan/architecture-audit-3.md`](../architecture-audit-3.md)。mmap ring、真实 ETW/TDH、`.cllens` SQLite writer、`run/attach` 生命周期仍未通过 Windows 运行态验收，继续保持为后续阻塞项。

