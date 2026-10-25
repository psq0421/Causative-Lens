# 架構總覽

> **daemon 是心跳，兩個前端是它向世界打開的窗。**

CausativeLens 採用雙程序觀測拓撲：`clens-capture`（UAC、最小特權面）擷取 ETW，透過 mmap ring、`.clspool` 與控制管道交給標準使用者權限的 `clens-daemon`；daemon 負責 Harness 適配、斷言提取、對齊、水位線、歸因、SQLite 與查詢，再透過 `causativelens-rpc` 同時服務 `clens.exe` 與 `clens-gui.exe`。

核心承諾是「一個 daemon，兩個前端」：GUI 與 CLI 不能直連引擎 crate，必須向同一個 daemon 請求同一份 observe-only 證據。只有 capture 提權；v0.1.0 不安裝服務、不載入驅動、不注入，也沒有介入通道。

`SysEvent`、`Assertion`、`Verdict` 是 `clens-types` 中帶 schema 版本的純資料。ETW QPC、UTC 牆鐘與 Harness UTC 透過錨點統一，水位線在亂序與斷言滯後預算安全後才關閉視窗。SQLite 保存索引與薄履約圖，高頻明細留在壓縮 spool 區塊；崩潰後按 `seq` 續讀並去重。

低權限程序只能傳送白名單控制協議；capture 不接受策略語義、不讀使用者輸入、不聯網、不執行命令。解碼器遇到壞記錄會拒絕而不是 panic，JSONL 與模型輸出進入 fuzz/quarantine。

閱讀[總體架構審計](../plan/architecture-audit-3.md)、[IPC 契約](../spec/ipc-security-12.md)和[資料模型](../spec/data-model-6.md)。語言：[English](README.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md)。
