# T0104 CLI 快捷方式

- 用户问题：双击 CLI 后立即退出，看起来无法打开。
- 决策：开始菜单快捷方式打开 `cmd.exe /D /K`，显示 CLI 帮助后保留提示符；安装目录作为工作目录，含空格和 Unicode 的路径完整引用，使用产品图标。`/D` 禁止运行用户 AutoRun 脚本。
- 不把当前占位 `shell` 描述成已实现的 REPL。
- DoD：`powershell -NoProfile -File installer/test-cli-shortcut.ps1 -InstallDir "$env:LOCALAPPDATA\causativelens"`；安装器 E2E 也运行同一检查。
- 验收（2026-10-25）：本机已存在的开始菜单快捷方式已修复，原 DoD 通过：`cmd.exe /D /K` 路径引用、工作目录、图标正确；隐藏测试进程在 1200ms 后仍存活，随后只终止测试进程。CLI 帮助同步增加三语快速操作与实际能力说明。安装器生成快捷方式的验证留给 T0103 E2E。
