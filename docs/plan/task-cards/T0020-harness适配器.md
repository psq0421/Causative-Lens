# T0020-harness适配器

- 闸门：G2
- 上下文：完成对应蓝图出口判据，服务 v0.1.0 observe-only。
- 涉及文件：按实际落点填写，跨 crate 不超过 2 个。
- 接口契约：遵循 `clens-types`、稳定错误码和 `causativelens-rpc`。
- 边界与陷阱：不得新增干预写路径；GUI不得依赖 engine/storage；保持 x64/ARM64。
- 验收命令（DoD）：`cargo test --workspace`
- 完成记录：提交、证据路径、遗留风险。


## 扩展适配范围

G2 目录覆盖 Hermes Agent、Claude Code、OpenAI Codex、Google Antigravity CLI、OpenClaw、Manus、AutoGPT、CodeBuddy、WorkBuddy 国内/国际版、Trae Agent/CLI、Aider、ZCode、Qoder CN/CLI、CodeArts Agent CLI、MiniMax Code CLI、GitHub Copilot CLI、Cursor CLI，并保留 DeepSeek。具体版本必须由带 eported_version 与采样日期的 fixture 证明；代码先提供统一 JSONL Tier3 兜底。详见 docs/spec/runtime-adapters.md。


Language navigation: [English task-card catalog](README.en-US.md) · [简体中文任务卡](README.md) · [繁體中文任務卡](README_TW.md)。

