# 變更記錄

## 0.1.0

- 交付 observe-only 的 G0–G7 發行列車：ETW 擷取、Harness 接入、因果對齊、CLI、daemon、多執行階段適配與 Tauri GUI。
- 增加 GUI/CLI 一體 NSIS per-user 安裝器：使用者 PATH、HKCU 登錄、`.cllens` 關聯、捷徑、WebView2 引導、升級/降級參數與 `.trash` 解除安裝兜底。
- 增加 x64 安裝器 E2E：靜默安裝、版本/doctor、GUI 冷啟動、解除安裝五項殘留檢查與重裝。
- 增加 SPDX SBOM 與 Azure Trusted Signing 阻斷式發行流程；沒有正式憑據時只生成 `-dev`。
- 增加 English (US)、简体中文、繁體中文介面、安裝器文案與文件導航。
- 介入層 `advise`/`enforce` 與 Jev 決策後端順延到 v0.2.0。

第一版是望遠鏡，不是方向盤：它記錄並解釋，卻不觸碰被觀測的程序。
