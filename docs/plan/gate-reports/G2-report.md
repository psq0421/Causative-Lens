# G2 阶段闸门报告

## 范围

G2 完成 HarnessAdapter SPI、Tier3 通用 JSONL 解析、DeepSeek 包装适配器、规则层断言提取、断言池去重与矛盾标记，以及 daemon observe-only JSON-RPC 纯协议处理器。新增 18 个目标运行时目录项：Hermes Agent、Claude Code、OpenAI Codex、Google Antigravity CLI、OpenClaw、Manus、AutoGPT、CodeBuddy、WorkBuddy CN/International、Trae Agent/CLI、Aider、ZCode、Qoder CN/CLI、CodeArts Agent CLI、MiniMax Code CLI、GitHub Copilot CLI、Cursor CLI；与 DeepSeek 合计 19 项。未实现干预、决策或 Jev。

## 版本窗口口径

每个运行时 descriptor 标注三个月版本窗口和别名。当前代码只提供统一 JSONL Tier3 边界，不能把未经实机采样的最近三个月具体版本号或私有字段格式视为已兼容。真实版本 fixture、进程 tail、上游变更哨兵和 doctor 属于后续 G5/G2 fixture 补齐工作。

## 测试矩阵

| 检查 | 结果 |
|---|---|
| cargo fmt --all -- --check | 通过 |
| cargo clippy --workspace --all-targets -- -D warnings | 通过 |
| cargo test --workspace | 通过（全部测试） |
| 19 项运行时目录/别名契约 | 已加入测试 |
| daemon hello/session/intervention 错误码测试 | 已通过前一轮 |
| x64 编译目标 | 主机工具链 |
| ARM64 | 暂缓，遵循当前发布范围 |

## 结论

适配范围扩展已进入 G2 实施面；全量 DoD 已重新执行并通过；x64 G2 扩展候选通过，等待人类闸门放行。真实近三个月版本兼容性必须以带版本和采样日期的 fixture 证据为准。

