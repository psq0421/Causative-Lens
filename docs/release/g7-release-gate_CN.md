# G7 发布闸门

只有具备以下证据才允许正式发布：x64/ARM64 安装包 `CausativeLens-setup-x64.exe` 与 `CausativeLens-setup-arm64.exe`；每个可执行文件都有 Azure Trusted Signing 验证与时间戳；SPDX SBOM、锁定依赖、确定性构建元数据和哈希；干净 Windows VM 通过静默安装、重启、`clens doctor`、最小会话、GUI 冷启动/RSS/帧率、卸载五查和重装；英文、简体中文、繁體中文文档链接全部可用。

没有签名凭据的机器可以构建和测试 `-dev` 包，但不能创建正式 Release。闸门刻意保持枯燥：软件用一个可观察的检查接一个可观察的检查赢得信任。

语言：[English](g7-release-gate.md) · [简体中文](g7-release-gate_CN.md) · [繁體中文](g7-release-gate_TW.md)。
