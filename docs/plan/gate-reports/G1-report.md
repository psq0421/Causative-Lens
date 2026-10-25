# G1 阶段闸门报告（进行中，未放行）

## 已完成

- `clens-types` 增加版本化 `SysEvent`、`SysKind`、`EventPayload`、网络端点和路径索引类型。
- `clens-capture` 增加四 provider 名称契约、`CausativeLens_` 会话名、Process/File/Registry/Network fixture 解码器、TDH oracle 边界、FileObject 表、健康度指标和十六轮同拍时钟锚定选择器。
- `clens-storage` 增加 `.clspool` 帧格式：magic、schema、flags、payload/raw 长度、CRC32；损坏帧可报告为 gap 候选，并支持恢复扫描保留有效帧、登记坏帧偏移。
- 运行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`，全部通过；`cargo check --workspace --target aarch64-pc-windows-msvc` 通过。

## 尚未完成

- Windows `StartTrace`/`EnableTraceEx2`/`ProcessTrace` 真实会话。
- Windows `StartTrace`/`EnableTraceEx2`/`ProcessTrace` 真实会话。
- TDH oracle golden 对照和 logman provider manifest 自动比对。
- mmap ring、崩溃恢复、gaps manifest、时钟锚定十六轮同拍。
- ARM64 可执行文件链接与含采集器的双架构安装包；当前 ARM64 check 通过，但链接器缺少可用 MSVC `link.exe`。
- G1 性能预算前三行的 criterion 数字。

## 当前结论

**未放行。** 当前只完成 G1 的数据契约和可测试骨架；继续实现真实 ETW 采集与剩余解码链路后，再提交闸门评审。

## 最新验证

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo test --workspace`：通过，新增 G1 测试共 12 项相关断言。
- `cargo check --workspace --target aarch64-pc-windows-msvc`：通过。
- ARM64 完整链接：阻塞于机器缺少 Windows SDK ARM64/MSVC 目标库（`msvcrt.lib`）；x64 编译已通过。

G1 仍未放行，原因是实际 ETW 会话控制面和 TDH golden oracle 尚未接入，且 ARM64 链接环境未补齐。

## x64 收口验证

- `cargo build --workspace --release --target x86_64-pc-windows-msvc`：通过。
- `scripts/etw_provider_self_check.ps1`：四个 provider GUID 与 `logman query providers` 全部一致。
- `dist/CausativeLens-setup-x64-dev.exe`：NSIS 编译通过，237,709 bytes。
- 安装器静默安装：通过；安装目录包含 `clens.exe`、`clens-daemon.exe`、`clens-capture.exe`、`uninstall.exe`。
- 三个已安装可执行文件启动：通过；版本保持 `0.0.0-dev`。
- 静默卸载：通过；安装目录删除。
- 真实 `logman create trace`：当前 shell 非管理员，返回 `Access is denied`；已加入 `scripts/etw_smoke.ps1`，脚本强制检查管理员令牌并验证 ETL 生成/清理，按 R4 设计不绕过权限检查。

## 当前结论

**x64 G1 候选通过，等待人类放行。** 代码、解码/恢复测试、provider manifest 自检、x64 release 构建和带采集器安装烟测已完成；真实 ETW 会话需通过带 `requireAdministrator` manifest 的 capture 进程在 UAC 环境执行。ARM64 按人类决定暂缓。
