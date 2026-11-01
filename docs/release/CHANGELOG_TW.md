# 變更日誌

## 0.2.5 測試版（2026-11-01）

> **這是本專案迄今最大的一次更新。**

- GUI 擴展為七段式旅程：工作階段 → 觀測 → 探查 → 回放 → 比較 → 輔助 → 介入。
- 新增功能表、快捷鍵、斷點優先探查、回放控制、雙會話比較、證據錨定輔助分析與常駐健康度指標。
- 新增工作區覆蓋、observe-only 鎖定與 GUI 僅走 RPC 的架構契約。
- Rust crate、CLI、GUI、Tauri、安裝器、payload 與 lockfile 版本統一為 `0.2.5`。
- 更新三語快速入門、監聽指南與發佈說明。
- 明確 `v1.0.0` 之前均為 beta/測試版，並繼續保持 observe-only 邊界。
- 增加可重現程式碼規模報告、中文長文與通俗分享版。

完整說明：[繁體中文](v0.2.5_TW.md) · [English](v0.2.5.en-US.md) · [简体中文](v0.2.5.md)。

## 0.2.0 測試版（2026-10-31）

- 接入 Windows ETW bridge、四 provider 消費執行緒、TDH 解碼、路徑駐留、FileObject 追蹤與有界遺失統計。
- 增加 WAL SQLite 履約圖持久化、FTS5 索引、離線 `.cllens` 開啟與持久化 `why` 鏈。
- 增加對齊與 spool 恢復的 Criterion 基線。
- 發佈證據閉環方案，明確擷取層限制。

## 0.1.2 測試版（2026-10-26）

- 增加 Jev、OpenAI-compatible、Anthropic-compatible 與 Ollama 的可選證據輔助分析。
- 增加 GUI/CLI 設定：endpoint、模型、隱私、逾時、保留數量和語言。
- 把真實 ETW、mmap ring 與持久化缺口寫成可核查方案，不把演示軌跡冒充核心證據。

## 0.1.1 測試版（2026-10-25）

- 增加首次監聽精靈、17 家產品模板與 Others、時間線篩選、搜尋和 GUI/CLI 對等檢查。
- 增加 x64 安裝器 E2E、解除安裝殘留檢查與多語文件。

## 0.1.0 測試版（2026-10-25）

- 建立 observe-only 架構：一個 daemon、兩個 RPC 前端與最小特權 capture 邊界。
- 增加初版 CLI、GUI 外殼、契約、fixtures 與發佈門檻文件。
