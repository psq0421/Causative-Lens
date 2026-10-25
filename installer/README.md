# CausativeLens v0.1.0 G7 安装与发布线

安装器页面支持 English、简体中文和繁體中文；同一签名 payload 只按架构区分。完整双语/三语文档入口见 [docs/README.md](../docs/README.md)。

本目录是 NSIS 一体安装器的唯一入口。x64 包同时安装 `clens.exe`（CLI）、`clens-gui.exe`（默认 GUI）、`clens-daemon.exe` 与 `clens-capture.exe`；安装器本身为当前用户权限运行，不写 HKLM、不装服务。只有 `clens-capture.exe` 携带 `requireAdministrator` manifest。

## 本机 x64 构建

```powershell
# 构建 release 二进制、校验 CLI/GUI 同版本并生成 payload-manifest.json
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.1.0

# 编译 NSIS；无 Trusted Signing 时只生成 -dev 包
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.0
```

输出为 `dist/CausativeLens-setup-x64-dev.exe`。正式 Release 文件名只能是 `CausativeLens-setup-x64.exe`，必须在签名验证通过后构建：

```powershell
$env:AZURE_TRUSTED_SIGNING_DLIB = 'C:\path\Azure.CodeSigning.Dlib.dll'
$env:AZURE_TRUSTED_SIGNING_METADATA = 'C:\path\metadata.json'
powershell -ExecutionPolicy Bypass -File installer/sign-release.ps1 -Files installer/payload/clens.exe,installer/payload/clens-gui.exe,installer/payload/clens-daemon.exe,installer/payload/clens-capture.exe -OutputStatus dist/payload-signature-status.json
# 只有 payload-signature-status.json 为 Trusted Signing 时才能生成正式文件
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.0 -Release -Signed
powershell -ExecutionPolicy Bypass -File installer/sign-release.ps1 -Files dist/CausativeLens-setup-x64.exe
```

也可使用 `installer/build-release.ps1 -Arch x64 -Version 0.1.0` 一次执行 payload 签名、正式包编译、安装器签名和 E2E；任一步没有 Trusted Signing 都会阻断。

`sign-release.ps1` 使用 Azure Trusted Signing 的 `signtool /dlib /dmdf` 流程并运行 `signtool verify /pa /all`。缺少配置时只生成 `signature-status.json` 的 `unsigned-dev` 状态，正式构建会阻断；禁止用自签名证书代替公开发布签名。Azure 的租户、客户端和 secret 由 Trusted Signing DLib 的受支持凭据链提供，不写入仓库。

`generate-sbom.ps1` 使用锁定的 `cargo metadata` 生成 SPDX 2.3 文件。包体上限由 `build-installer.ps1` 阻断（`<25 MB`）。

## 安装器行为

- `/S` 静默；`/D=<绝对路径>` 自定义目录（最后一个参数）；`/NORESTART` 禁止自动重启；`/LOG=<path>` 记录步骤；卸载支持 `/CLEANUSERDATA`。
- 安装写入用户级 PATH（按分号 token 幂等去重）、HKCU Uninstall/App Paths、`.cllens` Classes，建立桌面和开始菜单 GUI/CLI 快捷方式。
- WebView2 Evergreen 已安装则继续；缺失时优先执行随包 bootstrapper，否则在交互安装打开微软官方引导页，静默安装只记录可修复状态。
- 失败或锁定文件不会覆盖其他用户文件；卸载会尝试将锁定文件移到 `$INSTDIR/.trash/pending`。会话数据只在用户勾选或传 `/CLEANUSERDATA` 时清理。
- 完成页解释 PATH、注册表和 WebView2 生效需要重启，并提供立即重启/稍后选项。

## x64 安装器 E2E

```powershell
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 `
  -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

脚本会在包含空格和中文的临时目录执行静默安装、CLI/GUI 版本检查、`clens doctor --json`、PATH/注册表/关联/快捷方式检查、GUI 冷启动（目标 `<600 ms`）、卸载五项残留检查与重装，并把证据写到 `dist/installer-e2e-report.json`。ARM64 使用同一脚本和 NSIS 脚本，在 ARM64 runner 上传入 `-Arch arm64`。




