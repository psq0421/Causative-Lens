# CausativeLens v0.2.5 安装器与发布线

NSIS 安装器按当前用户安装，一次安装 `clens.exe`（CLI）、`clens-gui.exe`（GUI）、`clens-daemon.exe` 与 `clens-capture.exe`。它不写 HKLM、不安装服务、不加载驱动，也不向其他进程注入。只有 `clens-capture.exe` 带有 `requireAdministrator` manifest，用于真实 ETW 采集。

## 本机 x64 构建

```powershell
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

没有 Azure Trusted Signing 时，输出为 `dist/CausativeLens-setup-x64-dev.exe`。正式命名的资产必须先通过 Trusted Signing 校验：

```powershell
$env:AZURE_TRUSTED_SIGNING_DLIB = 'C:\path\Azure.CodeSigning.Dlib.dll'
$env:AZURE_TRUSTED_SIGNING_METADATA = 'C:\path\metadata.json'
powershell -ExecutionPolicy Bypass -File installer/build-release.ps1 -Arch x64 -Version 0.2.5
```

禁止用自签名证书冒充生产签名。SBOM 使用锁定的 Cargo 元数据生成 SPDX 2.3 文件。

## 安装器行为

- `/S` 静默安装；`/D=<绝对路径>` 自定义目录；`/NORESTART` 禁止自动重启；`/LOG=<path>` 写安装日志。
- 幂等写入用户级 PATH、HKCU Uninstall/App Paths、`.cllens` Classes、开始菜单、桌面快捷方式和 GUI 默认入口。
- 检查 WebView2；缺失时引导官方 Evergreen bootstrapper。
- 完成页解释 PATH、注册表和 WebView2 注册为什么可能需要重启。
- 卸载时将锁定文件移入 `.trash/pending`；只有用户勾选或传入 `/CLEANUSERDATA` 才删除会话数据。

## E2E

```powershell
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 `
  -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

E2E 会在含空格和中文的目录中执行静默安装，检查 CLI/GUI 版本、doctor、PATH、注册表、GUI 冷启动、五项残留，完成卸载和重装。ARM64 在 ARM64 runner 上复用同一套脚本。

`v1.0.0` 之前所有版本都是 beta/测试版。完整文档见[三语目录](../docs/README_CN.md)。
