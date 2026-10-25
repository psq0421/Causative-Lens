# G2 Observe-only JSON-RPC 契约

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

协议以 Named Pipe `\\.\pipe\causativelens-rpc` 为传输目标；G2 先固定无 I/O 的请求处理契约，G5 再接入实际管道。

请求是单行 JSON：`{"id":1,"method":"hello","params":{...}}`。响应包含同一 `id`，成功时只有 `result`，失败时只有 `error`。

有效方法：

- `hello`：参数 `{protocol_version:1,mode:"observe"}`，返回产品、开发版本和 observe 模式。
- `session.open`：创建 observe 会话票据。
- `session.status`：返回 `{mode:"observe",active:true}`。

`advise`、`enforce`、`decision.*` 始终返回 `intervention_unavailable`。未知方法返回 `method_not_found`。协议版本不为 1 返回 `unsupported_protocol`。v0.1.0 不存在任何干预或决策写入路径。

