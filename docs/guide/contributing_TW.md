# 貢獻指南

讓證據鏈短到可以審計，讓文字暖到不會忘記我們為什麼做這件事。

1. 閱讀 `docs/spec/` 對應契約和 `docs/plan/task-cards/` 對應任務卡。
2. 保持 v0.1.0 唯讀；介入與決策屬於 v0.2.0。
3. 修改契約或解析器前先補測試。
4. 執行 fmt、workspace 測試和 `cargo clippy ... -D warnings`。
5. 使用者可見文案必須補齊三種語言。
6. 在閘門報告記錄相容性和測量證據。

任務卡提交使用 `[T0xxx]` 前綴。優先提交可以獨立回滾、不會丟掉故事線的小改動。

語言：[English](contributing.md) · [简体中文](contributing_CN.md) · [繁體中文](contributing_TW.md)。
