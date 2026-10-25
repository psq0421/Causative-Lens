# 安装器行为规格

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

语言：安装器页面支持 en-US、zh-CN、zh-TW；payload、注册表键和静默参数保持语言中立。

## 安装流程

1. 检测架构、既有版本、安装路径、用户权限、WebView2；路径可含空格、中文、OneDrive、>260字符。2. 解压当前架构文件。3. 幂等写入用户级 PATH 并广播 `WM_SETTINGCHANGE`。4. 写入 Uninstall、App Paths、`.cllens` Classes 和图标注册表。5. 创建桌面快捷方式、开始菜单项（GUI置顶）。6. 检测 WebView2，缺失则引导 Evergreen/离线 bootstrapper。7. 写安装清单、版本 `0.1.0`、SHA-256。8. 完成页提供“立即重启/稍后”，说明 PATH/注册表/WebView2 生效原因。

## 静默参数

`/S` 静默安装；`/D=<绝对路径>` 自定义路径（必须为最后参数）；`/NORESTART` 禁止自动重启但保留退出码 3010；`/LOG=<path>` 输出日志；`/CLEANUSERDATA` 卸载时清理会话数据。参数解析失败退出 2，安装失败 1，成功 0，成功但需重启 3010。

## 升级、覆盖、降级

同架构同产品升级保留 `%LOCALAPPDATA%\causativelens`；先停止 GUI/daemon，备份注册表和安装清单，再原子替换文件。降级默认拒绝并退出 5；显式 `/ALLOWDOWNGRADE` 才允许，且保留数据 schema 兼容检查。跨架构安装需先卸载并提示用户。

## 失败回滚

任一步失败按逆序删除新文件、恢复 PATH/注册表/关联/快捷方式，保留日志；已存在用户文件不删除。回滚本身失败时写 `.trash\<timestamp>` 并在日志列出可恢复路径。

## 卸载

停止并等待 daemon/GUI/capture；删除安装文件、Uninstall/App Paths/Classes/Path 条目、快捷方式和关联；询问是否删除 `%LOCALAPPDATA%\causativelens` 会话数据。锁定文件先改名到 `.trash`，重启后清理。卸载自检五项必须全空：文件、注册表、PATH、关联、快捷方式；否则返回 1。之后 `clens doctor` 应显示“未安装”。

## G7 实现映射

- NSIS `RequestExecutionLevel user` 保证 per-user 安装；不创建服务、不写 HKLM。唯一提权组件是 payload 中的 `clens-capture.exe.manifest`，其 `requestedExecutionLevel` 为 `requireAdministrator`。
- 一个安装包同时包含 `clens.exe`、`clens-gui.exe`、`clens-daemon.exe` 和 `clens-capture.exe`。GUI/CLI 的版本由 `payload-manifest.json` 和安装后 `--version` 检查锁定为 `0.1.0`。
- 正式签名只能由 Azure Trusted Signing 完成；`installer/sign-release.ps1` 缺少 DLib/metadata 时输出 `unsigned-dev` 并拒绝 `-Release -Signed` 之前的正式发布路径。签名验证使用 `signtool verify /pa /all`，时间戳为 `http://timestamp.acs.microsoft.com`。
- `installer/installer-e2e.ps1` 是安装器 E2E 事实来源：静默安装、路径/版本/doctor、GUI 冷启动、五项卸载残留、重装；报告写入 `dist/installer-e2e-report.json`。
- `install-manifest.json` 只写可跨平台解析的产品/版本/架构/模式字段；payload 文件哈希在 `payload-manifest.json`，避免把 Windows 反斜杠绝对路径未经 JSON 转义写入清单。


