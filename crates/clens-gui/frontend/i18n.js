const TRANSLATIONS = {
  "zh-CN": {
    "language.label": "语言",
    "theme.toggle": "切换主题",
    "help.open": "打开帮助",
    "workspace.nav": "工作区",
    "nav.sessions": "Sessions",
    "nav.timeline": "Timeline",
    "nav.graph": "Graph",
    "nav.replay": "Replay / Query",
    "nav.about": "关于",
    "command.search": "搜索命令或会话",
    "connection.disconnected": "daemon 未连接",
    "connection.connected": "daemon 已连接",
    "connection.offline": "离线模式",
    "connection.waiting": "等待连接",
    "sessions.subtitle": "选择一个观测会话，查看它的实时状态。",
    "sessions.open": "打开会话",
    "summary.mode": "当前模式",
    "summary.readonly": "只读观测",
    "summary.capture": "采集状态",
    "summary.sources": "ETW 与 Harness",
    "summary.compatibility": "兼容性",
    "summary.unknown": "未知版本不阻断",
    "sessions.empty.title": "还没有打开会话",
    "sessions.empty.body": "使用 CLI 的 <code>clens run</code> 或点击上面的按钮开始一次只读观测。",
    "sessions.copyRun": "复制 clens run 命令",
    "timeline.subtitle": "断言和系统事件按水位线增量到达。",
    "timeline.virtualized": "窗口化渲染",
    "filter.all": "全部",
    "filter.assertions": "断言",
    "filter.files": "文件",
    "filter.network": "网络",
    "timeline.domLimit": "DOM 上限 500",
    "graph.subtitle": "履约关系和因果断点。",
    "graph.canvas": "履约图",
    "graph.waiting": "等待 graph.delta",
    "replay.subtitle": "回放、why 查询和只读导出。",
    "replay.copyWhy": "复制对应 CLI 命令",
    "query.label": "受限查询",
    "query.placeholder": "例如：为什么它修改了 config.yaml？",
    "query.run": "查询",
    "query.hint": "查询使用确定性模板；无法识别时会提示对应的 <code>clens why --path</code> 命令。",
    "about.subtitle": "实时、只读、可回放的因果观测。",
    "about.architecture": "一 daemon，两前端。GUI 通过 causativelens-rpc 访问 daemon，不复制引擎逻辑。",
    "about.compatibility": "兼容性：未知 Agent 版本使用 Tier3 fallback，不强制阻断。",
    "palette.label": "命令搜索",
    "palette.placeholder": "输入命令…",
    "palette.sessions": "打开 Sessions",
    "palette.timeline": "打开 Timeline",
    "palette.graph": "打开 Graph",
    "palette.replay": "打开 Replay / Query"
  },
  "zh-TW": {
    "language.label": "語言",
    "theme.toggle": "切換主題",
    "help.open": "開啟說明",
    "workspace.nav": "工作區",
    "nav.sessions": "Sessions",
    "nav.timeline": "Timeline",
    "nav.graph": "Graph",
    "nav.replay": "Replay / Query",
    "nav.about": "關於",
    "command.search": "搜尋命令或工作階段",
    "connection.disconnected": "daemon 未連線",
    "connection.connected": "daemon 已連線",
    "connection.offline": "離線模式",
    "connection.waiting": "等待連線",
    "sessions.subtitle": "選擇觀測工作階段，查看即時狀態。",
    "sessions.open": "開啟工作階段",
    "summary.mode": "目前模式",
    "summary.readonly": "唯讀觀測",
    "summary.capture": "擷取狀態",
    "summary.sources": "ETW 與 Harness",
    "summary.compatibility": "相容性",
    "summary.unknown": "未知版本不阻斷",
    "sessions.empty.title": "尚未開啟工作階段",
    "sessions.empty.body": "使用 CLI 的 <code>clens run</code> 或點擊上方按鈕開始唯讀觀測。",
    "sessions.copyRun": "複製 clens run 命令",
    "timeline.subtitle": "斷言與系統事件依水位線增量抵達。",
    "timeline.virtualized": "視窗化渲染",
    "filter.all": "全部",
    "filter.assertions": "斷言",
    "filter.files": "檔案",
    "filter.network": "網路",
    "timeline.domLimit": "DOM 上限 500",
    "graph.subtitle": "履約關係與因果斷點。",
    "graph.canvas": "履約圖",
    "graph.waiting": "等待 graph.delta",
    "replay.subtitle": "回放、why 查詢與唯讀匯出。",
    "replay.copyWhy": "複製對應 CLI 命令",
    "query.label": "受限查詢",
    "query.placeholder": "例如：為什麼它修改了 config.yaml？",
    "query.run": "查詢",
    "query.hint": "查詢使用確定性範本；無法識別時會提示對應的 <code>clens why --path</code> 命令。",
    "about.subtitle": "即時、唯讀、可回放的因果觀測。",
    "about.architecture": "一個 daemon，兩個前端。GUI 透過 causativelens-rpc 存取 daemon，不複製引擎邏輯。",
    "about.compatibility": "相容性：未知 Agent 版本使用 Tier3 fallback，不強制阻斷。",
    "palette.label": "命令搜尋",
    "palette.placeholder": "輸入命令…",
    "palette.sessions": "開啟 Sessions",
    "palette.timeline": "開啟 Timeline",
    "palette.graph": "開啟 Graph",
    "palette.replay": "開啟 Replay / Query"
  },
  "en-US": {
    "language.label": "Language",
    "theme.toggle": "Toggle theme",
    "help.open": "Open help",
    "workspace.nav": "Workspace",
    "nav.sessions": "Sessions",
    "nav.timeline": "Timeline",
    "nav.graph": "Graph",
    "nav.replay": "Replay / Query",
    "nav.about": "About",
    "command.search": "Search commands or sessions",
    "connection.disconnected": "daemon disconnected",
    "connection.connected": "daemon connected",
    "connection.offline": "offline mode",
    "connection.waiting": "waiting for connection",
    "sessions.subtitle": "Choose an observation session to see its live state.",
    "sessions.open": "Open session",
    "summary.mode": "Current mode",
    "summary.readonly": "Read-only observation",
    "summary.capture": "Capture status",
    "summary.sources": "ETW and Harness",
    "summary.compatibility": "Compatibility",
    "summary.unknown": "Unknown versions never block",
    "sessions.empty.title": "No session is open",
    "sessions.empty.body": "Use the CLI <code>clens run</code> or click the button above to begin a read-only observation.",
    "sessions.copyRun": "Copy clens run command",
    "timeline.subtitle": "Assertions and system events arrive incrementally at the watermark.",
    "timeline.virtualized": "Virtualized rendering",
    "filter.all": "All",
    "filter.assertions": "Assertions",
    "filter.files": "Files",
    "filter.network": "Network",
    "timeline.domLimit": "DOM limit 500",
    "graph.subtitle": "Fulfilment relationships and causal breakpoints.",
    "graph.canvas": "Fulfilment graph",
    "graph.waiting": "Waiting for graph.delta",
    "replay.subtitle": "Replay, why queries, and read-only export.",
    "replay.copyWhy": "Copy matching CLI command",
    "query.label": "Restricted query",
    "query.placeholder": "For example: why did it modify config.yaml?",
    "query.run": "Query",
    "query.hint": "Queries use deterministic templates; unsupported text falls back to <code>clens why --path</code>.",
    "about.subtitle": "Live, read-only, replayable causal observation.",
    "about.architecture": "One daemon, two frontends. The GUI uses causativelens-rpc and never copies engine logic.",
    "about.compatibility": "Compatibility: unknown Agent versions use the Tier3 fallback and never block.",
    "palette.label": "Command search",
    "palette.placeholder": "Type a command…",
    "palette.sessions": "Open Sessions",
    "palette.timeline": "Open Timeline",
    "palette.graph": "Open Graph",
    "palette.replay": "Open Replay / Query"
  }
};

function normalizeLanguage(value) {
  return Object.prototype.hasOwnProperty.call(TRANSLATIONS, value) ? value : 'zh-CN';
}

function translate(key) {
  const language = document.documentElement.lang || 'zh-CN';
  return TRANSLATIONS[normalizeLanguage(language)][key] || TRANSLATIONS['en-US'][key] || key;
}

function applyLanguage(value) {
  const language = normalizeLanguage(value);
  document.documentElement.lang = language;
  localStorage.setItem('causativelens-language', language);
  document.querySelectorAll('[data-i18n]').forEach(element => { element.innerHTML = translate(element.dataset.i18n); });
  document.querySelectorAll('[data-i18n-placeholder]').forEach(element => { element.placeholder = translate(element.dataset.i18nPlaceholder); });
  document.querySelectorAll('[data-i18n-aria-label]').forEach(element => { element.setAttribute('aria-label', translate(element.dataset.i18nAriaLabel)); });
  window.dispatchEvent(new CustomEvent('causativelens-language-changed', { detail: language }));
}

window.causativelensI18n = { applyLanguage, translate, normalizeLanguage, supported: Object.keys(TRANSLATIONS) };
const initialLanguage = normalizeLanguage(localStorage.getItem('causativelens-language') || (navigator.language || 'zh-CN'));
const selector = document.querySelector('#language-select');
if (selector) {
  selector.value = initialLanguage;
  selector.addEventListener('change', () => applyLanguage(selector.value));
}
applyLanguage(initialLanguage);
