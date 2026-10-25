# G7 阶段闸门报告（x64 本机执行版）

## 范围

本轮完成 G7 的双前端一体安装器、签名阻断流水线、SBOM、安装器 E2E 与卸载五查。ARM64 依照当前指令暂缓，不宣称已通过。

## 产物

- `dist/CausativeLens-setup-x64-dev.exe`：NSIS Unicode、per-user、GUI/CLI/daemon/capture 一体包，约 2.06 MB，低于 25 MB 红线。
- `installer/payload/payload-manifest.json`：x64 payload、CLI/GUI 版本和每个文件 SHA-256。
- `dist/causativelens-sbom.spdx.json`：SPDX 2.3，来自锁定 Cargo 元数据。
- `dist/signature-status.json` 与 `dist/payload-signature-status.json`：本机无 Azure Trusted Signing 凭据，状态为 `unsigned-dev`，`formal_release_allowed=false`；直接执行 `build-installer.ps1 -Release -Signed` 返回非零并阻断。
- `dist/installer-e2e-report.json`：静默安装、doctor、GUI 冷启动、卸载五查、重装证据。

## E2E 矩阵（本机 x64）

| 检查 | 结果 | 证据 |
|---|---|---|
| `/S /NORESTART /LOG= /D=` | 通过 | `installer-e2e-report.json` |
| 空格/中文临时安装路径 | 通过 | 同上 |
| CLI 与 GUI 版本 | `0.1.0` | 同上 |
| `clens doctor --json` | `status=ok`, `mode=observe`, `installation=installed` | 同上 |
| 三语 CLI 人类文案 | 通过：en-US、zh-CN、zh-TW | installer-e2e-report.json 的 language_checks |
| 用户级 PATH 注入/幂等/移除 | 通过 | `uninstall-verify.path=true` |
| Uninstall/App Paths/Classes | 通过 | `uninstall-verify.registry=true` |
| `.cllens` 关联 | 通过 | `uninstall-verify.association=true` |
| 桌面/开始菜单快捷方式 | 通过 | `uninstall-verify.shortcuts=true` |
| 安装文件删除 | 通过 | `uninstall-verify.files=true` |
| GUI 冷启动 | 通过，约 133.55 ms（目标 <600 ms） | `gui-cold-start.milliseconds` |
| 卸载后 doctor | 通过：`clens` 不再可发现，报告 `not-installed (command absent)` | `uninstall-verify.doctor_after_uninstall` |
| 重装 | 通过 | `reinstall-verify.status=ok` |

## 签名与发布门

`installer/sign-release.ps1` 只接受 Azure Trusted Signing DLib/metadata，使用 RFC 3161 时间戳并执行 `signtool verify /pa /all`。缺少凭据时只允许 `-AllowDevUnsigned`，不会生成或冒充正式 Release 签名。正式文件名必须由 `build-installer.ps1 -Release -Signed` 生成为 `CausativeLens-setup-x64.exe`；ARM64 使用相同流程。

## 未放行项与风险

- ARM64 原生 runner、双架构 VM E2E 尚未执行。
- Azure Trusted Signing account/profile/DLib/CI secret 未注入，故本报告不放行 tag 或 GitHub Release。
- WebView2 缺失分支已实现引导，但本机实际环境已有 Runtime，缺失/离线 bootstrapper 场景需 VM 证据。
- 锁定文件 `.trash`、升级/降级退出码 5 和重启后清理需 CI VM 演练。









