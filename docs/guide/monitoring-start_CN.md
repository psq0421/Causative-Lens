# v0.2.5：第一次监听

[English](monitoring-start.md) · [简体中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> 给程序留一条回家的线索。先看见轨迹，再讨论证据。

## 安装与打开

运行 x64 `-dev` 测试安装包。安装为当前用户生效；双击桌面快捷方式打开 GUI，开始菜单中的 CLI 快捷方式会在帮助结束后保留终端窗口。标题栏可以切换简体中文、繁體中文、English (US) 与浅色/深色外观。ARM64 暂缓，测试包没有生产级 Authenticode 签名。

## 三步开始

1. 打开 **监听 → 请选择你使用的软件**，例如选择 OpenClaw，然后点击 **下一步**。
2. 点击 **浏览**，选择项目、日志或安装文件夹；只想读取新增内容时，取消“读取已有近期轨迹”。
3. 点击 **检查目录**，确认实际扫描根和候选文件，再点击 **开始监听**。目录为空时可以保持等待，然后让 Agent 完成一个小任务，或使用下面的通用 JSONL 示例。

配置了目录不代表已经接入。请观察“已识别记录”计数和健康度条；未知 schema、坏 JSON、轮转和截断都会保留为可见状态。监听器不会启动 Agent、修改配置或调用云端 API。

## 模板与输入准备

共有 20 个选项：17 家产品（WorkBuddy 国内版和国际版分开）以及 DeepSeek Harness、Others。模板按内容识别，不强制锁定版本；未知格式会明确报告。模板不代表最近三个月每个厂商版本都经过认证。

| 软件 | 输入准备 |
|---|---|
| Hermes Agent | 选择 session export 或 observer 日志；只有 `state.db` 时先导出 |
| Claude Code | 选择 message/tool_use、CLI stream-json 或支持的 transcript |
| OpenAI Codex | 选择 exec JSONL 与 session trace；不会读取私有思维链 |
| Google Antigravity CLI | 选择当前版本导出的结构化轨迹 |
| OpenClaw | 选择 gateway session transcript；健康日志本身不产生工具断言 |
| Manus | 选择 task activity export；云端报告不是本机系统证据 |
| AutoGPT | 选择实际 worker 日志并确认 Classic/Platform/容器变体 |
| CodeBuddy | 选择 stream-json 或 session export |
| WorkBuddy（国内版） | 选择国内版活动导出，与国际版分开识别 |
| WorkBuddy International | 选择国际版活动导出 |
| Trae Agent / TRAE CLI | 选择 executor trace；IDE、开源 Agent、CLI 协议可能不同 |
| Aider | 选择 chat-history Markdown 或结构化导出 |
| ZCode | 选择 ZCode 客户端轨迹，不要把其他客户端误选为 ZCode |
| Qoder CN / Qoder CLI | 选择 CLI stream-json 或渠道导出，CN/国际版分开验证 |
| CodeArts Agent CLI | 选择结构化活动导出，并记录区域与执行主机 |
| MiniMax Code CLI | 选择独立 CLI 轨迹目录；使用 MiniMax 模型不等于 MiniMax CLI |
| GitHub Copilot CLI | 选择 standalone Copilot 会话事件 |
| Cursor CLI | 选择 Cursor agent stream-json，不要选编辑器启动器 |
| DeepSeek Harness | 选择规范 Harness JSONL 或结构化工具轨迹 |
| Others | 使用通用 Harness/OpenAI/Anthropic 工具消息与文本，未知 JSON 只计数不冒充兼容 |

## 时间线、详情与查询

- **全部 / 断言 / 文件 / 网络**筛选真实记录；搜索会与当前筛选叠加。
- **来源**区分 Agent 报告、提取意图和系统证据。此监听流没有 ETW 连接时，履约判定保持未观测。
- 点击记录或按 Enter/Space 查看 JSON 详情，按 Escape 关闭；导出保存当前 JSON 视图。
- **Graph** 只显示意图和真实证据边；**Query** 使用完整路径请求 daemon 的五段式 `why` 链。
- 停止监听会释放 reader 但保留记录；退出 daemon 会清空内存视图，因此请尽早导出，JSON 不是持久化 `.cllens` 会话。

## Others：通用输入

在选择的目录创建 `trace.jsonl`，每行一个对象：

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

这只验证轨迹接入，不证明真实文件或网络 I/O。JSONL 等待完整行；普通文本和 Markdown 仅作为文本展示。

## 排查与上限

目录无权限时重新选择可读目录；没有记录时导出日志或明确勾选 profile 日志。检查健康度中的未知 schema、轮转、截断和跳过字节。云端、WSL、容器和远程网关只能通过导出的报告显示。

扫描上限为深度 6、8192 个条目、128 个文件、单文件 256 KiB、队列 4096、视图 5000 行；超限和跳过字节会在健康度中显示。

[厂商轨迹准备](runtime-integration_CN.md) · [更新日志](../release/CHANGELOG_CN.md)
