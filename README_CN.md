# CausativeLens

> **给那些“软件为什么会这样”的瞬间，点一盏小灯。**

**CausativeLens v0.2.5 是本项目迄今最大的一次更新。** 它是一款 Windows 只读 Agent 轨迹观测器：把推理步骤编译成因果断言，再用可用的系统证据逐条核验。一套 `clens-daemon` 持有证据，`clens.exe` 与 `clens-gui.exe` 是共享 RPC 的两个前端。

> **证据边界：** v0.2.5 延续 observe-only 契约。GUI 现在覆盖“会话→观测→探查→回放→对比→辅助”的完整旅程。内核证据仍需要提权 capture；不可用时，产品会明确报告，而不会把 Harness 日志冒充 ETW 事实。

## 语言

产品支持 **简体中文**、**繁體中文** 与 **English (US)**。

- CLI：`--lang en-US|zh-CN|zh-TW`，或设置 `CAUSATIVELENS_LANG`。
- GUI：在标题栏选择语言，选择会按用户记住。
- 安装器：提供三种语言页面。
- JSON/RPC：字段名与枚举值保持英文稳定。

语言切换：[English](README.md) · [繁體中文](README_TW.md) · [文档目录](docs/README_CN.md)。

## 从这里开始

- [快速入门](docs/快速入门.md) · [Quick start](docs/quick-start.md) · [繁體中文快速入門](docs/快速入門.md)
- [第一次观测](docs/guide/monitoring-start_CN.md) · [运行时接入教程](docs/guide/runtime-integration_CN.md)
- [文档目录](docs/README_CN.md)
- [架构与契约](docs/architecture/README_CN.md)
- [v0.2.5 发布说明](docs/release/v0.2.5.md)

## 本地构建

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

per-user 安装器包含 GUI、CLI、daemon 与 capture，只有 `clens-capture.exe` 请求 UAC。GUI 是桌面默认入口，`.cllens` 文件关联到 GUI。

## 只读边界

CausativeLens 负责观测与解释。干预和决策命令继续不注册；相关 schema 保持前向兼容，但没有写入路径。镜头可以解释脚印，不会伸手改写被观测程序。

## 发布安全

`v1.0.0` 之前的版本都是 beta/测试版。v0.2.5 作为普通 GitHub 测试版发布；本机构建以 x64 为主，如果没有 Azure Trusted Signing，上传的安装包会明确标记为 `unsigned-dev`。稳定版仍要求签名的 x64/ARM64 安装器、SBOM、确定性构建证据和干净虚拟机安装器矩阵。详见 [v0.2.5 发布说明](docs/release/v0.2.5.md)。

## 贡献

修改契约前请阅读[贡献指南](docs/guide/contributing_CN.md)和[双语文档目录](docs/README_CN.md)。保持提交小而清晰，保持 JSON 稳定，也给未来维护者留下一条能听见你思路的测试。