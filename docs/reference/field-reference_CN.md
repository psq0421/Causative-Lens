# 字段参考

跨模块核心数据是 `SysEvent`、`Assertion`、`Verdict`。每个结构带 `schema_v`；`seq` 是 capture 全局单调序号，`assertion_id` 由日志/行/槽位稳定计算。`.cllens` 保存索引和履约图，`.clspool` 保存高频原始事件。

JSON/RPC 字段名、命令名、错误码和文件扩展名不随界面语言变化。用户文案按 [语言支持](../guide/language-support_CN.md) 选择。

语言：[English](field-reference.md) · [简体中文](field-reference_CN.md) · [繁體中文](field-reference_TW.md)。
