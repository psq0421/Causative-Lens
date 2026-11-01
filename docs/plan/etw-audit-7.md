# 第 7 章 ETW 采集审计记录

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

日期：2026-10-25
范围：CL-BLUEPRINT-001 第 7.1–7.11；当前只收 x64，ARM64 暂缓；v0.1.0 observe-only。

## 逐项审计

| 条款 | 状态 | 结论/证据 |
|---|---|---|
| 7.1 四 provider 独立会话 | 部分完成 | 四会话名、生命周期接口和残留清理已在 `clens-capture`；真实 StartTrace/ProcessTrace 未接入 |
| 7.2 启动与权限 | 部分完成 | manifest 已 requireAdministrator；runas、ACL 降权、CL-1001 尚未完成 |
| 7.3 keyword | 部分完成 | ProviderContract/Full-Lean-Minimal 白名单模型已补；数值 mask 仍为兼容全 mask，需 manifest runner 校准 |
| 7.4 手工解码/TDH | 未完成 | 当前是 JSON fixture 解码器和 TdhOracle trait 边界，无 EVENT_RECORD/TDH golden |
| 7.5 FileObject | 部分完成 | insert/rename/close、PathInterner 已测试；28 分片、TTL/LRU、真实 FileObject 字段未接线 |
| 7.6 重排/序列化 | 部分完成 | 64ms QPC 重排、seq 分配、窗口外迟到健康度和 spool writer 已有；mmap ring 跨进程未完成 |
| 7.7 rundown | 部分完成 | Windows `EnableTraceEx2(EVENT_CONTROL_CODE_CAPTURE_STATE)` 与 attach PID filter 已接线；真实提升权限 runner 仍需证明补发事件字段 |
| 7.8 自噪声/过滤 | 部分完成 | attach 路径已通过 `EVENT_FILTER_TYPE_PID` 接入；递归子孙 PID 和 excluded manifest 仍需 runner 证据 |
| 7.9 丢失/背压 | 部分完成 | CaptureHealth 已增加 ETW 丢失/drop 计数；QueryTrace 轮询和 80%/95% 策略未实现 |
| 7.10 GUID/布局自检 | 部分完成 | GUID JSON、Rust 常量表、PowerShell 自检已存在；TDH golden 对照未实现；capture/harness/spool fuzz target 已建立 |
| 7.11 Windows 兼容 | 文档完成 | x64 Win10 1809+ 是当前目标；ARM64 暂不承诺运行态 |

## 本轮补充

- `ProviderContract` 与 `keyword_policy` 将 provider、GUID、keyword 名称集中为可审计契约；
- `cleanup_stale_sessions` 在 Windows 启动前停止/删除残留会话；
- `CaptureLifecycle` 增加显式 rundown 请求状态；
- `CaptureFilter` 记录 include-only/excluded PID；
- `CaptureHealth` 增加 `real_time_buffers_lost`、`buffer_loss_events`、`dropped_events`；
- `ReorderBuffer` 增加窗口外迟到、最大迟到 QPC 和非单调释放计数，健康度通过 daemon/RPC 可见；
- Windows 原生会话控制、attach PID filter 和 Process/File capture-state rundown 已接线；
- 新增 provider、过滤、丢失健康度契约测试。

## 未静默改变

- 未把 JSON fixture 解码器宣称为真实 ETW 解码器；
- 未凭记忆填写 keyword 数值 mask；
- 未伪造 TDH、UAC、StartTrace 或 QueryTrace 运行态通过；
- 未承诺 ARM64 运行态；
- 未改变 v0.1.0 observe-only 范围或 workspace 版本。

## 验证

```powershell
cargo fmt --all -- --check
cargo test -p clens-capture --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
```

本机完整 workspace 离线解析仍可能受 Tauri 缓存缺少 `redox_users`/`windows_i686_gnu` 影响；非 GUI workspace 和架构契约可正常验证。真实 ETW DoD 必须在提升权限的 Windows runner 完成。

本机执行 `scripts/etw_provider_self_check.ps1` 已通过四个 provider GUID；当前令牌不是管理员，`scripts/etw_smoke.ps1` 未执行，避免绕过 UAC 门禁。

