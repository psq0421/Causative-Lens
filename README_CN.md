# CausativeLens

> **给那些“软件为什么会这样”的瞬间，点一盏小灯。**

CausativeLens v0.1.2 Beta 是 Windows 上的只读 Agent 轨迹观测工具。选软件、选目录、检查并监听。一个 `clens-daemon` 持有数据，`clens.exe` 与 `clens-gui.exe` 通过 `causativelens-rpc` 工作。

> **当前 Beta 能力与边界（2026-10-25）：** GUI 已可持续读取支持的本机轨迹，提供 17 家产品、WorkBuddy 两渠道、DeepSeek 与 Others 共 20 个选择项，按内容识别，不代表所有厂商版本均实测。完整 ETW/持久化链路仍未接通，客户端报告与系统证据明确标源；`run/attach` 仍为离线占位。请从[第一次监听](docs/guide/monitoring-start_CN.md)开始。

## 语言

产品支持 **简体中文**、**繁體中文** 与 **English (US)**。

- CLI：`--lang en-US|zh-CN|zh-TW`，或设置 `CAUSATIVELENS_LANG`。
- GUI：在标题栏选择语言，选择会按用户记住。
- 安装器：提供 English、简体中文、繁體中文安装页面。
- JSON/RPC：字段名和枚举值保持英文且稳定，方便自动化。

语言切换：[English](README.md) · [繁體中文](README_TW.md) · [文档目录](docs/README_CN.md)。

## 从这里开始

- [快速入门](docs/快速入门.md) · [Quick start](docs/quick-start.md) · [快速入門](docs/快速入門.md)
- [v0.1.2 新手监听向导](docs/guide/monitoring-start_CN.md) · [厂商轨迹准备教程](docs/guide/runtime-integration_CN.md)
- [文档目录](docs/README_CN.md)
- [架构与契约](docs/architecture/README_CN.md)
- [发布说明](docs/release/CHANGELOG_CN.md)

## 本地构建

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.1.2
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.2
```

安装器按当前用户安装 GUI、CLI、daemon 和 capture。只有 `clens-capture.exe` 请求 UAC；GUI 是桌面默认入口，`.cllens` 文件会关联到 GUI。

## 只读边界

v0.1.0 只做观测。干预与决策命令顺延到 v0.2.0；相关数据表保留前向兼容，但没有写入路径。这条边界是一份承诺：镜头可以解释脚印，却不会伸手改写被观测的程序。

## 发布安全

`v1.0.0` 之前的所有版本都是 beta 测试版。`v0.1.2` GitHub Release 按要求作为普通非预发布测试版本发布；本机 x64 包在提供 Azure Trusted Signing 凭据前保持未签名，ARM64 暂缓。稳定版需要 x64 与 ARM64 签名安装包、SBOM、确定性构建证据和安装器 E2E 矩阵，见[发布闸门](docs/release/g7-release-gate_CN.md)。

## 贡献

修改契约前请阅读[贡献指南](docs/guide/contributing_CN.md)与[双语文档目录](docs/README_CN.md)。保持提交小而清晰，保持 JSON 稳定，也给未来的维护者留一条能听见你思路的测试。
