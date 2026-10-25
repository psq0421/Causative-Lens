# v0.1.0 测试计划

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

语言矩阵：CLI、GUI 和 NSIS 均覆盖 en-US、zh-CN、zh-TW；协议 JSON 只做稳定英文断言。

## 测试金字塔

保留蓝图第14章的单元、契约、集成、合成 ETW fixture、端到端层，并追加安装器 E2E。单元覆盖 IR/时间窗/错误码；契约覆盖 JSON-RPC/schema/迁移；集成覆盖 capture-spool-daemon；fixture 覆盖 12 剧本；E2E 覆盖真实 Harness 五运行时。

## 安装器 E2E 主链

静默安装 → 立即/稍后重启矩阵 → `clens doctor` 全绿 → 最小会话 `run`/`open`/`why` 闭环 → GUI 冷启动达标 → 卸载 → 文件/注册表/PATH/关联/快捷方式五项残留检查 → 重装。每次保存日志、退出码、截图和包哈希。

## 双架构矩阵

x64 与 ARM64 均执行构建、签名验证、安装路径边界、WebView2 缺失、中文/OneDrive/>260路径、升级/降级、静默/卸载。ARM64 在原生 runner 做冒烟，x64 做完整性能基准。

## 性能门禁

包体<25MB；GUI冷启动<600ms；10万事件交互≥45fps；`graph.delta` p95<8ms/批；GUI RSS<150MB。测量方法写入 `implementation-plan.md`，CI 失败即阻断 G5/G6。

## CI 门禁总表

| 门 | 命令/证据 | 阻断 |
|---|---|---|
| 格式/静态 | `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo deny check` | G0+ |
| 单元/契约 | `cargo test --workspace`; schema/迁移 fixture | G0+ |
| GUI构建 | `cargo tauri build --target ...`; 架构依赖扫描 | G5 |
| ETW/fixture | `cargo test -p clens-capture`; 12剧本 | G1/G3 |
| 安装器E2E | Pester 静默装/卸载/残留/重装脚本 | G6 |
| 签名/SBOM | `signtool verify /pa`; SPDX 文件存在且哈希匹配 | G6/G7 |
| 发布 | 双 exe、doctor、20会话报告 | G7 |


