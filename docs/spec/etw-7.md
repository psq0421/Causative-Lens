# 第 7 章 Windows ETW 采集契约（v0.1.0）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

本契约覆盖 CL-BLUEPRINT-001 第 7.1–7.11。它区分“已可由 Rust fixture 测试证明”的采集边界和“必须在提升权限的真实 Windows runner 上证明”的 ETW 边界。没有真实 runner 证据时，不得把 JSON fixture 解码器称为 ETW 已接通。

## 会话矩阵

采集使用四条独立实时会话，名称固定为 `CausativeLens_Process`、`CausativeLens_File`、`CausativeLens_Registry`、`CausativeLens_Network`。provider GUID 只允许来自 `docs/spec/etw-guids.json` 和 `logman query providers` 自检；代码中的 `PROVIDER_CONTRACTS` 与四个逻辑名一一对应。

`CaptureLifecycle::start_all` 在 Windows 运行时必须先停止并删除同名残留会话；任一会话创建失败必须停止已创建会话并返回错误。关闭顺序是先停止消费线程、冲刷 spool，再停止 ETW 会话。当前代码已实现残留清理和部分失败回滚；真实 `StartTraceW/EnableTraceEx2/ProcessTrace` 仍是 G1 阻塞项。

## 权限与安全

`clens-capture.exe` manifest 使用 `requireAdministrator` 且 `uiAccess=false`。启动链路必须由标准用户主进程通过 `ShellExecuteW(..., "runas", ...)` 拉起；capture 控制面保留管理员令牌，数据写入目录只授予必要的 Users 写权限，控制通道只读。UAC 拒绝必须映射为 CL-1001，并给出可执行的手动提升命令。当前 manifest 已存在，runas、ACL 降权和 CL-1001 运行态路径尚未接线。

## Keyword 策略

`ProviderContract` 保存 provider 名称、GUID 和 manifest keyword 名称；`keyword_policy` 提供 Full/Lean/Minimal 三档，Lean/Minimal 只减少 FileRead 等高频低价值事件。数值 mask 必须由目标 Windows manifest 查询结果确认，不能凭记忆填写。当前 `EtwSessionSpec` 仍使用历史全 mask 兼容值，因此在真实 ETW 接通前不得宣称 keyword 成本红线已通过。

## 解码与 oracle

热路径目标是按 provider/opcode 使用无分配 cursor 解码 packed payload；无法确认布局时回落 TDH。每个手工解码器都必须有同一 `EVENT_RECORD` fixture 的 TDH golden 输出，字段、长度、UTF-16、FileObject 和 rundown 标记逐项比较。当前实现的 `decode_*_record` 接收 JSON fixture，`TdhOracle` 只有 trait 边界，尚未接入 `EVENT_RECORD`、TDH API 或 golden 对照。

## FileObject、路径和时序

FileCreate 建立 FileObject→PathIdx 映射，Rename 更新，Close 删除；FileRead/FileWrite 不携带文件内容。`PathInterner` 已提供路径驻留和 block snapshot。并发 28 分片、TTL/LRU 262144 上限和周期清扫仍需真实 capture 线程接线。

每 provider 消费线程使用 QPC 64ms 重排窗口；按 `(ts_qpc, seq)` 输出，关闭时必须 flush 尾窗。`SequenceAllocator` 保证 capture 侧 seq 单调。当前重排和 seq 已有纯 Rust 测试，mmap ring 和跨进程单写者接线仍未完成。

rundown 必须在 Process/File 会话建立后请求，并将补发事件标记 `is_rundown=true`；对齐层只能用它做状态补全和路径归属，不能把它当观测窗口内的时序证据。当前 `request_rundown` 只记录控制状态，尚未调用 provider rundown keyword。

## 过滤、丢失和背压

PID 过滤必须显式记录 excluded_pids；过滤事件不能伪装成“未发生”。`CaptureFilter` 已提供 include-only 和排除集合模型，EnableTraceEx2 的真实 PASS_FILTER 尚未接线。

健康度必须每 500 ms 读取 EventsLost/RealTimeBuffersLost；非零时写入可见 BufferLoss 事实并降低判定置信度。`CaptureHealth` 已增加实时 buffer 丢失、丢失事件次数和显式 drop 计数，但 ETW QueryTrace 轮询、ring 水位 80%/95% 策略和低价值 FileRead 丢弃尚未接线。任何窗口发生丢失时，Unobserved 只能输出“不可见”，不能解释为“未发生”。

## 自检与兼容矩阵

CI 需要两道门：真实 Windows runner 执行 GUID 自检；TDH 与手工解码对同一 fixture 逐字段比较。Win10 1809+ x64 是 v0.1.0 当前验证矩阵；ARM64 仅保留构建/契约矩阵，按当前指示暂不做运行态承诺。

