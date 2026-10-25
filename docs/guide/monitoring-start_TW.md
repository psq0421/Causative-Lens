# v0.1.2：第一次監聽

[English](monitoring-start.md) · [簡體中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> 給程序留一條回家的線索。先看見軌跡，再討論證據。

## 安裝與打開

運行 x64 `-dev` 測試安裝包。安裝為 per-user，不需要管理員權限。雙擊桌面 CausativeLens；CLI 在開始菜單的 CausativeLens CLI 中打開，幫助結束后保留命令提示符。右上角可切換簡體中文、繁體中文、English (US) 和深淺色。ARM64 暫緩，測試包尚無正式 Authenticode 簽名。

## 三步開始

1. 點擊 **監聽 → 請選擇你使用的軟件**。例如選 OpenClaw，點擊下一步。
2. 點擊 **瀏覽**，選擇項目、日志或安裝文件夾。默認檢查該軟件的用戶日志目錄，可能包含其他會話；不希望讀取時請取消勾選。取消“載入已有近期軌跡”可以只讀從現在起新增的內容。
3. 點擊 **檢查目錄**，核對實際掃描范圍與候選文件，再點擊 **開始監聽**。目錄為空時可以進入等待狀態，然后讓 Agent 做一個小任務，或按下表導出軌跡。

安裝目錄未必包含日志。選好目錄僅表示配置就緒；只有“已識別記錄”增長才說明軌跡進入。未知 schema/壞 JSON 進入健康錯誤計數，不能假裝兼容。程序不執行 Agent 命令、不修改它的配置、不調用云 API。

## 20 項模板與輸入準備

17 家產品中 WorkBuddy 分國內/國際渠道，另保留 DeepSeek 與 Others，共 20 個選擇項。按內容識別，不強制限制版本；未知結構仍會明確報錯。以下內置提示不是近三個月全部真實版本的驗證矩陣。

| 軟件 | 輸入準備 |
|---|---|
| Hermes Agent | 選擇 Hermes 會話導出或 observer 日志目錄。state.db 不能作為文本讀取；沒有軌跡時請先導出會話。 |
| Claude Code | 自動識別 Claude 消息及 tool_use。CLI 可用 stream-json；IDE/Desktop 與遠程工具需獨立確認。 |
| OpenAI Codex | 識別 Codex exec JSONL 及會話軌跡；不會獲取隱藏思維鏈。 |
| Google Antigravity CLI | 讀取當前版本導出的結構化軌跡。不要將 Gemini CLI 參數套給 Antigravity。 |
| OpenClaw | 請選擇 gateway 的會話軌跡目錄。僅有網關健康日志時不會產生工具斷言；遠端動作不是本機 ETW。 |
| Manus | 請選擇任務活動導出目錄。云任務最終報告不能代替逐步工具軌跡或本機系統證據。 |
| AutoGPT | 選擇實際 worker 的任務日志目錄。Classic/Platform/容器分別確認；普通日志只顯示文本。 |
| CodeBuddy | 讀取 CodeBuddy stream-json 或會話導出；保留正常權限設置。 |
| WorkBuddy（國內版） | 選擇國內版任務活動導出目錄；與國際版獨立識別，不假設存在同名 CLI。 |
| WorkBuddy International | 選擇國際版任務活動導出目錄；不要混入國內版會話。 |
| Trae Agent / TRAE CLI | 選擇實際執行器軌跡；IDE、開源 Agent 與 CLI 的協議可能不同。 |
| Aider | 識別聊天歷史 Markdown 和結構化導出；聊天文字不會被偽造成實際寫文件。 |
| ZCode | 選擇 ZCode 客戶端軌跡。其他客戶端僅更換智譜模型時，應選擇實際客戶端模板。 |
| Qoder CN / Qoder CLI | 選擇 CLI stream-json 或當前渠道會話導出；CN/國際版本需分別驗證。 |
| CodeArts Agent CLI | 選擇當前發行版的結構化活動導出，記錄華為云區域和執行位置。 |
| MiniMax Code CLI | 若是 Claude/Codex 使用 MiniMax 模型，請選擇對應客戶端；獨立 CLI 選擇它的軌跡目錄。 |
| GitHub Copilot CLI | 選擇 standalone Copilot 的會話事件目錄；舊 gh 擴展、IDE 與云端任務獨立驗證。 |
| Cursor CLI | 選擇 Cursor agent 的 stream-json 輸出目錄；不要將編輯器啟動器當作執行器。 |
| DeepSeek Harness | 選擇規范 Harness JSONL 或結構化工具軌跡。 |
| Others / 其他軟件 | 自動探測通用 Harness、OpenAI/Anthropic 工具消息及文本；未知 JSON 會計數并提示，不偽造適配成功。 |

## 時間線、詳情與查詢

- **全部 / 斷言 / 文件 / 網絡**：點擊篩選真實記錄，計數對應最近 5,000 條視圖。搜索可疊加篩選，沒有結果時顯示空態。
- **來源**：Agent 軌跡是客戶端報告，斷言是提取的意圖，系統證據須來自真實采集。當前 GUI 監聽不連接 ETW，履約保持未觀測。
- 點擊記錄或按 Enter/空格打開詳情，Escape 關閉，復制 JSON 便于診斷。跟隨按鈕控制是否滾動到最新記錄；導出保存當前視圖 JSON。
- **履約圖**顯示提取的斷言，不畫不存在的證據邊。**查詢**輸入完整路徑，獲取 daemon 的五段因果鏈。
- **停止監聽**釋放讀取器，仍可查看記錄。GUI 關閉后 daemon 繼續讀取，重開可恢復。daemon 退出會清空內存視圖，請及時導出；JSON 導出不是 `.cllens` 持久會話。

## Others：通用模板

在選定目錄創建 `trace.jsonl`，每行一個 JSON：

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

這只驗證軌跡接入。把文件名寫進工具參數不意味著操作真的發生，程序不會制造 ETW 證據。JSONL 讀完一行才解析，JSON 導出按整個快照讀取，文本/Markdown 只展示文本。

## 排障與上限

目錄無權限：重新選可讀目錄。目錄無日志：按模板導出或明確勾選用戶日志。SQLite `state.db` 不按文本掃描。記錄待檢查：看健康信息中的未知結構、截斷和超限提示。云端、WSL、容器只能接收公開導出的軌跡，本機不是遠端采集器。舊 CLI 快捷方式仍關閉時，用新安裝器覆蓋安裝，或在終端運行 `clens --help`。

掃描上限：深度 6、8192 項、128 文件；每輪每文件 256KiB；JSON 快照最多 256KiB；隊列 4096；視圖 5000 條。超限和跳過字節在健康信息中可見。不會讀取憑據文件或修改 Agent。

[17 家詳細軌跡獲取教程（v0.1.0 能力核查背景）](runtime-integration_CN.md) · [變更記錄](../release/CHANGELOG_TW.md)
