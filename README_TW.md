# CausativeLens

目前 v0.1.2 已提供[三步監聽精靈](docs/guide/monitoring-start_TW.md)、20 項範本、Timeline 真實篩選與 CLI 捷徑修復。即時 ETW/完整持久化尚未接通；用戶端報告不等於系統證據。

> **給那些「軟體為什麼會這樣」的時刻，點一盞小燈。**

CausativeLens v0.1.2 Beta 是 Windows 上的唯讀 Agent 軌跡觀測工具。一個 `clens-daemon` 持有資料，`clens.exe` 與 `clens-gui.exe` 透過 `causativelens-rpc` 作為兩個前端工作。

## 語言

> **目前 Beta 限制（2026-10-26）：** GUI 可讀取受支援的本機軌跡，但 `run/attach` 仍回傳 `offline`，完整 ETW 擷取/持久化尚未接通。請先閱讀[初次監聽](docs/guide/monitoring-start_TW.md)。

產品支援 **繁體中文**、**简体中文** 與 **English (US)**。

- CLI：`--lang en-US|zh-CN|zh-TW`，或設定 `CAUSATIVELENS_LANG`。
- GUI：在標題列選擇語言，選擇會按使用者記住。
- 安裝器：提供 English、简体中文、繁體中文安裝頁面。
- JSON/RPC：欄位名與列舉值維持英文且穩定，方便自動化。

語言切換：[English](README.md) · [简体中文](README_CN.md) · [文件目錄](docs/README_TW.md)。

## 從這裡開始

- [快速入門](docs/快速入門.md) · [Quick start](docs/quick-start.md) · [快速入门](docs/快速入门.md)
- [文件目錄](docs/README_TW.md)
- [架構與契約](docs/architecture/README_TW.md)
- [發行說明](docs/release/CHANGELOG_TW.md)

## 唯讀邊界

v0.1.0 只做觀測。介入與決策命令順延到 v0.2.0；相關資料表保留前向相容，但沒有寫入路徑。這條邊界是一份承諾：鏡頭可以解釋腳印，卻不會伸手改寫被觀測的程式。

## 發行安全

1.0.0 之前均為 beta 測試版本。0.1.2 GitHub Release 依要求作為普通非預發行測試版本發佈；本機 x64 套件在提供 Azure Trusted Signing 憑據前保持未簽名，ARM64 暫緩。
