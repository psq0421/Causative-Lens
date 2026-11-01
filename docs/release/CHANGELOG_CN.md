# 更新日志

## 0.2.5 测试版（2026-11-01）

> **这是该项目迄今最大的一次更新。**

- GUI 扩展为七段式旅程：会话 → 观测 → 探查 → 回放 → 对比 → 辅助 → 干预。
- 新增菜单栏、快捷键、断点优先探查、回放控制、双会话对比、证据锚定辅助分析和常驻健康度指标。
- 新增工作区覆盖、observe-only 锁定和 GUI 仅走 RPC 的架构契约。
- Rust crate、CLI、GUI、Tauri、安装器、payload 与 lockfile 版本统一为 `0.2.5`。
- 更新三语快速入门、监听向导与发布说明。
- 明确 `v1.0.0` 之前均为 beta/测试版，并继续保持 observe-only 边界。
- 增加可复现代码规模报告、稀土掘金长文和老少皆宜的通俗分享版。

完整说明：[简体中文](v0.2.5.md) · [English](v0.2.5.en-US.md) · [繁體中文](v0.2.5_TW.md)。

## 0.2.0 测试版（2026-10-31）

- 接入 Windows ETW bridge、四 provider 消费线程、TDH 解码、路径驻留、FileObject 跟踪和有界丢失统计。
- 增加 WAL SQLite 履约图持久化、FTS5 索引、离线 `.cllens` 打开和持久化 `why` 链。
- 增加对齐与 spool 恢复的 Criterion 基线。
- 发布证据闭环方案，明确采集层限制。

## 0.1.2 测试版（2026-10-26）

- 增加 Jev、OpenAI-compatible、Anthropic-compatible 与 Ollama 的可选证据辅助分析。
- 增加 GUI/CLI 设置：endpoint、模型、隐私、超时、保留条数和语言。
- 把真实 ETW、mmap ring 与持久化缺口写成可核查方案，不把演示轨迹冒充内核证据。

## 0.1.1 测试版（2026-10-25）

- 增加首次监听向导、17 家产品模板与 Others、时间线筛选、搜索和 GUI/CLI 对等检查。
- 增加 x64 安装器 E2E、卸载残留检查与多语言文档。

## 0.1.0 测试版（2026-10-25）

- 建立 observe-only 架构：一个 daemon、两个 RPC 前端和最小特权 capture 边界。
- 增加初版 CLI、GUI 壳、契约、fixtures 与发布门禁文档。
