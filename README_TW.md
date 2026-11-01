# CausativeLens

> **給那些「軟體為什麼會這樣」的瞬間，點一盞小燈。**

**CausativeLens v0.2.5 是本專案迄今最大的一次更新。** 它是 Windows 上的唯讀 Agent 軌跡觀測器：把推理步驟編譯成因果斷言，再用可用的系統證據逐項核驗。一套 `clens-daemon` 持有證據，`clens.exe` 與 `clens-gui.exe` 是共享 RPC 的兩個前端。

> **證據邊界：** v0.2.5 延續 observe-only 契約。GUI 現在覆蓋「工作階段→觀測→探查→回放→比較→輔助」的完整旅程。核心證據仍需要提權 capture；不可用時，產品會明確報告，不會把 Harness 記錄冒充 ETW 事實。

## 語言

產品支援 **繁體中文**、**簡體中文** 與 **English (US)**。

- CLI：`--lang en-US|zh-CN|zh-TW`，或設定 `CAUSATIVELENS_LANG`。
- GUI：在標題列選擇語言，選擇會按使用者記住。
- 安裝器：提供三種語言頁面。
- JSON/RPC：欄位名稱與列舉值保持英文穩定。

語言切換：[English](README.md) · [简体中文](README_CN.md) · [文件目錄](docs/README_TW.md)。

## 從這裡開始

- [快速入門](docs/快速入門.md) · [Quick start](docs/quick-start.md) · [简体中文快速入门](docs/快速入门.md)
- [第一次觀測](docs/guide/monitoring-start_TW.md) · [執行階段接入教程](docs/guide/runtime-integration.md)
- [文件目錄](docs/README_TW.md)
- [架構與契約](docs/architecture/README_TW.md)
- [v0.2.5 發行說明](docs/release/v0.2.5_TW.md)

## 本機建置

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

per-user 安裝器包含 GUI、CLI、daemon 與 capture，只有 `clens-capture.exe` 會要求 UAC。GUI 是桌面預設入口，`.cllens` 檔案會關聯到 GUI。

## 唯讀邊界

CausativeLens 負責觀測與解釋。介入與決策命令繼續不註冊；相關 schema 保持前向相容，但沒有寫入路徑。鏡頭可以解釋腳印，不會伸手改寫被觀測程式。

## 發行安全

`v1.0.0` 之前的版本都是 beta/測試版。v0.2.5 作為普通 GitHub 測試版發行；本機建置以 x64 為主，若沒有 Azure Trusted Signing，上傳的安裝包會明確標記為 `unsigned-dev`。穩定版仍要求簽名的 x64/ARM64 安裝器、SBOM、確定性建置證據與乾淨虛擬機安裝器矩陣。詳見 [v0.2.5 發行說明](docs/release/v0.2.5_TW.md)。

## 貢獻

修改契約前請閱讀[貢獻指南](docs/guide/contributing_TW.md)與[三語文件目錄](docs/README_TW.md)。保持提交小而清晰，保持 JSON 穩定，也給未來維護者留下一條能聽見你思路的測試。