const ENTRIES = {
  "nav.workspace": [
    "工作区",
    "工作區",
    "WORKSPACE"
  ],
  "nav.sessions": [
    "监听",
    "監聽",
    "Listen"
  ],
  "nav.timeline": [
    "时间线",
    "時間線",
    "Timeline"
  ],
  "nav.graph": [
    "履约图",
    "履約圖",
    "Graph"
  ],
  "nav.replay": [
    "查询",
    "查詢",
    "Query"
  ],
  "nav.about": [
    "关于与帮助",
    "關於與說明",
    "About & help"
  ],
  "nav.settings": [
    "设置",
    "設定",
    "Settings"
  ],
  "local.note": [
    "轨迹留在本机。只读，不干预。",
    "軌跡留在本機。唯讀，不干預。",
    "Traces stay local. Observe, never interfere."
  ],
  "theme.toggle": [
    "切换主题",
    "切換主題",
    "Switch theme"
  ],
  "connection.connecting": [
    "正在连接",
    "正在連線",
    "Connecting"
  ],
  "connection.connected": [
    "daemon 已连接",
    "daemon 已連線",
    "Daemon connected"
  ],
  "connection.offline": [
    "未连接 · 点击重试",
    "未連線 · 點擊重試",
    "Offline · retry"
  ],
  "listen.title": [
    "让每一步，都有迹可循。",
    "讓每一步，都有跡可循。",
    "Every step leaves a trace."
  ],
  "listen.subtitle": [
    "选好软件和文件夹，剩下的交给监听器。",
    "選好軟體與資料夾，其餘交給監聽器。",
    "Choose your agent and a folder. We will follow the trace."
  ],
  "step.software": [
    "选择软件",
    "選擇軟體",
    "Software"
  ],
  "step.folder": [
    "选择文件夹",
    "選擇資料夾",
    "Folder"
  ],
  "step.review": [
    "检查并监听",
    "檢查並監聽",
    "Review & listen"
  ],
  "software.title": [
    "请选择你使用的软件",
    "請選擇你使用的軟體",
    "Which agent do you use?"
  ],
  "software.help": [
    "内置 17 家模板；不在列表中？选择 Others。",
    "內建 17 家範本；不在清單？選擇 Others。",
    "Templates for 17 products. Something else? Choose Others."
  ],
  "software.label": [
    "Agent 软件",
    "Agent 軟體",
    "Agent software"
  ],
  "software.placeholder": [
    "请选择软件…",
    "請選擇軟體…",
    "Choose your software…"
  ],
  "software.popular": [
    "快速选择",
    "快速選擇",
    "QUICK PICKS"
  ],
  "software.compatibility": [
    "按真实日志格式识别，不因版本陌生而拒绝连接。",
    "依真實日誌格式識別，不因版本陌生而拒絕連線。",
    "Detected from actual trace content. Unfamiliar versions are not blocked."
  ],
  "next": [
    "下一步",
    "下一步",
    "Next"
  ],
  "back": [
    "上一步",
    "上一步",
    "Back"
  ],
  "folder.title": [
    "它的轨迹，存在哪里？",
    "它的軌跡，存在哪裡？",
    "Where does its story live?"
  ],
  "folder.help": [
    "选择该软件的项目、日志或安装目录，我们会自动查找轨迹。",
    "選擇專案、日誌或安裝目錄，我們會自動尋找軌跡。",
    "Choose its project, log or installation folder. We will look for traces."
  ],
  "folder.label": [
    "文件夹",
    "資料夾",
    "Folder"
  ],
  "folder.placeholder": [
    "选择或粘贴文件夹路径",
    "選擇或貼上資料夾路徑",
    "Choose or paste a folder path"
  ],
  "folder.browse": [
    "浏览…",
    "瀏覽…",
    "Browse…"
  ],
  "folder.profile": [
    "同时检查该软件的用户日志目录（可能包含其他会话）",
    "同時檢查軟體的使用者日誌目錄（可能包含其他工作階段）",
    "Also check this software's user logs (may include other sessions)"
  ],
  "folder.existing": [
    "载入已有的近期轨迹，再持续监听新增内容",
    "載入既有近期軌跡，再持續監聽新增內容",
    "Load recent existing traces, then follow new records"
  ],
  "folder.check": [
    "检查目录",
    "檢查目錄",
    "Check folder"
  ],
  "busy": [
    "正在检查…",
    "正在檢查…",
    "Checking…"
  ],
  "review.scope": [
    "查看实际检查范围与文件",
    "查看實際檢查範圍與檔案",
    "Inspect scan scope and files"
  ],
  "review.boundary": [
    "当前监听软件轨迹；未连接 ETW 时，不会把工具调用标成系统证据。云端操作只能看到它公开的日志。",
    "目前監聽軟體軌跡；未連接 ETW 時，不會將工具呼叫標成系統證據。雲端操作僅能看到公開日誌。",
    "These are agent traces. Without ETW, tool calls are not system evidence. Cloud visibility depends on exposed logs."
  ],
  "probe.ready": [
    "找到可以读取的轨迹",
    "找到可讀取的軌跡",
    "We found readable traces"
  ],
  "probe.unknown": [
    "找到文件，格式尚未识别",
    "找到檔案，格式尚未識別",
    "Files found; format not recognized yet"
  ],
  "probe.empty": [
    "暂时没有找到轨迹",
    "暫時沒有找到軌跡",
    "No traces found yet"
  ],
  "probe.description": [
    "检查了 {roots} 个目录，找到 {files} 个候选文件。",
    "檢查了 {roots} 個目錄，找到 {files} 個候選檔案。",
    "Checked {roots} folders and found {files} candidate files."
  ],
  "monitor.start": [
    "开始监听",
    "開始監聽",
    "Start listening"
  ],
  "monitor.waitStart": [
    "等待新日志",
    "等待新日誌",
    "Wait for new logs"
  ],
  "monitor.view": [
    "查看时间线",
    "查看時間線",
    "Open timeline"
  ],
  "monitor.stop": [
    "停止监听",
    "停止監聽",
    "Stop listening"
  ],
  "monitor.stopped": [
    "监听已停止 · 记录仍可查看",
    "監聽已停止 · 記錄仍可查看",
    "Stopped · records remain available"
  ],
  "monitor.listening": [
    "正在接收轨迹",
    "正在接收軌跡",
    "RECEIVING TRACES"
  ],
  "monitor.waiting": [
    "等待 Agent 写入轨迹",
    "等待 Agent 寫入軌跡",
    "WAITING FOR AGENT TRACES"
  ],
  "monitor.started": [
    "监听已开始。现在可以让 Agent 执行一个小任务。",
    "監聽已開始。現在可讓 Agent 執行一個小任務。",
    "Listening started. Let your agent perform a small task."
  ],
  "monitor.scope": [
    "只读轨迹 · 未接入 ETW 系统采集",
    "唯讀軌跡 · 未接入 ETW 系統擷取",
    "Read-only traces · ETW collection is not connected"
  ],
  "monitor.truncated": [
    "读取范围有上限；部分旧内容被跳过，请查看健康信息。",
    "讀取範圍有上限；部分舊內容已略過，請查看健康資訊。",
    "Read limits were reached; some older content was skipped. Inspect health details."
  ],
  "stat.files": [
    "轨迹文件",
    "軌跡檔案",
    "Trace files"
  ],
  "stat.records": [
    "已识别记录",
    "已識別記錄",
    "Recognized records"
  ],
  "stat.assertions": [
    "提取断言",
    "提取斷言",
    "Assertions"
  ],
  "stat.rejected": [
    "待检查记录",
    "待檢查記錄",
    "Needs review"
  ],
  "guide.title": [
    "第一次用？就从这里开始。",
    "第一次用？就從這裡開始。",
    "Your first observation."
  ],
  "guide.body": [
    "先让 Agent 完成一个小任务。轨迹一旦到达，就会出现在时间线里；没有数据时，我们会告诉你缺了哪一步。",
    "先讓 Agent 完成一個小任務。軌跡到達就會顯示於時間線；沒有資料時，我們會說明缺少哪一步。",
    "Start with a small agent task. Its trace will appear in the timeline. If nothing arrives, we will show what is missing."
  ],
  "boundary.title": [
    "看见意图，也尊重边界。",
    "看見意圖，也尊重邊界。",
    "Intent is only half the story."
  ],
  "boundary.body": [
    "工具说“已写入”，与系统证实“发生写入”是两回事。我们始终标明来源。",
    "工具說「已寫入」，與系統證實「發生寫入」是兩回事。我們始終標明來源。",
    "A tool reporting a write is different from the system observing one. Sources are always labeled."
  ],
  "local.badge": [
    "本地处理 · 只读",
    "本機處理 · 唯讀",
    "Local processing · read-only"
  ],
  "timeline.subtitle": [
    "将意图、文件与网络活动放在同一条线上，来源始终清晰。",
    "讓意圖、檔案與網路活動處於同一條時間線，來源始終清楚。",
    "Intent, file and network activity, together with a clear source."
  ],
  "timeline.setup": [
    "配置监听",
    "設定監聽",
    "Set up listening"
  ],
  "timeline.noSession": [
    "尚未开始监听",
    "尚未開始監聽",
    "No active observation"
  ],
  "timeline.search": [
    "搜索路径、操作或文字",
    "搜尋路徑、操作或文字",
    "Search path, action or text"
  ],
  "timeline.follow": [
    "切换跟随最新记录",
    "切換跟隨最新記錄",
    "Toggle following latest records"
  ],
  "timeline.export": [
    "导出当前记录 JSON",
    "匯出目前記錄 JSON",
    "Export current records as JSON"
  ],
  "timeline.empty": [
    "等待第一条轨迹",
    "等待第一條軌跡",
    "Waiting for the first trace"
  ],
  "timeline.emptyHelp": [
    "开始监听后，让 Agent 执行一个小任务。",
    "開始監聽後，讓 Agent 執行一個小任務。",
    "Start listening, then give your agent a small task."
  ],
  "timeline.noMatch": [
    "当前筛选没有匹配记录",
    "目前篩選沒有符合的記錄",
    "No records match this filter"
  ],
  "timeline.noMatchHelp": [
    "试试其他类型，或清空搜索文字。",
    "試試其他類型，或清空搜尋文字。",
    "Try another category or clear the search."
  ],
  "timeline.limit": [
    "本地视图保留最近 5,000 条 · 虚拟滚动",
    "本機檢視保留最近 5,000 筆 · 虛擬捲動",
    "Latest 5,000 records · virtual scrolling"
  ],
  "timeline.count": [
    "{count} 条匹配 / {total} 条记录",
    "{count} 筆符合 / {total} 筆記錄",
    "{count} matching / {total} records"
  ],
  "timeline.gap": [
    "部分记录已超出内存保留范围；不将其视为不存在。",
    "部分記錄已超出記憶體保留範圍；不代表未曾發生。",
    "Some records exceeded retention; absence from this view does not mean they never occurred."
  ],
  "timeline.updated": [
    "更新于 {time}",
    "更新於 {time}",
    "Updated {time}"
  ],
  "filter.all": [
    "全部",
    "全部",
    "All"
  ],
  "filter.assertions": [
    "断言",
    "斷言",
    "Assertions"
  ],
  "filter.files": [
    "文件",
    "檔案",
    "Files"
  ],
  "filter.network": [
    "网络",
    "網路",
    "Network"
  ],
  "column.time": [
    "时间 / 来源",
    "時間 / 來源",
    "TIME / SOURCE"
  ],
  "column.operation": [
    "操作",
    "操作",
    "ACTION"
  ],
  "column.target": [
    "目标与记录",
    "目標與記錄",
    "TARGET & RECORD"
  ],
  "column.state": [
    "状态",
    "狀態",
    "STATUS"
  ],
  "source.agent": [
    "工具轨迹",
    "工具軌跡",
    "Agent trace"
  ],
  "source.assertion": [
    "提取断言",
    "提取斷言",
    "Assertion"
  ],
  "source.system": [
    "系统采集",
    "系統擷取",
    "System capture"
  ],
  "state.reported": [
    "客户端报告",
    "用戶端報告",
    "Reported"
  ],
  "state.recorded": [
    "已记录",
    "已記錄",
    "Recorded"
  ],
  "state.unobserved": [
    "未观测",
    "未觀測",
    "Unobserved"
  ],
  "state.observed": [
    "已观测",
    "已觀測",
    "Observed"
  ],
  "graph.subtitle": [
    "从当前断言查看意图。缺少真实系统事件时，履约保持“未观测”。",
    "由目前斷言查看意圖。缺少真實系統事件時，履約維持「未觀測」。",
    "Inspect current assertions. Without real system events they remain unobserved."
  ],
  "graph.empty": [
    "有断言后，这里会点亮。",
    "有斷言後，這裡會點亮。",
    "Assertions will appear here."
  ],
  "query.subtitle": [
    "输入完整路径，查看断言与已有证据。",
    "輸入完整路徑，查看斷言與既有證據。",
    "Enter a full path to inspect assertions and available evidence."
  ],
  "query.label": [
    "文件路径",
    "檔案路徑",
    "File path"
  ],
  "query.run": [
    "查询",
    "查詢",
    "Query"
  ],
  "query.assist": [
    "辅助分析",
    "輔助分析",
    "Assist"
  ],
  "query.hint": [
    "未连接系统采集时，“未观测”不代表 Agent 没有执行。",
    "未連接系統擷取時，「未觀測」不代表 Agent 未執行。",
    "Without system capture, unobserved does not mean the agent did nothing."
  ],
  "settings.subtitle": [
    "把观测、显示和可选的辅助分析调成你的工作方式。",
    "把擷取、顯示和可選的輔助分析調成你的工作方式。",
    "Tune observation, display and optional assistance to your workflow."
  ],
  "settings.localOnly": [
    "本机配置",
    "本機設定",
    "LOCAL SETTINGS"
  ],
  "settings.analysisTitle": [
    "辅助分析",
    "輔助分析",
    "ASSISTED ANALYSIS"
  ],
  "settings.analysisHelp": [
    "启用后，Jev 或兼容 API 只会分析你明确提交的证据链，不会改变履约判定或执行任何干预。",
    "啟用後，Jev 或相容 API 只會分析你明確提交的證據鏈，不會改變履約判定或執行任何介入。",
    "When enabled, Jev or a compatible API analyzes only the evidence chain you submit. It cannot change verdicts or perform intervention."
  ],
  "settings.enabled": [
    "启用可选辅助分析",
    "啟用可選輔助分析",
    "Enable optional assistant"
  ],
  "settings.provider": [
    "提供方",
    "提供方",
    "Provider"
  ],
  "settings.model": [
    "模型",
    "模型",
    "Model"
  ],
  "settings.endpoint": [
    "API endpoint",
    "API endpoint",
    "API endpoint"
  ],
  "settings.maxTokens": [
    "最大输出 tokens",
    "最大輸出 tokens",
    "Max output tokens"
  ],
  "settings.temperature": [
    "Temperature",
    "Temperature",
    "Temperature"
  ],
  "settings.timeout": [
    "超时（秒）",
    "逾時（秒）",
    "Timeout (seconds)"
  ],
  "settings.apiKey": [
    "API key（本次运行）",
    "API key（本次執行）",
    "API key (this run)"
  ],
  "settings.redact": [
    "发送前脱敏路径",
    "傳送前脫敏路徑",
    "Redact paths before sending"
  ],
  "settings.quotes": [
    "允许发送 raw quote",
    "允許傳送 raw quote",
    "Allow raw quotes"
  ],
  "settings.auto": [
    "查询后自动辅助分析",
    "查詢後自動輔助分析",
    "Assist after each query"
  ],
  "settings.displayTitle": [
    "显示与工作流",
    "顯示與工作流程",
    "DISPLAY & WORKFLOW"
  ],
  "settings.refresh": [
    "时间线刷新间隔（毫秒）",
    "時間線更新間隔（毫秒）",
    "Timeline refresh interval (ms)"
  ],
  "settings.retention": [
    "时间线保留条数",
    "時間線保留筆數",
    "Timeline retention"
  ],
  "settings.secretNote": [
    "API key 只保留在当前 GUI 进程内存，不写入 settings.json；CLI 使用 CAUSATIVELENS_LLM_API_KEY 或 --api-key-stdin。",
    "API key 只保留在目前 GUI 進程記憶體，不寫入 settings.json；CLI 使用 CAUSATIVELENS_LLM_API_KEY 或 --api-key-stdin。",
    "The API key stays in this GUI process and is never written to settings.json. CLI uses CAUSATIVELENS_LLM_API_KEY or --api-key-stdin."
  ],
  "settings.test": [
    "测试配置",
    "測試設定",
    "Test configuration"
  ],
  "settings.save": [
    "保存设置",
    "儲存設定",
    "Save settings"
  ],
  "settings.saved": [
    "设置已保存。API key 仍只在本次运行有效。",
    "設定已儲存。API key 仍只在本次執行有效。",
    "Settings saved. The API key remains valid only for this run."
  ],
  "settings.tested": [
    "配置格式正确；实际 API 调用会在提交辅助分析时发生。",
    "設定格式正確；實際 API 呼叫會在提交輔助分析時發生。",
    "Configuration is valid; the API is called when you submit an analysis."
  ],
  "settings.disabled": [
    "请先启用辅助分析并填写 endpoint 与模型。",
    "請先啟用輔助分析並填寫 endpoint 與模型。",
    "Enable assistance and fill in an endpoint and model first."
  ],
  "about.architecture": [
    "一 daemon，两前端。GUI 通过 RPC 读取同一份证据，轨迹仅在本机处理。",
    "一 daemon，兩前端。GUI 經 RPC 讀取同一份證據，軌跡僅在本機處理。",
    "One daemon, two frontends. RPC shares the evidence; traces are processed locally."
  ],
  "about.boundary": [
    "安装目录并不一定有日志。没有识别到轨迹时，请按模板提示导出到所选目录，或勾选检查软件用户日志。",
    "安裝目錄不一定有日誌。未識別到軌跡時，依範本匯出至所選目錄，或勾選檢查使用者日誌。",
    "Installation folders may contain no traces. Follow the template export instructions or enable checking the product's user logs."
  ],
  "about.persistence": [
    "当前监听视图保存在 daemon 内存中。关闭 GUI 后读取器继续工作；退出 daemon 会清空视图。请及时导出。",
    "監聽檢視儲存於 daemon 記憶體。關閉 GUI 後讀取器繼續工作；退出 daemon 將清空檢視，請及時匯出。",
    "The view lives in daemon memory. Readers continue after the GUI closes; exiting the daemon clears the view. Export records when needed."
  ],
  "footer.note": [
    "留住脚印，不改写旅程。",
    "留住腳印，不改寫旅程。",
    "Keep the footprints. Leave the journey untouched."
  ],
  "detail.title": [
    "轨迹详情",
    "軌跡詳情",
    "Trace details"
  ],
  "detail.boundary": [
    "客户端轨迹不等于系统采集证据。",
    "用戶端軌跡不等於系統擷取證據。",
    "Client traces are not system capture evidence."
  ],
  "detail.copy": [
    "复制 JSON",
    "複製 JSON",
    "Copy JSON"
  ],
  "close": [
    "关闭",
    "關閉",
    "Close"
  ],
  "error.preview": [
    "这是网页预览。请启动桌面客户端后选择本机目录。",
    "這是網頁預覽。請啟動桌面用戶端後選擇本機目錄。",
    "This is a browser preview. Open the desktop app to select local folders."
  ],
  "error.folder": [
    "找不到或无法读取该文件夹。请重新选择有访问权限的目录。",
    "找不到或無法讀取資料夾。請重新選擇有權限的目錄。",
    "Folder not found or unreadable. Choose a folder you can access."
  ],
  "error.connection": [
    "连接失败。请检查 daemon 是否正在运行，然后点击右上角重试。",
    "連線失敗。請檢查 daemon 是否執行中，再點右上角重試。",
    "Connection failed. Check the daemon, then use the top-right retry button."
  ],
  "error.session": [
    "请先开始监听，再执行此操作。",
    "請先開始監聽，再執行此操作。",
    "Start an observation before using this action."
  ],
  "error.operation": [
    "操作没有完成。请查看下方诊断信息后重试。",
    "操作未完成。請查看下方診斷資訊後重試。",
    "The operation did not complete. Inspect the diagnostic details and retry."
  ],
  "copied": [
    "已复制到剪贴板。",
    "已複製到剪貼簿。",
    "Copied to clipboard."
  ]
};

Object.assign(ENTRIES, {
  "nav.observe": ["观测", "觀測", "Observe"], "nav.investigate": ["探查", "探查", "Investigate"], "nav.query": ["查询控制台", "查詢主控台", "Query console"], "nav.compare": ["对比", "對比", "Compare"], "nav.assist": ["辅助", "輔助", "Assist"], "nav.intervention": ["干预", "干預", "Intervention"],
  "sessions.title": ["会话", "工作階段", "Sessions"], "sessions.subtitle": ["每一次观测都是一条可以回到现场的证据链。", "每一次觀測都是一條可以回到現場的證據鏈。", "Every observation is a trace you can return to."], "sessions.new": ["新建观测", "建立觀測", "New observation"], "sessions.count": ["个本机会话", "個本機工作階段", "local sessions"], "sessions.refresh": ["刷新", "重新整理", "Refresh"], "sessions.empty": ["还没有会话", "還沒有工作階段", "No sessions yet"], "sessions.emptyHelp": ["新建一个观测，会话会在这里留下可追溯的入口。", "建立觀測後，工作階段會在這裡留下可追溯入口。", "Start an observation and it will leave a return path here."], "sessions.start": ["开始第一次观测", "開始第一次觀測", "Start first observation"], "sessions.open": ["打开", "開啟", "Open"], "sessions.summary": ["摘要", "摘要", "Summary"],
  "investigate.subtitle": ["断点先于答案出现：从履约断点回到断言、证据和时间。", "斷點先於答案出現：從履約斷點回到斷言、證據和時間。", "Breakpoints come first: return to assertion, evidence and time."], "investigate.graph": ["履约图", "履約圖", "Fulfilment graph"], "investigate.query": ["查询控制台", "查詢主控台", "Query console"], "investigate.breakpoints": ["断点", "斷點", "Breakpoints"], "investigate.noBreakpoints": ["当前没有已识别的偏离。", "目前沒有已識別的偏離。", "No deviations identified."], "investigate.cardTitle": ["选择一个断点", "選擇一個斷點", "Select a breakpoint"], "investigate.cardHelp": ["断点卡会展示触发断言、证据事件、判定、反向归因和健康度。", "斷點卡會展示觸發斷言、證據事件、判定、反向歸因和健康度。", "The card shows trigger, evidence, verdict, attribution and health."], "investigate.trigger": ["触发断言", "觸發斷言", "Trigger assertion"], "investigate.evidence": ["证据事件", "證據事件", "Evidence event"], "investigate.verdict": ["履约判定", "履約判定", "Verdict"], "investigate.health": ["采集健康度", "採集健康度", "Capture health"], "investigate.why": ["Why 查询", "Why 查詢", "Why query"], "investigate.ai": ["AI 分析", "AI 分析", "AI analysis"], "investigate.copy": ["复制引用", "複製引用", "Copy reference"], "investigate.timeline": ["跳到时间线", "跳到時間軸", "Jump to timeline"], "lane.assertions": ["断言泳道", "斷言泳道", "Assertions"], "lane.events": ["系统事件泳道", "系統事件泳道", "System events"],
  "replay.subtitle": ["逐事件重看一条已经发生过的观测，不重新采集。", "逐事件重看一條已發生的觀測，不重新擷取。", "Review an observation without collecting again."], "replay.play": ["播放", "播放", "Play"], "replay.back": ["← 步退", "← 步退", "← Back"], "replay.next": ["步进 →", "步進 →", "Next →"], "replay.query": ["打开查询控制台", "開啟查詢主控台", "Open query console"], "replay.empty": ["暂无可回放事件", "暫無可回放事件", "No events to replay"], "query.title": ["查询控制台", "查詢主控台", "Query console"],
  "compare.subtitle": ["把两次观测的断言、判定和断点并排放在一起。", "把兩次觀測的斷言、判定和斷點並排放在一起。", "Place two observations side by side."], "compare.choose": ["选择会话", "選擇工作階段", "Choose sessions"], "compare.left": ["基线会话", "基線工作階段", "Baseline session"], "compare.right": ["当前会话", "目前工作階段", "Current session"], "compare.empty": ["选择两个会话后显示三向差异。", "選擇兩個工作階段後顯示三向差異。", "Choose two sessions to show the three-way diff."],
  "assist.subtitle": ["可选的大模型只解释已采集证据，引用始终可以跳回泳道。", "可選的大型模型只解釋已擷取證據，引用始終可以跳回泳道。", "An optional model explains captured evidence only."], "assist.run": ["分析当前断点", "分析目前斷點", "Analyze current breakpoint"], "assist.safety": ["辅助层不会改变履约判定，也不会执行干预。", "輔助層不會改變履約判定，也不會執行干預。", "Assist never changes verdicts or performs intervention."], "assist.empty": ["还没有分析记录。", "還沒有分析記錄。", "No analyses yet."], "intervention.subtitle": ["当前版本只有 observe：这里保留位置，不注册命令，不产生写入。", "目前版本只有 observe：這裡保留位置，不註冊命令，不產生寫入。", "Observe-only in this version: reserved space, no writes."], "intervention.title": ["干预中心尚未启用", "干預中心尚未啟用", "Intervention center is not enabled"], "intervention.body": ["Jev 决策后端与 advise/enforce 将在 v0.2.0 通过独立兼容性门禁进入。", "Jev 決策後端與 advise/enforce 將在 v0.2.0 通過獨立相容性門禁進入。", "The Jev decision backend enters through a separate gate in v0.2.0."],
  "menu.file": ["文件", "檔案", "File"], "menu.session": ["会话", "工作階段", "Session"], "menu.analysis": ["分析", "分析", "Analysis"], "menu.replay": ["回放", "回放", "Replay"], "menu.view": ["视图", "檢視", "View"], "menu.tools": ["工具", "工具", "Tools"], "menu.help": ["帮助", "說明", "Help"], "menu.new": ["新建观测…", "建立觀測…", "New observation…"], "menu.open": ["打开会话文件…", "開啟工作階段檔案…", "Open session…"], "menu.attach": ["附加到运行中进程…", "附加到執行中程序…", "Attach to process…"], "menu.export": ["导出", "匯出", "Export"], "menu.close": ["关闭会话", "關閉工作階段", "Close session"], "menu.quit": ["退出", "離開", "Quit"], "menu.list": ["会话列表", "工作階段列表", "Session list"], "menu.refresh": ["刷新", "重新整理", "Refresh"], "menu.stop": ["停止观测", "停止觀測", "Stop observation"], "menu.summary": ["会话摘要", "工作階段摘要", "Session summary"], "menu.why": ["Why 因果链查询", "Why 因果鏈查詢", "Why causal chain"], "menu.find": ["检索断言/事件", "檢索斷言/事件", "Find assertions/events"], "menu.stats": ["会话统计与健康度", "工作階段統計與健康度", "Session stats and health"], "menu.assist": ["AI 辅助分析", "AI 輔助分析", "AI assist"], "menu.diff": ["对比两个会话…", "比較兩個工作階段…", "Compare sessions…"], "menu.play": ["播放/暂停", "播放/暫停", "Play/Pause"], "menu.back": ["步退", "步退", "Step back"], "menu.next": ["步进", "步進", "Step forward"], "menu.break-next": ["下一个断点", "下一個斷點", "Next breakpoint"], "menu.graph": ["履约图", "履約圖", "Fulfilment graph"], "menu.timeline": ["时间轴", "時間軸", "Timeline"], "menu.breakpoints": ["断点列表", "斷點列表", "Breakpoints"], "menu.query": ["查询控制台", "查詢主控台", "Query console"], "menu.deviated": ["只看 Deviated", "只看 Deviated", "Deviated only"], "menu.settings": ["设置", "設定", "Settings"], "menu.doctor": ["适配器医生", "配接器診斷", "Adapter doctor"], "menu.audit": ["审计校验", "稽核驗證", "Audit verify"], "menu.intervention": ["干预中心（规划中）", "干預中心（規劃中）", "Intervention center (planned)"], "menu.quickstart": ["快速入门", "快速入門", "Quick start"], "menu.docs": ["完整文档", "完整文件", "Documentation"], "menu.about": ["关于", "關於", "About"], "menu.openHint": ["打开会话文件请使用 clens open；当前 GUI 保留只读入口。", "開啟工作階段檔案請使用 clens open；目前 GUI 保留唯讀入口。", "Use clens open for a session file; this entry is read-only for now."], "menu.cliHint": ["该诊断仍以 CLI 为权威入口：clens doctor / clens audit。", "此診斷仍以 CLI 為權威入口：clens doctor / clens audit。", "This diagnostic remains CLI-authoritative."], "menu.docsHint": ["文档位于仓库 docs/；桌面版可用帮助菜单打开。", "文件位於倉庫 docs/；桌面版可用說明選單開啟。", "Documentation lives under docs/. "]
});
const locales = ['zh-CN', 'zh-TW', 'en-US'];
const TRANSLATIONS = Object.fromEntries(locales.map((lang, i) => [lang, Object.fromEntries(Object.entries(ENTRIES).map(([key, values]) => [key, values[i]]))]));
function normalizeLanguage(value) { return locales.includes(value) ? value : 'zh-CN'; }
function stored(key) { try { return localStorage.getItem(key); } catch { return null; } }
function translate(key, params = {}) {
  const lang = normalizeLanguage(document.documentElement.lang);
  let value = TRANSLATIONS[lang][key] || TRANSLATIONS['en-US'][key] || key;
  for (const [name, text] of Object.entries(params)) value = value.replaceAll('{' + name + '}', String(text));
  return value;
}
function applyLanguage(value) {
  document.documentElement.lang = normalizeLanguage(value);
  try { localStorage.setItem('causativelens-language', document.documentElement.lang); } catch { /* Storage is optional. */ }
  document.querySelectorAll('[data-i18n]').forEach(el => { el.textContent = translate(el.dataset.i18n); });
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => { el.placeholder = translate(el.dataset.i18nPlaceholder); });
  document.querySelectorAll('[data-i18n-aria-label]').forEach(el => { el.setAttribute('aria-label', translate(el.dataset.i18nAriaLabel)); });
  window.dispatchEvent(new CustomEvent('causativelens-language-changed'));
}
window.causativelensI18n = { applyLanguage, translate, normalizeLanguage, supported: locales };
const selector = document.querySelector('#language-select');
selector.value = normalizeLanguage(stored('causativelens-language') || navigator.language);
selector.addEventListener('change', () => applyLanguage(selector.value));
applyLanguage(selector.value);
