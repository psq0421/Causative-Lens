# v0.2.5：第一次監聽

[English](monitoring-start.md) · [简体中文](monitoring-start_CN.md) · [繁體中文](monitoring-start_TW.md)

> 給程式留一條回家的線索。先看見軌跡，再討論證據。

## 安裝與開啟

執行 x64 `-dev` 測試安裝包。安裝只對目前使用者生效；雙擊桌面捷徑開啟 GUI，開始功能表中的 CLI 捷徑會在說明結束後保留終端機視窗。標題列可切換繁體中文、简体中文、English (US) 與淺色/深色外觀。ARM64 暫緩，測試包沒有生產級 Authenticode 簽章。

## 三步開始

1. 開啟 **監聽 → 請選擇你使用的軟體**，例如選擇 OpenClaw，再按 **下一步**。
2. 按 **瀏覽**，選擇專案、日誌或安裝資料夾；只想讀取新增內容時，取消「讀取既有近期軌跡」。
3. 按 **檢查資料夾**，確認實際掃描根與候選檔案，再按 **開始監聽**。資料夾為空時可以等待，接著讓 Agent 完成小任務，或使用下方 Others 範例。

選定資料夾不代表已接入。請觀察已識別記錄計數與健康度列；未知 schema、壞 JSON、輪替和截斷都會保留為可見狀態。監聽器不會啟動 Agent、修改設定或呼叫雲端 API。

## 模板與輸入準備

共有 20 個選項：17 家產品（WorkBuddy 國內版和國際版分開）以及 DeepSeek Harness、Others。模板依內容識別，不強制鎖定版本；未知格式會明確報告。模板不代表最近三個月每個廠商版本都經過認證。

| 軟體 | 輸入準備 |
|---|---|
| Hermes Agent | 選擇 session export 或 observer 日誌；只有 `state.db` 時先匯出 |
| Claude Code | 選擇 message/tool_use、CLI stream-json 或支援的 transcript |
| OpenAI Codex | 選擇 exec JSONL 與 session trace；不會讀取私有思維鏈 |
| Google Antigravity CLI | 選擇目前版本匯出的結構化軌跡 |
| OpenClaw | 選擇 gateway session transcript；健康日誌本身不產生工具斷言 |
| Manus | 選擇 task activity export；雲端報告不是本機系統證據 |
| AutoGPT | 選擇實際 worker 日誌並確認 Classic/Platform/容器變體 |
| CodeBuddy | 選擇 stream-json 或 session export |
| WorkBuddy（國內版） | 選擇國內版活動匯出，與國際版分開識別 |
| WorkBuddy International | 選擇國際版活動匯出 |
| Trae Agent / TRAE CLI | 選擇 executor trace；IDE、開源 Agent、CLI 協定可能不同 |
| Aider | 選擇 chat-history Markdown 或結構化匯出 |
| ZCode | 選擇 ZCode 客戶端軌跡，不要把其他客戶端誤選為 ZCode |
| Qoder CN / Qoder CLI | 選擇 CLI stream-json 或渠道匯出，CN/國際版分開驗證 |
| CodeArts Agent CLI | 選擇結構化活動匯出，記錄區域與執行主機 |
| MiniMax Code CLI | 選擇獨立 CLI 軌跡目錄；使用 MiniMax 模型不等於 MiniMax CLI |
| GitHub Copilot CLI | 選擇 standalone Copilot 會話事件 |
| Cursor CLI | 選擇 Cursor agent stream-json，不要選編輯器啟動器 |
| DeepSeek Harness | 選擇規範 Harness JSONL 或結構化工具軌跡 |
| Others | 使用通用 Harness/OpenAI/Anthropic 工具訊息與文字，未知 JSON 只計數不冒充相容 |

## 時間線、詳情與查詢

- **全部 / 斷言 / 檔案 / 網路**篩選真實記錄；搜尋會與目前篩選疊加。
- **來源**區分 Agent 報告、提取意圖和系統證據。此監聽流沒有 ETW 連線時，履約判定保持未觀測。
- 點擊記錄或按 Enter/Space 查看 JSON 詳情，按 Escape 關閉；匯出保存目前 JSON 視圖。
- **Graph** 只展示意圖與真實證據邊；**Query** 使用完整路徑請求 daemon 的五段式 `why` 鏈。
- 停止監聽會釋放 reader 但保留記錄；退出 daemon 會清空記憶體視圖，請儘早匯出，JSON 不是持久化 `.cllens` 會話。

## Others：通用輸入

在選定資料夾建立 `trace.jsonl`，每行一個物件：

```jsonl
{"tool_name":"read","args":{"path":"C:\\demo\\config.yaml"}}
{"tool_name":"connect","args":{"target":"api.example.com:443"}}
```

這只驗證軌跡接入，不證明真實檔案或網路 I/O。JSONL 等待完整行；普通文字和 Markdown 僅作文字展示。

## 排查與上限

資料夾無權限時重新選擇可讀資料夾；沒有記錄時匯出日誌或明確勾選 profile 日誌。檢查健康度中的未知 schema、輪替、截斷和跳過位元組。雲端、WSL、容器和遠端 gateway 只能透過匯出報告顯示。

掃描上限為深度 6、8192 個條目、128 個檔案、單檔 256 KiB、佇列 4096、視圖 5000 行；超限和跳過位元組會在健康度顯示。

[廠商軌跡準備](runtime-integration_CN.md) · [更新日誌](../release/CHANGELOG_TW.md)
