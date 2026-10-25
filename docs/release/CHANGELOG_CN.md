# 变更记录

## 0.1.0

- 交付 observe-only 的 G0–G7 发布列车：ETW 采集、Harness 接入、因果对齐、CLI、daemon、多运行时适配和 Tauri GUI。
- 增加 GUI/CLI 一体 NSIS per-user 安装器：用户级 PATH、HKCU 注册表、`.cllens` 关联、快捷方式、WebView2 引导、升级/降级参数和 `.trash` 卸载兜底。
- 增加 x64 安装器 E2E：静默安装、版本/doctor、GUI 冷启动、卸载五项残留检查和重装。
- 增加 SPDX SBOM 与 Azure Trusted Signing 阻断式发布流水线；没有正式凭据时只生成 `-dev` 包。
- 增加 English (US)、简体中文、繁體中文界面、安装器文案和文档导航。
- 干预层 `advise`/`enforce` 与 Jev 决策后端顺延到 v0.2.0。

第一版是望远镜，不是方向盘：它记录并解释，却不触碰被观测的进程。
