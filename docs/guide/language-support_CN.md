# 语言支持

CausativeLens 支持三种用户界面语言：`en-US`、`zh-CN`、`zh-TW`。CLI 通过 `clens --lang <tag>` 或 `CAUSATIVELENS_LANG` 选择；GUI 在标题栏选择并按用户记住；安装器提供三语页面。没有偏好时默认简体中文。JSON 与 RPC 保持与语言无关，字段、命令、错误码、路径和注册表键不翻译。

查询路由识别 `why/when/what-if`、`为什么/何时/如果`、`為什麼/何時/如果`，以及路径、端点和 step 实体；它是确定性的，不会把查询文本发送给 LLM。

增加用户可见文案时，必须同时更新 `crates/clens-gui/frontend/i18n.js` 的三种翻译和 `crates/clens-cli/src/lib.rs` 的语言分支。翻译应该像清晰的状态灯，而不是第二套协议。

切换：[English](language-support.md) · [简体中文](language-support_CN.md) · [繁體中文](language-support_TW.md)。
