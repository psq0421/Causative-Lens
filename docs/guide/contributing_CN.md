# 贡献指南

让证据链短到可以审计，让文字暖到不会忘记我们为什么做这件事。

1. 阅读 `docs/spec/` 对应契约和 `docs/plan/task-cards/` 对应任务卡。
2. 保持 v0.1.0 只读；干预与决策属于 v0.2.0。
3. 修改契约或解析器前先补测试。
4. 运行 fmt、workspace 测试和 `cargo clippy ... -D warnings`。
5. 用户可见文案必须补齐三种语言。
6. 在闸门报告记录兼容性和测量证据。

任务卡提交使用 `[T0xxx]` 前缀。优先提交可以独立回滚、不会丢掉故事线的小改动。

语言：[English](contributing.md) · [简体中文](contributing_CN.md) · [繁體中文](contributing_TW.md)。
