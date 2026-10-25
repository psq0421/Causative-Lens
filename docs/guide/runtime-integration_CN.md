# 17 款 Agent / CLI 接入 CausativeLens：详细教程

语言：[简体中文](runtime-integration_CN.md) · [English (US)](runtime-integration.md)

返回：[文档目录](../README_CN.md) · [快速入门](../快速入门.md)

> 程序说“我做了什么”，系统留下“实际发生了什么”。只有两份时间线真正相遇，我们才把它称为证据。

适用：Windows x64、当前仓库及 v0.1.0 Beta。核查日期：**2026-10-25**。v1.0.0 之前均为测试版。厂商命令须先与本机 `--help` 核对；示例任务由你按需执行，可能消耗厂商额度。

## 先看结论：现在到底能接到哪一步

**当前安装包尚不能完成这 17 款软件的“一键实时系统行为监测”。** 本次回读了 CLI、daemon、capture、Harness 源码，并用随包 daemon 验证了 Named Pipe RPC。此前文档将规划中的接线写成了已经完成的能力，完成度表述过高；此处按可验证行为纠正。

| 层次 | 当前状态 | 能证明什么 |
|---|---|---|
| 运行时名称登记 | 17 项均登记；WorkBuddy 拆国内/国际，加 DeepSeek 共 19 项 | 名称可识别，不代表有原生适配器 |
| 运行时协议 | 全部仍是 `jsonl-tier3`；library doctor 标记 `tier3-jsonl-unverified` | 不能宣称近三个月所有版本已实测 |
| 厂商原始日志获取 | 部分厂商提供流式输出、hook 或会话导出，本文分项说明 | 拿到原始轨迹，不等于 CausativeLens 已接收 |
| 原始日志 → HarnessEvent | 通用解析/tail 有库级实现；厂商转换和自动启动接线未完成 | 厂商 JSONL 不能直接当作 Harness JSONL |
| 手工 HarnessEvent → RPC → 断言 → why | **本机验证通过**，第 6 节可复现 | 仅证明推理侧手工协议链路 |
| `clens run` / `clens attach` | 当前返回 `status: offline` | 未启动完整监测，也未因该命令启动 Agent |
| ETW → spool → daemon 常驻链 | 尚未形成可用的生产闭环 | 手工上报系统事件不能冒充真实采集 |
| 持久化、GUI 实时订阅 | 不能据此认定已打通 | 当前 RPC 会话在内存中，不能当作已保存的 `.cllens` |

`clens doctor` 的 `status: ok` 有离线回退，必须检查是否实际有 `daemon: connected`。即使连接成功，也只证明 daemon 可达。当前没有可用的 `clens adapters doctor` CLI 子命令。

本文区分三种完成状态：

- **原始轨迹准备完成**：厂商日志、版本和执行环境已保存。
- **推理侧接入验证完成**：规范化事件已送入指定 RPC 会话，查到对应断言。
- **完整监测完成**：真实系统事件持续进入，同步对齐且会话可保存/重开；当前不能宣称达到这一状态。

## 目录

- [1. 共同准备](#prepare)
- [2. 17 款产品路线总表](#routes)
- [3. 五个重点产品](#priority)
- [4. 其余 12 项](#others)
- [5. 转换为 HarnessEvent](#normalize)
- [6. 可验证的 RPC 接入全过程](#rpc)
- [7. 验收标准](#acceptance)
- [8. 常见问题](#troubleshooting)
- [9. 后续接线顺序](#next)
- [10. 官方资料与核查依据](#sources)

<a id="prepare"></a>
## 1. 共同准备

### 1.1 先确定 Agent 实际在哪里执行

| 执行位置 | Windows 本机的观测范围 | 不能据此推出的结论 |
|---|---|---|
| Windows 原生 CLI / 本地 worker | 完成采集接线后，可观测 Windows 进程树 | 有工具调用就一定发生了对应 IO |
| WSL2 Linux 内部 | 部分宿主/边界活动 | Linux 进程和文件事件已逐项被 ETW 捕获 |
| Docker Linux 容器 | 宿主边界活动及主动导出的日志 | 容器内调用就是 Windows 文件事件 |
| SSH / remote gateway / 云 Agent | 本地客户端与网络边界；远端导出轨迹 | 云端改文件等于本机改文件 |
| 浏览器中的 Manus / Web Agent | 本地浏览器活动 | 浏览器 PID 就是云端 Agent 根进程 |

**观察位置跟着执行器走。** MCP、SSH、远程工具的执行主机须单独记录。本地客户端不保证工具在本地运行。本文没有交付 WSL/Linux/macOS 采集器。

### 1.2 找到程序，不猜安装目录

普通用户 PowerShell 中执行：

~~~powershell
$ErrorActionPreference = 'Stop'
$clensExe = (Get-Command clens.exe -ErrorAction Stop).Source
$installDir = Split-Path -Parent $clensExe
$daemonExe = Join-Path $installDir 'clens-daemon.exe'
& $clensExe --version
& $clensExe doctor --json
Test-Path -LiteralPath $daemonExe
~~~

PATH 未刷新时重新打开终端。不要因为 `run/attach` 离线就把 GUI/daemon 全部改为管理员运行；这不能补齐接线。未来也只有 capture 应提权。

### 1.3 每次采样使用独立目录和 UTF-8

下方 `$runDir` 在后续例子中复用：

~~~powershell
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$suffix = [Guid]::NewGuid().ToString('N').Substring(0, 8)
$runDir = Join-Path $env:LOCALAPPDATA "causativelens\integration\$stamp-$suffix"
New-Item -ItemType Directory -Path $runDir -Force | Out-Null
$utf8NoBom = [Text.UTF8Encoding]::new($false)
~~~

推荐 PowerShell 7。Windows PowerShell 5.1 的 `>` 可能生成 UTF-16；不要直接用它写要求为 UTF-8 的 JSONL。下面函数逐行写 UTF-8 并 flush，**只保存原始输出，不是适配器或后台 tail**：

~~~powershell
function Save-Utf8Lines {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [AllowEmptyString()][string]$Line,
        [Parameter(Mandatory = $true)][string]$Path
    )
    begin {
        $encoding = [Text.UTF8Encoding]::new($false)
        $writer = [IO.StreamWriter]::new($Path, $false, $encoding)
    }
    process {
        $writer.WriteLine($Line)
        $writer.Flush()
    }
    end { $writer.Dispose() }
}
~~~

此函数适用于短采样；中断后检查末行完整性。标准错误单独存 `stderr.log`，不要 `2>&1` 混进 JSONL。每次厂商命令结束后、运行其他原生命令之前保存退出码：

~~~powershell
$vendorExitCode = $LASTEXITCODE
[IO.File]::WriteAllText((Join-Path $runDir 'exit-code.txt'), [string]$vendorExitCode, $utf8NoBom)
~~~

非零先看厂商错误；文件非空不代表采样成功。banner/ANSI/进度条也不是合法 JSONL。

### 1.4 保存版本与来源

每个产品/渠道各建一份，填入真实版本，不要只写“最新版”：

~~~powershell
$metadata = [ordered]@{
    runtime = 'claude-code'
    reported_version = 'REPLACE_WITH_ACTUAL_VERSION'
    sampled_at_utc = [DateTime]::UtcNow.ToString('o')
    execution_environment = 'windows-native'
    execution_host = $env:COMPUTERNAME
    working_directory = (Get-Location).Path
    source_kind = 'vendor-stdout'
    compatibility = 'unverified'
}
[IO.File]::WriteAllText((Join-Path $runDir 'metadata.json'), ($metadata | ConvertTo-Json -Depth 8), $utf8NoBom)
~~~

另记 CLI/IDE/Desktop/SDK、国内/国际站、profile、会话 ID。共享前脱敏 token、提示词、路径用户名、源码和工具结果；系统事件只收元数据，不代表推理轨迹已自动脱敏。

<a id="routes"></a>
## 2. 17 款产品路线总表

“有入口”指厂商提供获取方式；**没有任何一行代表已完成 CausativeLens 原生实时适配**。

| 编号 | 产品 | 推荐原始输入 | 建议 |
|---|---|---|---|
| 01 | **Hermes Agent** | sessions export；Observer Hooks | 先导出，再接只读 observer |
| 02 | **Claude Code / 相关产物** | CLI stream-json；hooks/transcript | 优先 CLI，其他形态独立验证 |
| 03 | **OpenAI Codex** | `codex exec --json` | exec 与 Desktop 单独验证 |
| 04 | Antigravity CLI | 发行版支持的导出/hook | 先确认产品身份 |
| 05 | **OpenClaw** | gateway JSON logs + session transcript | 两类日志分开 |
| 06 | **Manus** | 任务导出/API/webhook（账户支持时） | 标明云端可见性 |
| 07 | AutoGPT | 执行器事件 / 平台 task 日志 | Classic、Platform、容器分开 |
| 08 | CodeBuddy | CLI stream-json；IDE 独立导出 | 独立转换字段 |
| 09 | WorkBuddy 国内/国际 | 各自任务日志/导出 | 两套版本 fixture |
| 10 | Trae Agent / TRAE CLI | 帮助确认的流/导出；IDE 日志 | IDE、CLI、开源 Agent 分开 |
| 11 | Aider | chat history + llm history | 文本，非 Harness JSONL |
| 12 | ZCode | 发行版工具轨迹 | 先确认客户端身份 |
| 13 | Qoder CN / CLI | CLI stream-json；IDE 独立导出 | 核对实际命令和渠道 |
| 14 | CodeArts Agent CLI | 发行渠道的事件日志 | 记录区域及构建来源 |
| 15 | MiniMax Code CLI | 实际客户端输出 / 独立 CLI 日志 | 模型供应商不是运行时身份 |
| 16 | GitHub Copilot CLI | 官方 logging/monitoring、会话导出 | standalone、旧 gh、IDE 分开 |
| 17 | Cursor CLI | `agent` 的 stream-json | 确认 agent 确属 Cursor |

兼容策略是记录、提示、降级，不按版本强制阻断。`recent_window_months=3` 是测试目标，不是近三个月全部版本已通过的证书。

<a id="priority"></a>
## 3. 五个重点产品

### 01. Hermes Agent：优先级最高

**A. 确认形态和版本。** CLI/TUI/Desktop 可能由 gateway 执行工具，terminal PID 不一定是 Agent 根 PID。[H1][H2]

~~~powershell
Get-Command hermes -ErrorAction Stop | Select-Object Name, Source
hermes --version
hermes chat --help
hermes sessions --help
hermes sessions export --help
~~~

仅在 WSL 安装时，在 WSL 执行厂商命令并记录环境为 `wsl2`。把日志导到 Windows 不等于取得 Linux 系统事件。

**B. 产生小会话。** 在含 README.md 的测试目录运行，沿用 Hermes 自身权限和模型登录：

~~~powershell
hermes chat -q "Read README.md and summarize it. Do not edit files."
hermes sessions list
~~~

记录真实 session ID；gateway 多任务时不要笼统选择“最新会话”。

**C. 导出指定会话。** 先与本机帮助核对：

~~~powershell
$hermesSessionId = Read-Host 'Hermes session ID'
$hermesExport = Join-Path $runDir 'hermes-session-export.jsonl'
hermes sessions export $hermesExport --session-id $hermesSessionId --redact
~~~

默认 JSONL 导出是**每行一个会话对象，内部包含 messages**，不是每行一个 HarnessEvent。必须展开 session → messages → tool calls。快照适合离线样本；反复覆盖导出文件再 tail 不是可靠实时流。

**D. 实时轨迹选官方 Observer Hooks。** 文档提供 `pre_tool_call` / `post_tool_call`，telemetry schema 为 `hermes.observer.v1`。[H3]

| 信息 | 转换用途 |
|---|---|
| session_id / task_id | 会话与任务分流 |
| turn_id / tool_call_id / api_request_id | 调用关联、去重、子任务归属 |
| tool_name / args | 规范工具名与参数 |
| post_tool_call 的 result/status/error/duration_ms | 单独结果事件与健康信息 |
| 厂商时间戳 / 接收时间 | 分开记录，不无声互换 |

只读 observer 应有界排队、不阻塞 Agent，失败 fail-open；不返回修改参数、批准、拒绝或中止工具的决策。**当前包没有 Hermes→CausativeLens observer 插件及自动安装命令。** 这是待接线接口，不能假装已能复制运行。

**现阶段完成条件：** 导出指定会话，取得带关联 ID 的 call/result 样本；按第 5 节规范化、第 6 节验证。gateway/WSL 的实际执行器要单独标明。

### 02. Claude Code 及相关产物

**A. 核对程序。**

~~~powershell
Get-Command claude -ErrorAction Stop | Select-Object Name, Source
claude --version
claude --help
~~~

**B. 采集官方非交互消息流。** 沿用现有权限设置，不添加跳过权限选项。[C1]

~~~powershell
$raw = Join-Path $runDir 'claude.raw.jsonl'
$err = Join-Path $runDir 'claude.stderr.log'
claude -p "Read README.md and summarize it. Do not edit files." --output-format stream-json --verbose 2> $err | Save-Utf8Lines -Path $raw
~~~

访问被拒绝时保留拒绝结果，通过 Claude 自身流程授权测试目录，或取交互会话样本。不要为了监测关闭权限保护。

**C. 转换真实消息，而非最终回答。**

| 来源 | 规范化方向 |
|---|---|
| assistant text block | 消息文本；不代表完整内部思维链可见 |
| assistant.message.content[] 的 tool_use | tool_call，保留 block ID |
| Read + input.file_path | read + arguments.path |
| Edit + input.file_path | edit + arguments.path |
| Write + input.file_path | write；不能一律认定为首次 create |
| Bash + input.command | shell + arguments.cmd |
| tool_result，含失败/取消 | tool_result；本 Beta RPC 中 tool=null，见第 5 节 |
| parent_tool_use_id | 子 Agent 关联，不能全部压成主会话工具 |

这是转换规则，需用对应版本 fixture 核对，不是已打包 converter。只有最终 result 时不能反推出文件动作。

**D. CLI、IDE、Desktop、SDK 独立验证。** hooks 中的 `transcript_path`、`tool_name`、`tool_input` 可用于定位关联。[C2] 按对应文档使用只读 exporter，勿覆盖用户 hooks。Desktop MCP、远端工具和 Code 本地进程不是同一观测面。

**完成条件：** 至少一组可关联的工具调用/结果，转换后 RPC 能提取断言；实际 IO 仍需系统采集验证。

### 03. OpenAI Codex

本项按 **OpenAI Docs** 流程核查。官方页面本次网络返回 `403`，选项另以本机 CLI 帮助核实：`codex-cli 0.164.0-alpha.11.1`。仅查询版本/帮助，**未代发模型任务**。[O1]

~~~powershell
Get-Command codex -ErrorAction Stop | Select-Object Name, Source
codex --version
codex exec --help
~~~

PATH 找不到但装有 Desktop 时，从实际安装位置定位 CLI，勿硬编码版本化目录。

~~~powershell
$raw = Join-Path $runDir 'codex.raw.jsonl'
$err = Join-Path $runDir 'codex.stderr.log'
codex exec --json --sandbox read-only "Read README.md and summarize it. Do not edit files." 2> $err | Save-Utf8Lines -Path $raw
~~~

`--json` 提供事件流；`--output-schema` 约束最终答案，不能代替轨迹。不要为接日志绕过 sandbox/审批。

转换要求：

- thread/turn/item ID 保留，消息和工具执行分开。
- command execution 命令可映射 shell；开始/完成/失败关联，不能每个状态各重复创建一条意图。
- 补丁/文件变化摘要是客户端报告，不能直接变成已观测的系统 FileWrite。
- 工具输出中的“read X”不是新 read 调用。
- Desktop/IDE、后台/远程运行与 exec 的位置、存储可能不同，分别验证。
- 若靠会话存储读取，避免 `--ephemeral` 禁用会话持久化。

**完成条件：** 保存版本和原始流，执行状态可关联并按第 5 节转换。没有公开 reasoning 内容时，不声称取得隐藏思维链。

### 05. OpenClaw

**A. 确认 gateway。**

~~~powershell
Get-Command openclaw -ErrorAction Stop | Select-Object Name, Source
openclaw --version
openclaw logs --help
~~~

记录 profile、gateway 主机、工具执行位置；token 沿用凭据机制，不写入示例或仓库。

**B. 保存 gateway JSON 流。** 官方支持：[W1]

~~~powershell
$raw = Join-Path $runDir 'openclaw.gateway.raw.jsonl'
$err = Join-Path $runDir 'openclaw.stderr.log'
openclaw logs --follow --json 2> $err | Save-Utf8Lines -Path $raw
~~~

另开终端做测试任务，结束后 Ctrl+C 并检查末行。重连/切换日志时按游标或稳定 ID 去重。

**C. 另取 session transcript。** 按实际部署和官方 session 文档定位指定会话。[W2] gateway logs 可能只有生命周期/notice/错误，不包含完整工具轨迹。不存在本文已验证的全版本通用 transcript 路径。

**D. 两路关联。** session/task/tool-call ID 串联，notice 记 system，真实工具调用才记 tool_call；网关健康不等于工具成功。Linux/Docker/远端 gateway 的内部操作不是 Windows 本地证据。

**完成条件：** 取得实际会话工具轨迹，而非只有 gateway started；版本、profile、主机和游标可追溯。

### 06. Manus

**本文没有验证 `manus.exe --json` 这样的命令。** 官方开放平台有 Task/Webhook 等入口；本次核实了文档入口，未核实具体账户接口可用性、参数或完整工具 schema。[M1]

1. 记任务 ID、客户端版本、执行位置，区分云任务和本机连接器。
2. 当前产品有任务/活动导出时，导出指定任务并保留步骤/时间/工具 ID。只有最终报告时，不能伪造逐步事件。
3. 账户开放 API/Webhook 时，依当时官方文档接收你自己的任务事件，做签名验证、去重、接收时间记录。本文不编造 endpoint，也不启用公网服务。
4. 只有 task completed 就记 system 状态；有结构化工具记录才映射 tool_call。
5. 本机连接器真正执行工具时需定位 worker；浏览器不是云沙箱的根进程。

**完成条件：** 活动样本可归属任务并明确可见性。缺工具明细时只能准备有限轨迹，不能宣称完整因果监测。

<a id="others"></a>
## 4. 其余 12 项

这些产品同样需“原始样本 → 第 5 节规范化 → 第 6 节 RPC”。未核实参数不杜撰。

### 04. Google Antigravity CLI

1. 确认 Antigravity IDE、独立 CLI 或非官方 wrapper，记发行者/版本/程序路径。
2. 查实际程序 `--help` 的日志/export/headless/hooks；**不套 Gemini CLI 参数**。
3. 有结构化流时取只读 call/result；只有 IDE 活动时先导出离线样本。
4. 标记 local/remote/sandbox worker，IDE PID 不一定执行工具。

本次未建立其 CLI 参数和 schema 契约，故无虚构一键命令。完成条件：可重放的真实工具样本；截图不等于可机械对齐的事件。

### 07. AutoGPT

1. 区分 Classic、Platform、云任务，记录包/仓库版本、worker、task ID。
2. 可控执行器从已有 callback/event 导出工具/参数/时间/结果 ID；当前包没有此 exporter。
3. 容器先取诊断日志，使用自己的实际容器名：

~~~powershell
$containerName = Read-Host 'Actual AutoGPT worker container name'
docker logs --follow --timestamps $containerName 2> (Join-Path $runDir 'autogpt.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'autogpt.container.log')
~~~

4. docker logs 是文本，不保证工具 schema；Windows ETW 不由此获得 Linux 容器内部系统调用。

完成条件：task/worker/tool 可关联；仅 Web 管理日志不足。

### 08. CodeBuddy（腾讯云）

官方已确认 `codebuddy`、`-p` 和 `--output-format stream-json`，不是照搬 Claude 参数。[B1][B2]

~~~powershell
Get-Command codebuddy -ErrorAction Stop | Select-Object Name, Source
codebuddy --version
codebuddy --help
$raw = Join-Path $runDir 'codebuddy.raw.jsonl'
$err = Join-Path $runDir 'codebuddy.stderr.log'
codebuddy -p "Reply with the word READY. Do not use tools." --output-format stream-json 2> $err | Save-Utf8Lines -Path $raw
~~~

此例只验证流，**没有文件证据**。官方说明非交互需授权工具可能被阻止；不通过跳过权限解决。工具样本用已授权交互任务、transcript 或该版本只读 hooks。

记录国内/国际/企业认证入口，IDE/CLI 分开。后台工具终态可能晚于当前轮 result，不能见首个 result 就关全会话。协议形状相似也要独立 fixture。

完成条件：实际工具调用及结果可关联，不止 READY 文本。

### 09. WorkBuddy 国内版 / 国际版

1. 分 `workbuddy-cn` / `workbuddy-international` 目录，记完整构建号、渠道、区域、执行位置。
2. 各建只读任务，从当前版支持的活动/任务/诊断导出获取记录；本次未确认统一 JSONL 参数和路径。
3. 即使使用 CodeBuddy 执行器，也须用 worker PID、session/tool ID 证明，不能因同属腾讯就复用 schema。
4. 桌面/远端/移动/小程序分别标注；只有最终报告只能作报告。

仓库别名不等于有同名 CLI。完成条件：两版各至少一份真实 call/result fixture。

### 10. Trae Agent / TRAE CLI

1. 区分 IDE、独立 CLI、开源 Agent，保留安装/仓库来源及版本。
2. CLI 按实际帮助取流，IDE 取支持的日志，开源 Agent 从执行器现有事件出口加只读 exporter。
3. 计划文本、工具请求、结果、子任务 ID 分开；计划文件不是已执行证据。
4. 后台/远程 worker 标明主机和 workspace。

本次没有可覆盖这三种形态的已验证统一命令。完成条件：同一任务 call/result 可关联，环境明确。

### 11. Aider

官方参数支持聊天/LLM 历史；输出是 Markdown/文本，不是 Harness JSONL。[A1]

~~~powershell
aider --version
aider --help
aider --message "Read README.md and summarize it. Do not edit files." --dry-run --chat-history-file (Join-Path $runDir 'aider.chat.md') --llm-history-file (Join-Path $runDir 'aider.llm.log')
~~~

`--dry-run` 用于首次避免写入。日志可包含源码/提示词，共享前脱敏。对话、拟议 diff、应用操作分开；“我将修改 X”不等于写入。

无结构化工具时保留文本；本 Beta 没有已接通的小模型后端自动补齐。完成条件：来源/版本/目录/可信度明确，不能把 diff proposal 填成系统 FileWrite。

### 12. ZCode（智谱）

1. 确认真正的客户端/CLI；Claude/Codex 改用智谱模型仍按原客户端适配。
2. 按该版帮助找 export/log/hooks；本次未核实固定参数/目录。
3. 取 read 与结果，按真实字段转 read/path。
4. API 供应商、客户端版本、执行主机分开记录。

完成条件：客户端身份和工具 schema 有真实样本。

### 13. Qoder CN / CLI

官方当前脚本文档展示 `qoder` 与 `--output-format stream-json`；旧安装可能用不同入口，核对来源/帮助。[Q1]

~~~powershell
Get-Command qoder,qodercli -ErrorAction SilentlyContinue | Select-Object Name, Source
# Use qoder below only if it is your verified executable.
qoder --version
qoder --help
qoder -p "Reply with the word READY. Do not use tools." --output-format stream-json 2> (Join-Path $runDir 'qoder.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'qoder.raw.jsonl')
~~~

首例只验流。真实工具样本使用正常权限流程或已授权只读任务；headless 可能拒绝待确认工具，不为采样开启绕过权限模式。

CN/国际、IDE/CLI/Cloud Agent 分开。最终单个 json 对象不是持续事件流；CN 不因国际文档存在选项就自动认证。完成条件：对应渠道真实工具 fixture。

### 14. CodeArts Agent CLI（华为）

1. 记准确产品/渠道/华为云区域/版本，避免与其他 CodeArts 插件或仅模型接口混淆。
2. 查实际 CLI 帮助，仅用明确支持的结构化输出/诊断导出。
3. 取只读任务的参数/结果/时间/session；无工具结构先保存原文。
4. 云 workspace/容器标 remote，不能归本机 CLI PID。

本次未核实所有发行渠道统一参数，故不虚构。完成条件：版本化真实协议样本。

### 15. MiniMax Code CLI

1. 确认独立 CLI，还是 Coding Plan/API 接到已有客户端。
2. Claude/Codex/Aider 改用 MiniMax 时按**客户端章节**；另记 model_provider，不创造虚假进程族。
3. 真有独立 CLI 时记官方来源/程序/帮助，再选其真实事件流或导出。
4. 确认工具在本机、云端还是 SDK worker。

完成条件：实际客户端明确且工具轨迹可解析；模型请求日志不能代替系统行为。

### 16. GitHub Copilot CLI

官方确认 standalone `copilot` 与 logging/monitoring 帮助。[G1][G2]

~~~powershell
Get-Command copilot -ErrorAction Stop | Select-Object Name, Source
copilot --help
copilot help logging
copilot help monitoring
~~~

版本按本机帮助记录；官方还列出 `copilot version`，可能检查更新，离线取证优先用帮助支持的版本选项。

1. 按 logging/monitoring 帮助取日志/导出，不臆造 `--json` 事件参数。
2. 已授权只读任务中取 session/tool ID 和结果；最终聊天文本与工具活动分开。
3. 旧 `gh copilot`、IDE、云 coding agent 独立验证。
4. /delegate 云执行记 remote。

完成条件：standalone 真实工具记录可追溯。`config --list --json` 配置 JSON 不是轨迹。

### 17. Cursor CLI

官方当前入口为 `agent`，旧版可能为 `cursor-agent`。agent 名称通用，先查 Source；cursor 常是编辑器启动器。[U1]

~~~powershell
Get-Command agent,cursor-agent -ErrorAction SilentlyContinue | Select-Object Name, Source
# Continue only after confirming agent is Cursor's executable.
agent --version
agent --help
agent -p --output-format stream-json "Read README.md and summarize it. Do not edit files." 2> (Join-Path $runDir 'cursor.stderr.log') | Save-Utf8Lines -Path (Join-Path $runDir 'cursor.raw.jsonl')
~~~

不加 `--force`。拆 assistant/call/result 并保留 ID。最终答案、拟议补丁与实际写入分开。IDE、Background/Cloud Agent 独立核对执行位置。

完成条件：程序身份正确且有可关联工具样本；仅文本/权限拒绝不算工具轨迹采集完成。

<a id="normalize"></a>
## 5. 转换为 HarnessEvent

### 5.1 保留两份数据

~~~text
<runDir>\
  metadata.json
  <vendor>.raw.jsonl        # Or the original export / text / Markdown
  <vendor>.stderr.log
  normalized.harness.jsonl
  rejected.jsonl
  mapping-version.txt
~~~

**目前未提供覆盖这 17 家格式的现成 converter。** 先手工转换一条真实记录验证协议；不能只改后缀、把原始厂商 JSONL 直接发进 ingest_harness。

### 5.2 最小有效记录

下面是**协议示例，不是真实 Agent 轨迹**：

~~~json
{"schema_v":1,"ts_utc_ms":1792886400000,"step":1,"phase":"tool_call","tool":{"name":"read","arguments":{"path":"C:\\demo\\config.yaml"}},"text":"DOCS_EXAMPLE_NOT_AGENT_OUTPUT","harness_log_id":20261025,"line_no":1}
~~~

| 字段 | 要求 |
|---|---|
| schema_v | 整数 1 |
| ts_utc_ms | Unix UTC 毫秒；无发生时间时用接收时间须在旁路记录 estimated，不冒充发生时间 |
| step | 转换规则下稳定步骤号 |
| phase | reasoning / tool_call / tool_result / system / unknown |
| tool | 明确调用才填 {name,arguments}，其余 null |
| text | 来源原文，不补写动作；人工示例标记 |
| harness_log_id / line_no | 当前稳定 ID 来源，同一规范化日志保留稳定值 |
| byte_start / byte_end | 可选字节区间；默认 0 不代表精确原文定位 |
| raw_json | 可选**字符串**，不是嵌套对象 |

不能用 mtime 精确反推事件时间。多主机记录时钟来源、时差和不确定度，不调整时间来“凑匹配”。

当前 ID 取 `(harness_log_id,max(line_no,step),slot)`。规范化时让 `line_no` 稳定且不小于 `step`，避免多个记录被同一较大 step 折叠成相同 ID。原始行含多个 tool block 时，拆成独立稳定编号的规范化记录，以旁路映射保留原始行/块 ID；不能所有工具共享同一来源键。库内 slot 只区分一条规范化工具事件产生的多个断言。

### 5.3 规则能力与边界

| tool.name | 参数 | 当前提取 |
|---|---|---|
| read | path 或 target | Read |
| write | path 或 target | Write |
| edit | path 或 target | Read 前置 + Write 顺序关系 |
| create | path 或 target | Create |
| delete | path 或 target | Delete |
| shell / exec | cmd 或 command | 少量 rm/mv/cp/curl/ssh 模板 |
| connect / bind / listen | 目标字段 | 名称被识别，端点规范化未完整接通 |
| 其他 | 保留原文 | 可能无断言，不等于行为已完成 |

shell 模板不是完整 PowerShell/bash parser；复杂引号、管道、别名、重定向、空格路径需独立验证，不改写命令伪造兼容。

**本 Beta 注意事项：**

- ingest_harness 直接反序列化 HarnessEvent，绕过通用 parser 的 quarantine、未知字段保留及哨兵。未知 phase 在此不会自动变 unknown，由转换端处理。
- 提取器未充分按 phase 隔离。tool_result 再填相同 tool 可能重复提取；目前结果事件用 `tool=null`，关联放原文/旁路。
- 文本“已写入”不是系统证据；`extractor-llm` 尚无可用的内置 ONNX 后端自动兜底。
- 单个 RPC 请求含封装不超过约 64 KiB；超长结果留原始文件及引用，不截断 JSON 或静默吞错。

<a id="rpc"></a>
## 6. 可验证的 RPC 接入全过程

这是**手工推理侧验证**，不会启动 Agent、ETW 或 UAC，不生成真实文件证据。在仓库根执行；安装包用户需取得下面 helper。

### 6.1 普通用户启动 daemon

测试时关闭占用连接的 GUI，不结束别人的进程。当前 daemon 的单客户端连接限制并发；helper 每次调用断开。

~~~powershell
$clensExe = (Get-Command clens.exe -ErrorAction Stop).Source
$daemonExe = Join-Path (Split-Path -Parent $clensExe) 'clens-daemon.exe'
if (-not (Test-Path -LiteralPath $daemonExe)) {
    throw 'clens-daemon.exe is missing next to clens.exe.'
}
if (-not (Get-Process -Name clens-daemon -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath $daemonExe -WindowStyle Hidden | Out-Null
}
~~~

仅验证仓库 payload 时，可将 $clensExe 先设为真实 installer/payload/clens.exe 路径。开发 payload 与安装版证据分别标注。

### 6.2 加载 helper，握手、开会话

脚本：[harness-rpc.ps1](examples/harness-rpc.ps1)。只封装已有协议，带超时/长度/错误检查；不含 tail 或格式转换。

~~~powershell
. .\docs\guide\examples\harness-rpc.ps1
$hello = Invoke-CausativeLensRpc -Method 'hello' -Params @{
    protocol_version = 1
    mode = 'observe'
}
$hello | ConvertTo-Json -Depth 8
$ticket = Invoke-CausativeLensRpc -Method 'session.open' -Params @{}
$sessionId = $ticket.session_id
$sessionId
~~~

期望 product=CausativeLens、mode=observe、protocol_version=1，非空票据。当前请求是单行 `{id,method,params}`，勿添加未实现的订阅方法。

### 6.3 发送人工示例、查询、清理

~~~powershell
$demoPath = 'C:\causativelens-docs-demo\config.yaml'
try {
    $event = [ordered]@{
        schema_v = 1
        ts_utc_ms = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        step = 1
        phase = 'tool_call'
        tool = @{
            name = 'read'
            arguments = @{ path = $demoPath }
        }
        text = 'DOCS_RPC_DEMO_NOT_AGENT_OUTPUT'
        harness_log_id = 20261025
        line_no = 1
    }
    $ingest = Invoke-CausativeLensRpc -Method 'session.ingest_harness' -Params @{
        session_id = $sessionId
        event = $event
    }
    $status = Invoke-CausativeLensRpc -Method 'session.status' -Params @{
        session_id = $sessionId
    }
    $why = Invoke-CausativeLensRpc -Method 'query.why' -Params @{
        session_id = $sessionId
        path = $demoPath
    }
    $ingest | ConvertTo-Json -Depth 10
    $status | ConvertTo-Json -Depth 10
    $why | ConvertTo-Json -Depth 20
} finally {
    Invoke-CausativeLensRpc -Method 'session.close' -Params @{
        session_id = $sessionId
    } | Out-Null
}
~~~

期望 assertion_id 非空、active=true；但 events_seen=0、spool_frames=0，why 为 unobserved、证据为空。**这是正确的诚实结果**：有 read 断言，没有实际读取证据，也没证明文件存在。勿补造 ingest_system 数据让截图变绿。

显式传 session_id 很重要；CLI why 目前不带票据，可能选择其他内存会话；watch/timeline 也不能可靠显示本节会话。

session.close 释放内存，不保存文件；daemon 重启丢失状态。若需停止 daemon，只停止自己启动且确认无人使用的进程。

### 6.4 有限导入已规范化 JSONL

仅接受第 5 节转换后的文件。**离线有限导入，不是厂商实时监测器。**

~~~powershell
$normalizedPath = Join-Path $runDir 'normalized.harness.jsonl'
if (-not (Test-Path -LiteralPath $normalizedPath)) {
    throw 'Normalize vendor records before importing them.'
}
$ticket = Invoke-CausativeLensRpc -Method 'session.open' -Params @{}
$sessionId = $ticket.session_id
$lineNo = 0
try {
    foreach ($line in [IO.File]::ReadLines($normalizedPath, [Text.Encoding]::UTF8)) {
        $lineNo++
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $event = $line | ConvertFrom-Json -ErrorAction Stop
        Invoke-CausativeLensRpc -Method 'session.ingest_harness' -Params @{
            session_id = $sessionId
            event = $event
        } | ConvertTo-Json -Depth 8 -Compress
    }
    Invoke-CausativeLensRpc -Method 'session.status' -Params @{
        session_id = $sessionId
    } | ConvertTo-Json -Depth 10
    # Query with query.why and the same session_id here, before closing.
} catch {
    throw ("Import stopped at normalized line {0}: {1}" -f $lineNo, $_.Exception.Message)
} finally {
    Invoke-CausativeLensRpc -Method 'session.close' -Params @{
        session_id = $sessionId
    } | Out-Null
}
~~~

遇错停下，修正后用新会话重放。RPC 返回值不是已持久化的完整会话文件。

<a id="acceptance"></a>
## 7. 验收标准

| 层 | 测量/证据 | 本次结论 |
|---|---|---|
| 厂商原始输出 | 版本/session/原始记录/官方参数 | 核实部分入口，未代跑 17 家 Agent |
| 编码 | 每行 UTF-8 JSON 可解析，stderr 分离 | 给出方法，样本逐家验证 |
| 转换 | call/result/子任务/未知类型/时间/来源键 golden fixture | 全套版本 fixture 尚缺 |
| RPC | hello/ticket/ingest/status/why/close | **随包 daemon 本机通过** |
| 连续性 | append/轮换/截断/半行/重连去重/坏行计数 | 库有部分能力，自动接线未验收 |
| 系统采集 | 已知本地操作对应真实 seq/pid/path，计数增长、gaps 可解释 | 完整常驻链未验证 |
| 因果闭环 | 断言/真实事件/why 一致，拒绝不会判已履约 | 不能用人工事件代替 |
| 保存与 GUI | 重开内容一致，GUI/CLI 同票据，重启恢复 | 本教程所需闭环未完成 |
| 延迟 | 锚定时钟后，真实发生到可查的 P50/P99 | 未测；目标 P50≤400ms、P99≤1.5s |

本次真实冒烟：observe / 0.1.0、assertion 收到、events_seen=0、why unobserved、证据 0；`synthetic_harness_only=true`、`real_agent_test=false`、`etw_started=false`。开发机记录位于被 git 忽略的 `target/runtime-guide-smoke.json`。

教程定稿时直接执行了第 6.2–6.4 节代码块，并验证 UTF-8 中文往返、损坏行精确报错、64 KiB 请求限制、RPC 错误传播及会话清理，全部通过。证据位于本机 `target/runtime-guide-verification.json`；没有运行厂商付费任务或启动 ETW，也没有将这些检查当作 17 家真实兼容测试。

未来真实验收在专用测试目录做：读已有文件、人工授权写新文件、拒绝一次权限、一个子进程、退出/恢复；逐项对照真实系统元数据。首轮教程用只读样本，不操作实际项目文件。

<a id="troubleshooting"></a>
## 8. 常见问题

| 现象 | 原因与下一步 |
|---|---|
| run/attach offline，退出码 0 | 当前占位，不是网络问题，提权/重装不补齐实现 |
| doctor ok 却无数据 | 检查 daemon connected；ok 有离线回退 |
| 原始 JSONL missing field | 厂商格式非 HarnessEvent，需转换 |
| assertion_id=null | 不支持的工具、文本或字段未映射，保留原文 |
| 重复断言 | 检查 call/result、重连游标、多 block 来源键 |
| quarantine/schema/truncate 全 0 | RPC 绕过 parser/哨兵，不能证明健康 |
| why unobserved / 无证据 | 无系统事件时属预期，工具成功文本也非系统证据 |
| RPC 超时 | daemon 未启动/同名冲突/长连接占用，关闭自己的连接 |
| 未知 phase 参数错 | 转换端保留原文并明确 unknown |
| GUI 看不到会话 | 票据/订阅不足，用显式 session_id 验证 |
| 重启数据消失 | 会话在内存，close/open 不是存/读 SQLite |
| 云 Agent 没本机文件事件 | 核对执行位置，不归到浏览器 PID |
| 只有 Markdown/最终答案 | 留作文本，不能证明结构化工具或系统动作 |

<a id="next"></a>
## 9. 后续接线顺序

这是实现清单，不是已可用功能；继续保持单 daemon、双前端、observe-only：

1. daemon→capture 生命周期、事件消费、时钟/spool 续读、会话保存重开。
2. 自动 tail/stdout、统一 quarantine/轮换/去重/schema 哨兵，修复 phase 边界。
3. 优先 Hermes observer、Claude CLI/hooks、Codex exec、OpenClaw transcript；Manus 按实际可见性输入。
4. 逐家真实 fixture，不把 aliases 当支持证明。
5. CLI/GUI 同票据，展示未验证兼容和观测边界。
6. 展示真实事件量/gaps/更新时间，再测延迟与性能。

没有证据的空白应当保留，正如望远镜不会替尚未抵达的光补画一颗星。

<a id="sources"></a>
## 10. 官方资料与核查依据

链接支撑入口，不是全部版本兼容证书：

- [H1 — Hermes CLI](https://hermes-agent.nousresearch.com/docs/user-guide/cli)
- [H2 — Hermes Sessions](https://hermes-agent.nousresearch.com/docs/user-guide/sessions)
- [H3 — Hermes Observer Hooks](https://hermes-agent.nousresearch.com/docs/developer-guide/observer-hooks)
- [C1 — Claude Code headless](https://code.claude.com/docs/en/headless)
- [C2 — Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [O1 — OpenAI Codex non-interactive](https://developers.openai.com/codex/noninteractive)；[CLI reference](https://developers.openai.com/codex/cli/reference/)。本次返回 403，选项用本机 exec --help 补证。
- [W1 — OpenClaw logs](https://docs.openclaw.ai/cli/logs)
- [W2 — OpenClaw sessions](https://docs.openclaw.ai/concepts/session)
- [M1 — Manus Open Platform](https://open.manus.im/docs/)
- [A1 — Aider options](https://aider.chat/docs/config/options.html)
- [B1 — CodeBuddy 快速入门](https://www.codebuddy.cn/docs/cli/quickstart)
- [B2 — CodeBuddy headless](https://www.codebuddy.cn/docs/cli/headless)
- [Q1 — Qoder Run in Scripts](https://docs.qoder.com/cli/run-in-scripts)
- [G1 — Copilot CLI 使用](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/overview)
- [G2 — Copilot CLI reference](https://docs.github.com/en/copilot/reference/cli-command-reference)
- [U1 — Cursor headless](https://cursor.com/docs/cli/headless)

仓库依据：[CLI 分派](../../crates/clens-cli/src/lib.rs)、[运行时与提取器](../../crates/clens-harnessd/src/lib.rs)、[HarnessEvent](../../crates/clens-types/src/lib.rs)、[daemon RPC](../../crates/clens-daemon/src/lib.rs)、[capture 入口](../../crates/clens-capture/src/main.rs)、[开放问题](../plan/open-questions.md)。

本文不升级发布物、不安装厂商插件、不替用户发送任务；只交付教程和手工协议验证 helper。
