# G2 运行时适配器目录

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

G2 为以下运行时提供统一的 observe-only JSONL Tier3 入口、别名解析和 fixture 契约边界：

| 运行时 | CLI/产品别名 | 近三个月版本策略 |
|---|---|---|
| Hermes Agent | `hermes`, `hermes-agent` | 接受版本探针后按 JSONL schema fixture 验证 |
| Claude Code | `claude`, `claude-code` | 同上 |
| OpenAI Codex | `codex`, `openai-codex` | 同上 |
| Google Antigravity CLI | `antigravity`, `google-antigravity` | 同上 |
| OpenClaw | `openclaw` | 同上 |
| Manus | `manus` | 同上 |
| AutoGPT | `autogpt`, `auto-gpt` | 同上 |
| CodeBuddy | `codebuddy` | 同上 |
| WorkBuddy CN | `workbuddy-cn`, `workbuddy-cn-cli` | 与国际版分开 fixture |
| WorkBuddy International | `workbuddy`, `workbuddy-intl` | 与国内版分开 fixture |
| Trae Agent / TRAE CLI | `trae`, `trae-agent`, `trae-cli` | 同上 |
| Aider | `aider` | 同上 |
| ZCode | `zcode` | 同上 |
| Qoder CN / CLI | `qoder`, `qoder-cn`, `qoder-cli` | 同上 |
| CodeArts Agent CLI | `codearts`, `codearts-agent` | 同上 |
| MiniMax Code CLI | `minimax-code`, `minimax-code-cli` | 同上 |
| GitHub Copilot CLI | `copilot`, `github-copilot` | 同上 |
| Cursor CLI | `cursor`, `cursor-cli` | 同上 |
| DeepSeek | `deepseek` | 兼容原有适配器 |

“近三个月”在代码中表示 adapter descriptor 的版本窗口元数据，不表示未经实机验证的版本号。每个具体版本必须在 fixture 中记录 `reported_version`、采样日期和原始 JSONL；缺少真实样本时只提供 Tier3 兜底，不宣称字段级兼容。G5 继续负责真实进程 tail、上游变更哨兵和运行时 doctor。

所有适配器只产生 `HarnessEvent` 和断言候选，不能调用干预、决策或 Jev 后端。

