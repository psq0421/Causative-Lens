# 17 家 Agent / CLI 接入 CausativeLens

v0.2.5 提供原生日志归一、目录监听和 GUI 选择向导。请先阅读[第一次监听](monitoring-start_CN.md)，再按本页准备厂商轨迹。

语言切换：[简体中文](runtime-integration_CN.md) · [English (US)](runtime-integration.md)

返回：[文档目录](../README_CN.md) · [快速入门](../快速入门.md)

> Agent 告诉我们它打算做什么，操作系统留下它实际做了什么。两条时间线相遇，证据才开始发光。

## 先说清楚当前边界

v0.2.5 的 GUI 目录监听主要验证 Agent 轨迹接入。它不会凭一个日志文件声称完成真实系统监控；没有可用 ETW 证据时，履约判定保持未观测。v1.0.0 之前都是测试版。

| 层 | 当前事实 |
|---|---|
| 模板目录 | 17 家产品，WorkBuddy 国内版/国际版分开，另有 DeepSeek 与 Others |
| 协议 | 统一 JSONL/内容识别；未知格式保留为健康度问题，不强制版本白名单 |
| 厂商导出 | 可以先导出原始日志，再由用户选择目录监听；这不等于原生厂商适配器 |
| `run` / `attach` | 仍可能返回 `offline`，不能当作完整 Agent 启动或挂载证明 |
| ETW / `.cllens` | 真实内核证据与完整持久化仍是独立验收门槛 |

## 通用准备

在标准用户 PowerShell 中记录实际可执行文件、版本、会话 ID、运行主机和执行环境。Native Windows、WSL、Docker、SSH、云端任务必须分开标注；浏览器 PID 不是云 Agent 的进程根。

```powershell
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$runDir = Join-Path $env:LOCALAPPDATA "causativelens\integration\$stamp"
New-Item -ItemType Directory -Path $runDir -Force | Out-Null
Get-Command clens.exe | Select-Object Source
clens --version
clens doctor --json
```

保存原始输出时使用 UTF-8、单独保存 stderr，并记录退出码。不要把 token、提示词、用户名、源代码和工具结果直接上传。

## 17 家产品的推荐输入

| 产品 | 推荐输入与限制 |
|---|---|
| Hermes Agent | 导出 session 或 observer 日志；只有 `state.db` 时先导出。CLI、TUI、Desktop、gateway、WSL 要分开记录。 |
| Claude Code | 使用 message/tool_use、CLI stream-json、transcript 或 hooks；IDE、Desktop、MCP 和远端工具单独验证。 |
| OpenAI Codex | 使用 `codex exec --json` 与 session trace；Desktop/IDE、后台和远程执行单独验证，不声称读取隐藏推理。 |
| Google Antigravity CLI | 只使用当前版本支持的结构化导出；不要把 Gemini CLI 参数套过来。 |
| OpenClaw | 选择 gateway session transcript；gateway 健康日志不等于工具调用，远端操作不是本机 ETW。 |
| Manus | 使用任务活动导出或账户实际提供的 API/webhook；最终报告不是逐步工具轨迹。 |
| AutoGPT | 区分 Classic、Platform 和容器 worker；日志文本不自动等于工具事件。 |
| CodeBuddy | 使用 stream-json 或 session export，CLI 与 IDE 分开确认。 |
| WorkBuddy（国内版） | 选择国内版任务活动导出，与国际版单独做 schema fixture。 |
| WorkBuddy International | 选择国际版任务活动导出，不要混入国内会话。 |
| Trae Agent / TRAE CLI | 使用 executor trace；IDE、开源 Agent 和 CLI 协议可能不同。 |
| Aider | 使用 chat-history Markdown 或结构化历史；“准备修改”不等于真实写文件。 |
| ZCode | 选择实际 ZCode 客户端轨迹；使用智谱模型的其他客户端仍选原客户端模板。 |
| Qoder CN / Qoder CLI | 使用 CLI stream-json 或渠道导出，CN 与国际版分开验证。 |
| CodeArts Agent CLI | 使用结构化活动导出并记录华为云区域和执行主机。 |
| MiniMax Code CLI | 先确认是否是独立 CLI；Claude/Codex 使用 MiniMax 模型时仍选择 Claude/Codex 模板。 |
| GitHub Copilot CLI | 使用 standalone Copilot 会话事件；legacy `gh copilot`、IDE 和云端 coding agent 分开验证。 |
| Cursor CLI | 使用 Cursor `agent` 的 stream-json，确认 executable 来源，不要选择编辑器启动器。 |
| DeepSeek Harness | 使用规范 Harness JSONL 或结构化工具轨迹。 |
| Others | 使用通用 Harness、OpenAI/Anthropic tool message 或文本；未知 JSON 会计数并提示，不冒充兼容。 |

兼容性只记录和提示，不强制阻断。`recent_window_months=3` 是测试目标，不是最近三个月所有版本都通过认证的声明。

## 规范化为 HarnessEvent

保留原始文件，再把真实工具请求转成一行一个标准事件：`schema_v`、UTC 毫秒时间、稳定 step、`reasoning/tool_call/tool_result/system/unknown` phase、实际 tool 参数、原文和来源行号。工具结果不重复生成 tool_call；文本中的“我会写入 X”不是系统写入证据。

```json
{"schema_v":1,"ts_utc_ms":1792886400000,"step":1,"phase":"tool_call","tool":{"name":"read","arguments":{"path":"C:\\demo\\config.yaml"}},"text":"SOURCE_RECORD","harness_log_id":20261101,"line_no":1}
```

把原始、规范化、拒绝行、映射版本和 metadata 放在同一目录。未知 phase 进入 `unknown` 并保留 raw；不能用重命名文件代替转换。

## 手工 RPC 验证

仓库提供 `docs/guide/examples/harness-rpc.ps1`。它可以打开 observe 会话、提交一条明确标记的合成 HarnessEvent、查询 `why` 并关闭会话；该流程不启动 Agent、不请求 UAC、不产生真实文件证据。预期是 assertion 收到、`events_seen=0`、evidence 为空，这个“空”是诚实的边界。

## 验收标准

真正的接入必须同时具备：版本与来源记录、可重放原始样本、稳定转换规则、工具调用/结果关联、真实系统事件、断言与事件对齐、GUI/CLI 共享会话，以及保存后可重开。只看到最终答案、gateway 启动日志或云端任务完成通知，都不足以证明因果履约。

遇到 `offline`、空证据、未知 schema 或云端无本机事件时，请保留健康度提示并记录执行位置。望远镜不会把尚未抵达的星光画进镜片。
