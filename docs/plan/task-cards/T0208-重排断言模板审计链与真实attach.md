# T0208：重排、断言模板、审计链与真实 attach

状态：已完成本轮代码与契约补齐；真实提升权限 Windows runner 仍需执行运行态 DoD。

目标：关闭检察指出的四类正确性缺口，不把 fixture 或兼容桥误写成生产证据。

交付：

- `ReorderBuffer` 固化 64ms QPC 窗口、序号集合收敛、迟到事件健康度和非单调释放计数；不改写迟到事件原始时间戳。
- Windows capture 使用 `StartTraceW`/`EnableTraceEx2`/`ProcessTrace`/`ControlTraceW`；`monitor.start` 支持 `attach_pid`，Process/File provider 发送 capture-state rundown。
- 规则模板覆盖文件、进程、网络、注册表、资源预算、网络白名单、数据流溯源和幂等声明；声明型断言的单事件语义边界有测试，幂等不会被单事件错误判定为已履约。
- session manifest 保存 payload hash，并在验证时重算链；无 payload hash 的旧链明确验证失败。
- 新增 `fuzz/` 的 capture JSON、Harness JSONL、spool frame 三个 cargo-fuzz target。

DoD：

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" fmt --all -- --check
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p clens-capture -p clens-core -p clens-harnessd -p clens-storage -p clens-daemon --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" check -p clens-capture -p clens-daemon --target x86_64-pc-windows-msvc --offline
& "$env:USERPROFILE\.cargo\bin\cargo.exe" clippy -p clens-capture -p clens-core -p clens-harnessd -p clens-storage -p clens-daemon --all-targets --offline -- -D warnings
cargo fuzz run capture_json -- -max_total_time=900
cargo fuzz run harness_jsonl -- -max_total_time=900
cargo fuzz run spool_frame -- -max_total_time=900
```

运行态证据：提升权限 Win10 1809+ x64 runner；`attach_pid` 仅捕获目标 PID，rundown 事件 `is_rundown=true`；记录 `EventsLost`、`RealTimeBuffersLost`、迟到计数、PID 过滤错误和 Win32 返回码。没有该证据时不得宣称 attach/rundown 门禁已绿。
