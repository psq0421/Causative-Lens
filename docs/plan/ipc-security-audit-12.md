# 第 12 章 IPC 与安全复检

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

## 已补齐

- ring header 与 256 MiB/95% fallback 策略契约。
- capture control 消息白名单、4 KiB 长度上限、拒绝计数。
- 2 秒心跳与三次失联 `CL-2001` 判定。
- daemon Named Pipe owner-only ACL、64 KiB 有界 JSON 行、方法/参数大小检查。
- capture 健康度增加 ring→spool、控制拒绝、capture failure 计数。

## 仍需真实 Windows 验收

- `CreateFileMappingW` + `SEC_COMMIT` 实际共享内存与命名事件通知。
- Release/Acquire 读写进度、跨进程 CRC 恢复及 capture 崩溃自动收尾。
- 当前用户 SID ACL 的 clean VM 验证、UAC 拒绝路径和 CL-2001 端到端日志。
- ETW decoder/TDH oracle fuzz、依赖锁定后的 SBOM 与确定性供应链证明。

## DoD 命令

```powershell
C:\Users\彭思齐\.cargo\bin\cargo.exe fmt --all -- --check
C:\Users\彭思齐\.cargo\bin\cargo.exe test -p clens-types -p clens-capture -p clens-daemon --offline
C:\Users\彭思齐\.cargo\bin\cargo.exe clippy -p clens-types -p clens-capture -p clens-daemon --all-targets --offline -- -D warnings
C:\Users\彭思齐\.cargo\bin\cargo.exe build -p clens-daemon --offline
```

