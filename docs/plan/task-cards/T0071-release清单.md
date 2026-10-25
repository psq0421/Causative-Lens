# T0071-release清单

- 闸门：G7
- 发布形态：GitHub Releases 只上传 `CausativeLens-setup-x64.exe` 与 `CausativeLens-setup-arm64.exe`；zip 仅 CI 内部产物。
- 版本：workspace crate、Tauri 配置、CLI/GUI 和安装注册表统一 `0.1.0`；observe-only，干预/决策命令不注册。
- 发布前必须具备：两架构 payload manifest、SHA-256、SPDX SBOM、Trusted Signing 验证和签名指纹、确定性构建元数据、E2E 报告、CHANGELOG 0.1.0。

## DoD

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/generate-sbom.ps1 -Version 0.1.0
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

- 完成记录：本机 x64 `-dev` 包及 E2E 已完成；正式 Release 仍由签名 gate 阻断。
- 遗留：ARM64 原生 runner、Azure Trusted Signing、干净 VM 矩阵和 GitHub Release 上传。


Language navigation: [English task-card catalog](README.en-US.md) · [简体中文任务卡](README.md) · [繁體中文任務卡](README_TW.md)。

