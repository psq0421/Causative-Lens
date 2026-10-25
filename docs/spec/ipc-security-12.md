# 第 12 章进程间通信与安全契约（v0.1.0）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

## 协议边界

- 数据面目标是单写者/单读者 256 MiB mmap ring；ring 使用 `RingHeader`（magic、schema、writer/reader pos、dropped、heartbeat），达到 95% 水位后切 `.clspool`，不阻塞 ETW 消费线程。
- 通知面目标是命名事件/信号量，不允许轮询。
- 控制面使用字节流 JSON 行协议；消息长度上限为 4096 bytes（capture control），RPC 请求上限为 64 KiB。capture 只接受 `start/stop/heartbeat/metrics` 枚举，未知或策略语义消息计数并拒绝。
- 心跳每 2 秒检查，连续 3 个周期未收到时输出 `CL-2001`，会话必须保留已有 spool 数据并停止。

## 已落地的可测试边界

`clens-types` 提供 `CaptureControlMessage`、长度限制、枚举反序列化和 `CaptureHeartbeat`；`clens-capture` 提供 ring header 编解码、magic/schema/位置校验、95% spool fallback、控制门和健康度计数；daemon Named Pipe 使用 owner-only SDDL（`D:P(A;;GA;;;OW)`）、有界行读取和 RPC 参数长度检查。

## 权限口径

capture 不实现策略、注入、联网，也没有低权限控制服务。owner-only 管道 ACL 由创建 daemon 的当前用户 owner SID 生效；不授予 `Users`/`Everyone`。capture 控制门只允许生命周期/心跳/指标消息。

## 尚未宣称完成

本轮建立协议与安全契约，但尚未把 `CreateFileMappingW(SEC_COMMIT)`、命名事件对象、跨进程 Release/Acquire 读者进度和 capture↔daemon 实际控制通道接入运行时。真实 ring 崩溃恢复、ACL Windows runner、ETW/TDH fuzz 和 SBOM/`cargo cyclonedx` 仍是 G1/G7 门禁。

