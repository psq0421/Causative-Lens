mod query;
mod replay;
mod terminal;

use clens_types::UiLanguage;

pub use query::{
    parse_restricted_query, render_causal_chain, CausalChain, CausalRecord, QueryIntent,
    StructuredQuery,
};
pub use replay::{replay_overlay, ReplayDiff, ReplayEvent, ReplayOverlay};
pub use terminal::{detect_terminal, render_timeline, TerminalCapabilities, TimelineRow};

pub const OBSERVE_COMMANDS: &[&str] = &[
    "run", "attach", "open", "why", "timeline", "watch", "export", "doctor", "shell", "replay",
];
pub const DISABLED_COMMANDS: &[&str] = &["advise", "enforce", "decision"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Ok = 0,
    Usage = 2,
    Disabled = 4,
    Runtime = 5,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    pub code: ExitCode,
    pub text: String,
}

/// Resolve the human-facing locale without changing the machine-readable
/// JSON contract. `--lang` is accepted before or after the command.
pub fn language_from_args(args: &[String]) -> UiLanguage {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg
            .strip_prefix("--lang=")
            .or_else(|| arg.strip_prefix("--language="))
        {
            if let Some(language) = UiLanguage::parse(value) {
                return language;
            }
        }
        if (arg == "--lang" || arg == "--language") && index + 1 < args.len() {
            if let Some(language) = UiLanguage::parse(&args[index + 1]) {
                return language;
            }
        }
        index += 1;
    }
    UiLanguage::from_environment()
}

fn normalized_args(args: &[String]) -> Vec<String> {
    let mut normalized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--lang" || arg == "--language" {
            index += 2;
            continue;
        }
        if arg.starts_with("--lang=") || arg.starts_with("--language=") {
            index += 1;
            continue;
        }
        normalized.push(arg.clone());
        index += 1;
    }
    normalized
}

/// Executes commands with a direct daemon RPC mapping, with deterministic offline fallback.
pub fn dispatch_live(args: &[String]) -> CliOutput {
    let language = language_from_args(args);
    let args = normalized_args(args);
    let Some(command) = args.first().map(String::as_str) else {
        return dispatch_with_language(&args, language);
    };
    if command == "why" {
        let parsed = parse_restricted_query(&query_argument(&args));
        if parsed.response.hint.is_some() {
            return dispatch_with_language(&args, language);
        }
    }
    let method = match command {
        "doctor" => Some("hello"),
        "open" => Some("session.open"),
        "timeline" | "watch" => Some("session.status"),
        "why" => Some("query.why"),
        _ => None,
    };
    let Some(method) = method else {
        return dispatch_with_language(&args, language);
    };
    let params = if method == "hello" {
        serde_json::json!({"protocol_version": 1, "mode": "observe"})
    } else if command == "why" {
        let parsed = parse_restricted_query(&query_argument(&args));
        let path = parsed
            .query
            .entity
            .path
            .unwrap_or_else(|| query_argument(&args));
        serde_json::json!({"path": path})
    } else {
        serde_json::Value::Null
    };
    let mut client = clens_daemon_client::RpcClient::new();
    let Ok(result) = client.call(method, params) else {
        return dispatch_with_language(&args, language);
    };
    let payload = if command == "doctor" {
        serde_json::json!({"status":"ok","mode":"observe","installation":installation_label(),"daemon":"connected","hello":result})
    } else {
        result
    };
    let text = if args.iter().any(|arg| arg == "--json") {
        payload.to_string()
    } else {
        format!("{command}: {payload}")
    };
    CliOutput {
        code: ExitCode::Ok,
        text,
    }
}

pub fn dispatch(args: &[String]) -> CliOutput {
    dispatch_with_language(&normalized_args(args), language_from_args(args))
}

fn dispatch_with_language(args: &[String], language: UiLanguage) -> CliOutput {
    if args
        .first()
        .is_some_and(|arg| arg == "--version" || arg == "-V")
    {
        return CliOutput {
            code: ExitCode::Ok,
            text: format!(
                "{} {} (observe)",
                clens_types::PRODUCT_NAME,
                clens_types::DEV_VERSION
            ),
        };
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        return CliOutput {
            code: ExitCode::Ok,
            text: usage_for(language),
        };
    }
    let Some(command) = args.first().map(String::as_str) else {
        return CliOutput {
            code: ExitCode::Usage,
            text: usage_for(language),
        };
    };
    if DISABLED_COMMANDS
        .iter()
        .any(|item| command == *item || command.starts_with(&format!("{item}.")))
    {
        return CliOutput {
            code: ExitCode::Disabled,
            text: disabled_message(language),
        };
    }
    if !OBSERVE_COMMANDS.contains(&command) {
        return CliOutput {
            code: ExitCode::Usage,
            text: usage_for(language),
        };
    }
    let json = args.iter().any(|arg| arg == "--json");
    let payload = match command {
        "doctor" => {
            serde_json::json!({"status":"ok","mode":"observe","installation":installation_label()})
                .to_string()
        }
        "timeline" => "{\"events\":[],\"assertions\":[],\"mode\":\"observe\"}".to_owned(),
        "why" => why_payload(args),
        "watch" => "{\"stream\":\"observe\",\"connected\":false}".to_owned(),
        "replay" => replay_payload(args),
        "export" | "open" | "shell" | "run" | "attach" => {
            "{\"status\":\"offline\",\"mode\":\"observe\"}".to_owned()
        }
        _ => "{}".to_owned(),
    };
    let text = if json {
        payload
    } else if command == "timeline" {
        render_timeline(&[], detect_terminal())
    } else if command == "why" {
        human_why(&payload, language)
    } else {
        human_output(command, &payload)
    };
    CliOutput {
        code: ExitCode::Ok,
        text,
    }
}

fn installation_label() -> &'static str {
    let installed = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join("install-manifest.json"))
        })
        .is_some_and(|manifest| manifest.is_file());
    if installed {
        "installed"
    } else {
        "development"
    }
}
fn why_payload(args: &[String]) -> String {
    let query = parse_restricted_query(&query_argument(args));
    serde_json::to_string(&query.response).unwrap_or_else(|_| {
        "{\"query\":\"unsupported\",\"hint\":\"use clens why --path <p>\",\"mode\":\"observe\"}"
            .to_owned()
    })
}

fn replay_payload(args: &[String]) -> String {
    let baseline = replay_input(args, "--baseline");
    let branch = replay_input(args, "--branch");
    match (baseline, branch) {
        (Ok(Some(left)), Ok(Some(right))) => replay_overlay(&left, &right).to_json(),
        (Ok(None), Ok(None)) => replay_overlay(&[], &[]).to_json(),
        (Err(error), _) | (_, Err(error)) => serde_json::json!({
            "mode": "observe",
            "status": "error",
            "error": error
        })
        .to_string(),
        _ => serde_json::json!({
            "mode": "observe",
            "status": "error",
            "error": "both --baseline and --branch JSON event files are required"
        })
        .to_string(),
    }
}

fn replay_input(args: &[String], flag: &str) -> Result<Option<Vec<ReplayEvent>>, String> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };
    let Some(path) = args.get(index + 1) else {
        return Err(format!("missing path after {flag}"));
    };
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| error.to_string())
}
fn query_argument(args: &[String]) -> String {
    let mut values = Vec::new();
    let mut index = 1usize;
    while index < args.len() {
        if args[index] == "--json" {
            index += 1;
            continue;
        }
        if args[index] == "--path" {
            if let Some(path) = args.get(index + 1) {
                return format!("why --path {path}");
            }
            return String::new();
        }
        values.push(args[index].as_str());
        index += 1;
    }
    values.join(" ")
}
fn human_output(command: &str, payload: &str) -> String {
    format!("{command}: {payload}")
}

fn human_why(payload: &str, language: UiLanguage) -> String {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return format!("why: {payload}");
    };
    let Some(chain) = value.get("chain") else {
        return unsupported_message(language, payload);
    };
    let trigger = chain
        .get("trigger_assertion")
        .and_then(|item| item.get("raw_quote"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("(none)");
    let verdict = chain
        .get("verdict")
        .and_then(|item| item.get("state"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unobserved");
    let evidence_count = chain
        .get("evidence_events")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let health = chain
        .get("confidence_health")
        .and_then(|item| item.get("note"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    match language {
        UiLanguage::EnglishUs => format!(
            "why\n1. trigger assertion: {trigger}\n2. evidence events: {evidence_count}\n3. verdict: {verdict}\n4. reverse attribution: available in --json\n5. confidence/health: {health}"
        ),
        UiLanguage::SimplifiedChinese => format!(
            "为什么\n1. 触发断言：{trigger}\n2. 证据事件：{evidence_count}\n3. 履约判定：{verdict}\n4. 反向归因：请使用 --json 查看\n5. 置信度/健康度：{health}"
        ),
        UiLanguage::TraditionalChinese => format!(
            "為什麼\n1. 觸發斷言：{trigger}\n2. 證據事件：{evidence_count}\n3. 履約判定：{verdict}\n4. 反向歸因：請使用 --json 查看\n5. 信心度/健康度：{health}"
        ),
    }
}
fn usage_for(language: UiLanguage) -> String {
    let commands = OBSERVE_COMMANDS.join("|");
    match language {
        UiLanguage::EnglishUs => format!(
            "\n  CausativeLens {} / BETA\n  ----------------------------------------\n  Keep the footprints. Leave the journey untouched.\n\n  Usage: clens <{commands}>\n         [--json] [--lang en-US|zh-CN|zh-TW]\n\n  Start here\n    clens --version          Show the installed version\n    clens doctor --json      Installation / daemon diagnostics\n    clens-gui                Open the listening wizard\n\n  v0.1.1 listens to agent trace files through the GUI.\n  run / attach / shell are not connected to live capture yet.\n  Mode: observe / local processing\n",
            clens_types::DEV_VERSION
        ),
        UiLanguage::SimplifiedChinese => format!(
            "\n  CausativeLens {} / BETA\n  ----------------------------------------\n  留住脚印，不改写旅程。\n\n  用法：clens <{commands}>\n        [--json] [--lang en-US|zh-CN|zh-TW]\n\n  从这里开始\n    clens --version          查看安装版本\n    clens doctor --json      安装与 daemon 诊断\n    clens-gui                打开监听向导\n\n  v0.1.1 通过 GUI 监听 Agent 轨迹文件。\n  run / attach / shell 尚未接通常驻采集。\n  模式：observe / 本地处理\n",
            clens_types::DEV_VERSION
        ),
        UiLanguage::TraditionalChinese => format!(
            "\n  CausativeLens {} / BETA\n  ----------------------------------------\n  留住腳印，不改寫旅程。\n\n  用法：clens <{commands}>\n        [--json] [--lang en-US|zh-CN|zh-TW]\n\n  從這裡開始\n    clens --version          查看安裝版本\n    clens doctor --json      安裝與 daemon 診斷\n    clens-gui                開啟監聽精靈\n\n  v0.1.1 透過 GUI 監聽 Agent 軌跡檔案。\n  run / attach / shell 尚未接通常駐擷取。\n  模式：observe / 本機處理\n",
            clens_types::DEV_VERSION
        ),
    }
}

fn disabled_message(language: UiLanguage) -> String {
    match language {
        UiLanguage::EnglishUs => {
            "This command belongs to the v0.2.0 intervention/decision layer; v0.1.0 is observe-only.".to_owned()
        }
        UiLanguage::SimplifiedChinese => {
            "该命令属于 v0.2.0 干预/决策层；v0.1.0 仅支持 observe。".to_owned()
        }
        UiLanguage::TraditionalChinese => {
            "此命令屬於 v0.2.0 介入/決策層；v0.1.0 僅支援 observe。".to_owned()
        }
    }
}

fn unsupported_message(language: UiLanguage, payload: &str) -> String {
    match language {
        UiLanguage::EnglishUs => format!("why: {payload}"),
        UiLanguage::SimplifiedChinese => format!("为什么：{payload}"),
        UiLanguage::TraditionalChinese => format!("為什麼：{payload}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }
    #[test]
    fn disabled_commands_are_not_registered() {
        let result = dispatch(&args(&["advise"]));
        assert_eq!(result.code, ExitCode::Disabled);
    }
    #[test]
    fn doctor_supports_machine_json() {
        let result = dispatch(&args(&["doctor", "--json"]));
        assert_eq!(result.code, ExitCode::Ok);
        assert!(result.text.contains("\"status\":\"ok\""));
    }
    #[test]
    fn why_uses_restricted_path_route() {
        let result = dispatch(&args(&["why", "--path", "config.yaml", "--json"]));
        assert!(result.text.contains("\"intent\":\"why\""));
        assert!(result.text.contains("config.yaml"));
    }
    #[test]
    fn unsupported_nl_query_has_deterministic_hint() {
        let result = dispatch(&args(&["why", "随便问点什么", "--json"]));
        assert!(result.text.contains("clens why --path <p>"));
    }
    #[test]
    fn human_why_uses_the_five_section_labels() {
        let result = dispatch(&args(&["--lang", "en-US", "why", "--path", "config.yaml"]));
        for label in [
            "trigger assertion",
            "evidence events",
            "verdict",
            "reverse attribution",
            "confidence/health",
        ] {
            assert!(result.text.contains(label));
        }
    }
    #[test]
    fn replay_requires_a_pair_of_observed_event_files() {
        let result = dispatch(&args(&["replay", "--json", "--baseline", "baseline.json"]));
        assert_eq!(result.code, ExitCode::Ok);
        assert!(
            result.text.contains("No such file")
                || result.text.contains("cannot find")
                || result.text.contains("找不到")
        );
    }
    #[test]
    fn usage_is_utf8_and_observe_only() {
        let result = dispatch(&[]);
        assert!(result.text.contains("用法"));
        assert!(!result.text.contains('�'));
    }
    #[test]
    fn language_flag_localizes_human_output_without_changing_json_shape() {
        let result = dispatch(&args(&["--lang", "en-US", "why", "--path", "config.yaml"]));
        assert!(result.text.contains("trigger assertion"));
        let traditional = dispatch(&args(&["--lang=zh-TW", "advise"]));
        assert!(traditional.text.contains("僅支援 observe"));
    }
    #[test]
    fn help_uses_selected_language() {
        let result = dispatch(&args(&["--lang", "en-US", "--help"]));
        assert_eq!(result.code, ExitCode::Ok);
        assert!(result.text.contains("Usage:"));
    }
}
