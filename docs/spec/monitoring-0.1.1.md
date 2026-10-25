# v0.1.1 observe 监听契约

- monitor.templates：无参数，返回数据化模板。模板 ID 与显示语言分离。
- monitor.probe：{runtime_id,folder,include_profile}，目录必须存在；返回实际 roots、候选文件、识别格式和导出指导。
- monitor.start：同 probe 参数，加 read_existing（默认 true）；返回 session_id。读取线程不执行 Agent 命令、不改厂商配置。
- monitor.status：{session_id}，返回运行状态、已读/识别/拒绝行、轮换/截断、丢弃及最近错误；没有真实 ETW 时 capture_connected=false。
- monitor.events：{session_id,after,limit}，after 为 Timeline 单调游标；响应有界，返回 rows/next_cursor/has_more/retained_from，读取不消费其他前端的数据。
- monitor.stop：{session_id}，停止读文件，保留内存 Timeline 供查询；可重复停止。
- monitor.list：返回 daemon 中仍存在的监听会话，GUI 重开可以选择。
- session.close：同时停止该会话 reader，再释放内存。

分类 assertions/files/network/other。来源 assertion/agent/system 分离；file 工具请求只代表客户端意图，不是 ETW FileRead。tool_result 不重复产断言。每条 Timeline 数据带 seq、ts_utc_ms、category、source、kind、target、text、state。

格式入口：规范 Harness JSONL、Anthropic 消息、OpenAI tool_calls、Codex exec/session、OpenClaw transcript、Hermes sessions 导出/observer、一般结构化 tool 事件。Markdown/普通文本仅为文本，SQLite 二进制不以文本解码。JSON 解析失败或未知对象计数可见。fixtures 标记 synthetic，不宣称全厂商最近三个月真实版本兼容。

安全：扫描只读、不追符号链接/重解析点、不读凭据文件；扫描用户目录需要 UI 显式勾选。完整提示词/工具轨迹可能敏感，GUI 说明本地处理。监听不自动开启 UAC、不宣称本机 ETW 能看到云/WSL 内部操作。
