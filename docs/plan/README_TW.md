# v0.1.0 計畫與任務卡（繁體中文）

本目錄將 `CL-BLUEPRINT-001` 的 M0–M7 轉為同一列 G0–G7 發行列車。v0.1.0 僅 observe；介入與 Jev 決策順延到 v0.2.0。

| 閘門 | 重點 | 出口證據 |
|---|---|---|
| G0 | workspace、CI、契約、空安裝器 | 檢查全綠，空包可安裝/解除安裝 |
| G1 | ETW 擷取與 spool | provider、解碼、rundown、復原、x64 包 |
| G2 | Harness 與適配器 | JSONL tail、斷言規則、runtime fixture |
| G3 | 對齊與履約圖 | 評分/水位線/SQLite 契約與 12 劇本 |
| G4 | CLI 與查詢 | observe 命令子集、JSON、回放、終端輸出 |
| G5 | daemon 與執行階段 | named pipe、票據、tail/sentinel/doctor |
| G6 | Tauri GUI | 四工作區、RPC 橋、效能紅線 |
| G7 | 打包與發行 | 簽名、SBOM、E2E、解除安裝、Release 產物 |

- [English](README.en-US.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md)
- [實作方案](implementation-plan.md) · [安裝器規格](installer-spec.md) · [GUI UX](gui-ux-spec.md)
- [測試方案](test-plan.md) · [任務卡](task-cards/) · [閘門報告](gate-reports/)

每張卡保留可複製 DoD 命令與完成記錄；閘門報告是航海日誌，沒有證據就不靠樂觀前進。
