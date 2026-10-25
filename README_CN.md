# CausativeLens

> **给那些“软件为什么会这样”的瞬间，点一盏小灯。**

CausativeLens v0.1.0 是 Windows 上的只读因果观测工具。一个 `clens-daemon` 持有引擎状态，`clens.exe` 与 `clens-gui.exe` 通过 `causativelens-rpc` 作为两个前端工作。系统记录 ETW 元数据和结构化 Agent 轨迹，把断言与真实事件对齐，并保留可回放的证据。

> **当前 Beta 限制（2026-10-25）：** 上述是目标架构。`run/attach` 目前返回 `offline`，厂商轨迹自动接入及完整的采集常驻/持久化链路尚未接通。实时监测前请先阅读[17 款产品接入教程与已验证状态](docs/guide/runtime-integration_CN.md)。

## 语言

产品支持 **简体中文**、**繁體中文** 与 **English (US)**。

- CLI：`--lang en-US|zh-CN|zh-TW`，或设置 `CAUSATIVELENS_LANG`。
- GUI：在标题栏选择语言，选择会按用户记住。
- 安装器：提供 English、简体中文、繁體中文安装页面。
- JSON/RPC：字段名和枚举值保持英文且稳定，方便自动化。

语言切换：[English](README.md) · [繁體中文](README_TW.md) · [文档目录](docs/README_CN.md)。

## 从这里开始

- [快速入门](docs/快速入门.md) · [Quick start](docs/quick-start.md) · [快速入門](docs/快速入門.md)
- [17 款产品接入教程与当前限制](docs/guide/runtime-integration_CN.md)
- [文档目录](docs/README_CN.md)
- [架构与契约](docs/architecture/README_CN.md)
- [发布说明](docs/release/CHANGELOG_CN.md)

## 本地构建

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.1.0
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.1.0
```

安装器按当前用户安装 GUI、CLI、daemon 和 capture。只有 `clens-capture.exe` 请求 UAC；GUI 是桌面默认入口，`.cllens` 文件会关联到 GUI。

## 只读边界

v0.1.0 只做观测。干预与决策命令顺延到 v0.2.0；相关数据表保留前向兼容，但没有写入路径。这条边界是一份承诺：镜头可以解释脚印，却不会伸手改写被观测的程序。

## 发布安全

`v1.0.0` 之前的所有版本都是 beta 测试版。`v0.1.0` prerelease 使用明确标注的 x64 `-dev` 安装包；该包未签名，ARM64 暂缓。稳定版需要 x64 与 ARM64 签名安装包、SBOM、确定性构建证据和安装器 E2E 矩阵，见[发布闸门](docs/release/g7-release-gate_CN.md)。

## 贡献

修改契约前请阅读[贡献指南](docs/guide/contributing_CN.md)与[双语文档目录](docs/README_CN.md)。保持提交小而清晰，保持 JSON 稳定，也给未来的维护者留一条能听见你思路的测试。
