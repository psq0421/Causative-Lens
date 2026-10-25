# v0.1.2：第一次监听

[English](monitoring-start.md) · [简体中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> 给程序留一条回家的线索。先看见轨迹，再讨论证据。

## 安装与打开

运行 x64 `-dev` 测试安装包。安装为 per-user，不需要管理员权限。双击桌面 CausativeLens；CLI 在开始菜单的 CausativeLens CLI 中打开，帮助结束后保留命令提示符。右上角可切换简体中文、繁體中文、English (US) 和深浅色。ARM64 暂缓，测试包尚无正式 Authenticode 签名。

## 三步开始

1. 点击 **监听 → 请选择你使用的软件**。例如选 OpenClaw，点击下一步。
2. 点击 **浏览**，选择项目、日志或安装文件夹。默认检查该软件的用户日志目录，可能包含其他会话；不希望读取时请取消勾选。取消“载入已有近期轨迹”可以只读从现在起新增的内容。
3. 点击 **检查目录**，核对实际扫描范围与候选文件，再点击 **开始监听**。目录为空时可以进入等待状态，然后让 Agent 做一个小任务，或按下表导出轨迹。

安装目录未必包含日志。选好目录仅表示配置就绪；只有“已识别记录”增长才说明轨迹进入。未知 schema/坏 JSON 进入健康错误计数，不能假装兼容。程序不执行 Agent 命令、不修改它的配置、不调用云 API。

## 20 项模板与输入准备

17 家产品中 WorkBuddy 分国内/国际渠道，另保留 DeepSeek 与 Others，共 20 个选择项。按内容识别，不强制限制版本；未知结构仍会明确报错。以下内置提示不是近三个月全部真实版本的验证矩阵。

| 软件 | 输入准备 |
|---|---|
| Hermes Agent | 选择 Hermes 会话导出或 observer 日志目录。state.db 不能作为文本读取；没有轨迹时请先导出会话。 |
| Claude Code | 自动识别 Claude 消息及 tool_use。CLI 可用 stream-json；IDE/Desktop 与远程工具需独立确认。 |
| OpenAI Codex | 识别 Codex exec JSONL 及会话轨迹；不会获取隐藏思维链。 |
| Google Antigravity CLI | 读取当前版本导出的结构化轨迹。不要将 Gemini CLI 参数套给 Antigravity。 |
| OpenClaw | 请选择 gateway 的会话轨迹目录。仅有网关健康日志时不会产生工具断言；远端动作不是本机 ETW。 |
| Manus | 请选择任务活动导出目录。云任务最终报告不能代替逐步工具轨迹或本机系统证据。 |
| AutoGPT | 选择实际 worker 的任务日志目录。Classic/Platform/容器分别确认；普通日志只显示文本。 |
| CodeBuddy | 读取 CodeBuddy stream-json 或会话导出；保留正常权限设置。 |
| WorkBuddy（国内版） | 选择国内版任务活动导出目录；与国际版独立识别，不假设存在同名 CLI。 |
| WorkBuddy International | 选择国际版任务活动导出目录；不要混入国内版会话。 |
| Trae Agent / TRAE CLI | 选择实际执行器轨迹；IDE、开源 Agent 与 CLI 的协议可能不同。 |
| Aider | 识别聊天历史 Markdown 和结构化导出；聊天文字不会被伪造成实际写文件。 |
| ZCode | 选择 ZCode 客户端轨迹。其他客户端仅更换智谱模型时，应选择实际客户端模板。 |
| Qoder CN / Qoder CLI | 选择 CLI stream-json 或当前渠道会话导出；CN/国际版本需分别验证。 |
| CodeArts Agent CLI | 选择当前发行版的结构化活动导出，记录华为云区域和执行位置。 |
| MiniMax Code CLI | 若是 Claude/Codex 使用 MiniMax 模型，请选择对应客户端；独立 CLI 选择它的轨迹目录。 |
| GitHub Copilot CLI | 选择 standalone Copilot 的会话事件目录；旧 gh 扩展、IDE 与云端任务独立验证。 |
| Cursor CLI | 选择 Cursor agent 的 stream-json 输出目录；不要将编辑器启动器当作执行器。 |
| DeepSeek Harness | 选择规范 Harness JSONL 或结构化工具轨迹。 |
| Others / 其他软件 | 自动探测通用 Harness、OpenAI/Anthropic 工具消息及文本；未知 JSON 会计数并提示，不伪造适配成功。 |

## 时间线、详情与查询

- **全部 / 断言 / 文件 / 网络**：点击筛选真实记录，计数对应最近 5,000 条视图。搜索可叠加筛选，没有结果时显示空态。
- **来源**：Agent 轨迹是客户端报告，断言是提取的意图，系统证据须来自真实采集。当前 GUI 监听不连接 ETW，履约保持未观测。
- 点击记录或按 Enter/空格打开详情，Escape 关闭，复制 JSON 便于诊断。跟随按钮控制是否滚动到最新记录；导出保存当前视图 JSON。
- **履约图**显示提取的断言，不画不存在的证据边。**查询**输入完整路径，获取 daemon 的五段因果链。
- **停止监听**释放读取器，仍可查看记录。GUI 关闭后 daemon 继续读取，重开可恢复。daemon 退出会清空内存视图，请及时导出；JSON 导出不是 `.cllens` 持久会话。

## Others：通用模板

在选定目录创建 `trace.jsonl`，每行一个 JSON：

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

这只验证轨迹接入。把文件名写进工具参数不意味着操作真的发生，程序不会制造 ETW 证据。JSONL 读完一行才解析，JSON 导出按整个快照读取，文本/Markdown 只展示文本。

## 排障与上限

目录无权限：重新选可读目录。目录无日志：按模板导出或明确勾选用户日志。SQLite `state.db` 不按文本扫描。记录待检查：看健康信息中的未知结构、截断和超限提示。云端、WSL、容器只能接收公开导出的轨迹，本机不是远端采集器。旧 CLI 快捷方式仍关闭时，用新安装器覆盖安装，或在终端运行 `clens --help`。

扫描上限：深度 6、8192 项、128 文件；每轮每文件 256KiB；JSON 快照最多 256KiB；队列 4096；视图 5000 条。超限和跳过字节在健康信息中可见。不会读取凭据文件或修改 Agent。

[17 家详细轨迹获取教程（v0.1.0 能力核查背景）](runtime-integration_CN.md) · [变更记录](../release/CHANGELOG_CN.md)
