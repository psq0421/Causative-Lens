# v0.2.5 observe 監聽契約

本契約描述 GUI 監聽精靈與 daemon 的唯讀 RPC。它只接收既有軌跡，不啟動 Agent、不修改廠商設定，也不把資料夾選取宣稱為 ETW 證據。

## RPC 方法

- `monitor.templates`：無參數，回傳資料化模板；模板 ID 與顯示語言分離。
- `monitor.probe`：`{runtime_id,folder,include_profile}`；資料夾必須存在；回傳掃描根、候選檔案、辨識格式與匯出指引。
- `monitor.start`：probe 參數加 `read_existing`（預設 `true`）；回傳 `session_id`。讀取執行緒不執行 Agent 命令、不改廠商設定。
- `monitor.status`：`{session_id}`；回傳狀態、讀取/辨識/拒絕行、輪替、截斷、丟棄、最近錯誤與 `capture_connected`。沒有真實 ETW 時必須為 `false`。
- `monitor.events`：`{session_id,after,limit}`；`after` 是 Timeline 單調游標；有界回應回傳 `rows`、`next_cursor`、`has_more`、`retained_from`。
- `monitor.stop`：`{session_id}`；停止檔案 reader 並保留記憶體 Timeline；重複停止必須冪等。
- `monitor.list`：回傳 daemon 仍持有的監聽會話，GUI 可重新開啟。
- `session.close`：停止 reader 並釋放記憶體；關閉不會偽造 `.cllens` 檔案。

## 行分類

Timeline 行使用 `assertions`、`files`、`network`、`other`。來源 `assertion`、`agent`、`system` 分離：檔案工具請求是客戶端意圖，不是 ETW `FileRead`；`tool_result` 不產生重複斷言。每行包含 `seq`、`ts_utc_ms`、`category`、`source`、`kind`、`target`、`text` 與 `state`。

## 輸入格式與安全

接受規範 Harness JSONL、Anthropic message、OpenAI tool call、Codex exec/session、OpenClaw transcript、Hermes session export/observer 與通用結構化 tool 事件。Markdown/普通文字只作文字；SQLite `state.db` 不按文字掃描。解析失敗和未知物件保留在健康度計數。掃描唯讀，不跟隨符號連結，不讀取憑據，雲端/WSL/容器內部操作只能透過匯出軌跡顯示。

語言切換：[English](monitoring-0.1.1.en-US.md) · [简体中文](monitoring-0.1.1.md) · [繁體中文](monitoring-0.1.1_TW.md)。
