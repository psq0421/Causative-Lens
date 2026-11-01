# v0.2.5 observe 监听契约

本契约描述 GUI 监听向导与 daemon 的只读 RPC。它只接收已有轨迹，不启动 Agent，不修改厂商配置，也不把目录选择宣称为 ETW 证据。

## RPC 方法

- `monitor.templates`：无参数，返回数据化模板；模板 ID 与显示语言分离。
- `monitor.probe`：`{runtime_id,folder,include_profile}`；目录必须存在；返回实际扫描根、候选文件、识别格式和导出指导。
- `monitor.start`：在 probe 参数上增加 `read_existing`（默认 `true`）；返回 `session_id`。读取线程不执行 Agent 命令、不改厂商配置。
- `monitor.status`：`{session_id}`；返回运行状态、已读/识别/拒绝行、轮转、截断、丢弃、最近错误，以及 `capture_connected`。没有真实 ETW 时必须为 `false`。
- `monitor.events`：`{session_id,after,limit}`；`after` 是 Timeline 单调游标；响应有界，返回 `rows`、`next_cursor`、`has_more`、`retained_from`，读取不消费其他前端的数据。
- `monitor.stop`：`{session_id}`；停止文件 reader，保留内存 Timeline 供查询；重复停止必须幂等。
- `monitor.list`：返回 daemon 内仍存在的监听会话，GUI 重开可以选择。
- `session.close`：停止该会话 reader，再释放内存；关闭不写入虚假的 `.cllens`。

## 行归类

Timeline 行分为 `assertions`、`files`、`network`、`other`。来源 `assertion`、`agent`、`system` 分离：file 工具请求代表客户端意图，不等于 ETW `FileRead`；`tool_result` 不重复生成断言。每行至少包含 `seq`、`ts_utc_ms`、`category`、`source`、`kind`、`target`、`text` 和 `state`。

## 输入格式

接受规范 Harness JSONL、Anthropic message、OpenAI tool call、Codex exec/session、OpenClaw transcript、Hermes session export/observer 与通用结构化 tool 事件。Markdown/普通文本只作为文本；SQLite `state.db` 不按文本扫描。解析失败和未知对象进入可见计数，不能假装兼容。fixtures 标注 `synthetic` 时，不得宣称厂商最近三个月版本认证。

## 安全与边界

扫描只读，不跟随符号链接，不读取凭据文件。扫描用户目录必须由 UI 明确勾选。完整提示词与工具轨迹可能敏感，GUI 明确说明本地处理。监听不自动开启 UAC，也不宣称能看见云端、WSL 或容器内部操作；这些活动只能通过导出的轨迹显示。

语言切换：[English](monitoring-0.1.1.en-US.md) · [简体中文](monitoring-0.1.1.md) · [繁體中文](monitoring-0.1.1_TW.md)。
