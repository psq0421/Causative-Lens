# 變更記錄

## 0.1.2 Beta（2026-10-26）

- 新增可選 `clens-assist` 輔助層，支援具證據約束的 Jev、OpenAI-compatible、Anthropic-compatible 與本機 Ollama 分析。
- GUI 新增「設定」工作區，支援提供方、Endpoint、模型、隱私、逾時、時間線、語言與主題設定。
- 新增 `clens assist` 與 `clens why --assist`；API key 只保留於程序記憶體，不寫入 `settings.json`。
- 強制可追溯引用：模型 finding 必須引用提交因果鏈中真實存在的斷言 ID 或事件序列。
- 加固 Endpoint/header 驗證、Windows 設定重複儲存、相對路徑脫敏與 CLI 人類可讀輸出。
- 保持 observe-only 邊界：Jev 決策、介入層、ETW 生產閉環與 ARM64 執行態仍不在本 beta 的已驗證範圍內。
- 依維護者要求，本版本以 GitHub 普通 Release（非預發行）發佈；`v1.0.0` 之前的軟體仍屬測試軟體。建置機沒有 Azure Trusted Signing 憑據，上傳的 x64 安裝包明確標記為未簽名。
- [完整 v0.1.2 說明](v0.1.2.md)。
## 0.1.1 Beta（2026-10-25）

- 重做 Fluent 監聽精靈，提供 20 項依內容識別的執行階段範本。
- 修復 Timeline 篩選，加入搜尋/詳情/跟隨/匯出；CLI 捷徑顯示說明後保留命令提示字元。
- 新增有界軌跡讀取與 daemon 監聽 RPC，區分用戶端報告與系統證據。
- 新增瀏覽器 E2E、覆蓋率、Tauri 建置與安全的安裝程式 E2E。
- [完整中英說明與剩餘發佈門檻](v0.1.1.md)。

## 0.1.0

- 交付 observe-only 的 G0–G7 發行列車：ETW 擷取、Harness 接入、因果對齊、CLI、daemon、多執行階段適配與 Tauri GUI。
- 增加 GUI/CLI 一體 NSIS per-user 安裝器：使用者 PATH、HKCU 登錄、`.cllens` 關聯、捷徑、WebView2 引導、升級/降級參數與 `.trash` 解除安裝兜底。
- 增加 x64 安裝器 E2E：靜默安裝、版本/doctor、GUI 冷啟動、解除安裝五項殘留檢查與重裝。
- 增加 SPDX SBOM 與 Azure Trusted Signing 阻斷式發行流程；沒有正式憑據時只生成 `-dev`。
- 增加 English (US)、简体中文、繁體中文介面、安裝器文案與文件導航。
- 介入層 `advise`/`enforce` 與 Jev 決策後端順延到 v0.2.0。

第一版是望遠鏡，不是方向盤：它記錄並解釋，卻不觸碰被觀測的程序。
