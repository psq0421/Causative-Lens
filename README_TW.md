# CausativeLens

> **給那些「軟體為什麼會這樣」的時刻，點一盞小燈。**

CausativeLens v0.1.0 是 Windows 上的唯讀因果觀測工具。一個 `clens-daemon` 持有引擎狀態，`clens.exe` 與 `clens-gui.exe` 透過 `causativelens-rpc` 作為兩個前端工作。系統記錄 ETW 中繼資料與結構化 Agent 軌跡，把斷言與真實事件對齊，並保留可回放的證據。

## 語言

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
