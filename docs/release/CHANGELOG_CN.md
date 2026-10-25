# 变更记录

## 0.1.2 Beta（2026-10-26）

- 新增可选 `clens-assist` 辅助层，支持证据约束的 Jev、OpenAI-compatible、Anthropic-compatible 和本机 Ollama 分析。
- GUI 新增“设置”工作区，支持提供方、Endpoint、模型、隐私、超时、时间线、语言和主题配置。
- 新增 `clens assist` 与 `clens why --assist`；API key 只留在进程内存，不写入 `settings.json`。
- 强制引用可追溯：模型 finding 必须引用提交因果链中真实存在的断言 ID 或事件序列。
- 加固 Endpoint/header 校验、Windows 设置重复保存、相对路径脱敏和 CLI 人类可读输出。
- 保持 observe-only 边界：Jev 决策、干预层、ETW 生产闭环和 ARM64 运行态仍不在本 beta 的已验证范围内。
- 应维护者要求，本版本以 GitHub 普通 Release（非预发布）发布；`v1.0.0` 之前的软件仍属于测试软件。构建机没有 Azure Trusted Signing 凭据，上传的 x64 安装包明确标记为未签名。
- [完整 v0.1.2 说明](v0.1.2.md)。
## 0.1.1 Beta（2026-10-25）

- 重做 Fluent 监听向导，提供 20 项按内容识别的运行时模板。
- 修复真实 Timeline 筛选，增加搜索/详情/跟随/导出；修复 CLI 快捷方式瞬退。
- 新增有界轨迹读取与 daemon 监听 RPC，客户端报告与系统证据明确区分。
- 新增浏览器 E2E、监听模块覆盖率、Tauri 构建与安全的安装器 E2E。
- [完整中英更新说明与剩余发布门槛](v0.1.1.md)。

## 0.1.0 Beta（2026-10-25）

这是测试版。`v1.0.0` 之前的所有版本均以 beta 测试版本发布。

- 交付 observe-only 的 G0–G7 发布列车：ETW 采集、Harness 接入、因果对齐、CLI、daemon、多运行时适配和 Tauri GUI。
- 增加 GUI/CLI 一体 NSIS per-user 安装器：用户级 PATH、HKCU 注册表、`.cllens` 关联、快捷方式、WebView2 引导、升级/降级参数和 `.trash` 卸载兜底。
- 增加 x64 安装器 E2E：静默安装、版本/doctor、GUI 冷启动、卸载五项残留检查和重装。
- 增加 SPDX SBOM 与 Azure Trusted Signing 阻断式发布流水线；没有正式凭据时只生成 `-dev` 包。
- 增加 English (US)、简体中文、繁體中文界面、安装器文案和文档导航。
- 干预层 `advise`/`enforce` 与 Jev 决策后端顺延到 v0.2.0。
- 发布经过测试的 x64 预览安装包 `CausativeLens-setup-x64-dev.exe`，并附 SHA-256 记录、SPDX SBOM、签名状态和安装器 E2E 报告。
- 本次 beta 有意暂缓 ARM64；稳定版 `v1.0.0` 必须具备已签名的 x64/ARM64 安装包和完整干净虚拟机矩阵。

第一版是望远镜，不是方向盘：它记录并解释，却不触碰被观测的进程。
