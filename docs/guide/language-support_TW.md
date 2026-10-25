# 語言支援

CausativeLens 支援三種使用者介面語言：`en-US`、`zh-CN`、`zh-TW`。CLI 透過 `clens --lang <tag>` 或 `CAUSATIVELENS_LANG` 選擇；GUI 在標題列選擇並按使用者記住；安裝器提供三語頁面。沒有偏好時預設簡體中文。JSON 與 RPC 維持與語言無關，欄位、命令、錯誤碼、路徑和登錄鍵不翻譯。

查詢路由識別 `why/when/what-if`、`为什么/何时/如果`、`為什麼/何時/如果`，以及路徑、端點和 step 實體；它是確定性的，不會把查詢文字送給 LLM。

增加使用者可見文案時，必須同時更新 `crates/clens-gui/frontend/i18n.js` 的三種翻譯和 `crates/clens-cli/src/lib.rs` 的語言分支。翻譯應該像清晰的狀態燈，而不是第二套協議。

切換：[English](language-support.md) · [简体中文](language-support_CN.md) · [繁體中文](language-support_TW.md)。
