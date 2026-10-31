//! Optional, evidence-grounded language-model assistance.
//!
//! The assistant is deliberately downstream of deterministic observation. It
//! receives a bounded causal chain, never opens a session or changes a verdict,
//! and must cite assertion ids or event sequences when it makes a claim.

use clens_types::PRODUCT_SLUG;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const SETTINGS_FILE: &str = "settings.json";
pub const DEFAULT_MAX_TOKENS: u32 = 1_200;
pub const DEFAULT_TIMEOUT_SECS: u64 = 45;
pub const MAX_PROMPT_BYTES: usize = 48 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssistantProvider {
    Jev,
    OpenAiCompatible,
    Anthropic,
    Ollama,
}

impl AssistantProvider {
    pub const ALL: [Self; 4] = [
        Self::Jev,
        Self::OpenAiCompatible,
        Self::Anthropic,
        Self::Ollama,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Jev => "Jev",
            Self::OpenAiCompatible => "OpenAI-compatible",
            Self::Anthropic => "Anthropic-compatible",
            Self::Ollama => "Ollama (local)",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "jev" => Some(Self::Jev),
            "openai" | "openai-compatible" | "openai_compatible" => Some(Self::OpenAiCompatible),
            "anthropic" | "anthropic-compatible" | "anthropic_compatible" => Some(Self::Anthropic),
            "ollama" | "local" => Some(Self::Ollama),
            _ => None,
        }
    }

    const fn wire_name(self) -> &'static str {
        match self {
            Self::Jev => "jev",
            Self::OpenAiCompatible => "openai-compatible",
            Self::Anthropic => "anthropic",
            Self::Ollama => "ollama",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_provider")]
    pub provider: AssistantProvider,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_true")]
    pub redact_paths: bool,
    #[serde(default)]
    pub include_raw_quotes: bool,
    #[serde(default)]
    pub auto_analyze: bool,
    #[serde(default = "default_refresh_ms")]
    pub timeline_refresh_ms: u64,
    #[serde(default = "default_retention")]
    pub timeline_retention: usize,
    #[serde(default = "default_theme")]
    pub default_theme: String,
    #[serde(default)]
    pub default_language: String,
}

fn default_provider() -> AssistantProvider {
    AssistantProvider::Jev
}
fn default_max_tokens() -> u32 {
    DEFAULT_MAX_TOKENS
}
fn default_temperature() -> f32 {
    0.2
}
fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}
fn default_true() -> bool {
    true
}
fn default_refresh_ms() -> u64 {
    500
}
fn default_retention() -> usize {
    5_000
}
fn default_theme() -> String {
    "system".to_owned()
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AssistantProvider::Jev,
            endpoint: String::new(),
            model: String::new(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: 0.2,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            redact_paths: true,
            include_raw_quotes: false,
            auto_analyze: false,
            timeline_refresh_ms: 500,
            timeline_retention: 5_000,
            default_theme: "system".to_owned(),
            default_language: String::new(),
        }
    }
}

impl AssistantConfig {
    pub fn normalized(&self) -> Result<Self, AssistError> {
        let mut config = self.clone();
        config.endpoint = config.endpoint.trim().to_owned();
        config.model = config.model.trim().to_owned();
        if config.enabled && config.endpoint.is_empty() {
            return Err(AssistError::Configuration(
                "assistant endpoint is required when assistance is enabled".to_owned(),
            ));
        }
        if config.enabled && config.model.is_empty() {
            return Err(AssistError::Configuration(
                "assistant model is required when assistance is enabled".to_owned(),
            ));
        }
        if config.endpoint.len() > 2_048 {
            return Err(AssistError::Configuration(
                "assistant endpoint is too long".to_owned(),
            ));
        }
        if config.enabled && !valid_endpoint(&config.endpoint) {
            return Err(AssistError::Configuration(
                "assistant endpoint must use http:// or https://".to_owned(),
            ));
        }
        config.max_tokens = config.max_tokens.clamp(128, 16_384);
        config.temperature = config.temperature.clamp(0.0, 2.0);
        config.timeout_secs = config.timeout_secs.clamp(5, 300);
        config.timeline_refresh_ms = config.timeline_refresh_ms.clamp(250, 5_000);
        config.timeline_retention = config.timeline_retention.clamp(500, 50_000);
        if !matches!(config.default_theme.as_str(), "system" | "light" | "dark") {
            config.default_theme = "system".to_owned();
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantRequest {
    pub query: String,
    pub chain: Value,
    #[serde(default)]
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantFinding {
    pub title: String,
    pub severity: String,
    pub explanation: String,
    #[serde(default)]
    pub assertion_ids: Vec<u64>,
    #[serde(default)]
    pub event_seqs: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssistantResult {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    pub summary: String,
    #[serde(default)]
    pub findings: Vec<AssistantFinding>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
    pub grounded: bool,
}

#[derive(Debug, Error)]
pub enum AssistError {
    #[error("assistant is disabled")]
    Disabled,
    #[error("assistant configuration error: {0}")]
    Configuration(String),
    #[error("assistant request failed: {0}")]
    Request(String),
    #[error("assistant response was invalid: {0}")]
    Response(String),
    #[error("assistant settings I/O failed: {0}")]
    SettingsIo(String),
}

pub fn settings_path() -> PathBuf {
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    root.join(PRODUCT_SLUG).join(SETTINGS_FILE)
}

pub fn load_config() -> Result<AssistantConfig, AssistError> {
    load_config_from(settings_path())
}

pub fn load_config_from(path: impl AsRef<Path>) -> Result<AssistantConfig, AssistError> {
    let path = path.as_ref();
    if !path.is_file() {
        return Ok(AssistantConfig::default());
    }
    let bytes = fs::read(path).map_err(|error| AssistError::SettingsIo(error.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|error| AssistError::SettingsIo(error.to_string()))
}

pub fn save_config(config: &AssistantConfig) -> Result<PathBuf, AssistError> {
    let path = settings_path();
    save_config_to(config, &path)?;
    Ok(path)
}

pub fn save_config_to(config: &AssistantConfig, path: impl AsRef<Path>) -> Result<(), AssistError> {
    let config = config.normalized()?;
    let path = path.as_ref();
    let parent = path
        .parent()
        .ok_or_else(|| AssistError::SettingsIo("settings parent is missing".to_owned()))?;
    fs::create_dir_all(parent).map_err(|error| AssistError::SettingsIo(error.to_string()))?;
    let temp = path.with_extension("json.tmp");
    let body = serde_json::to_vec_pretty(&config)
        .map_err(|error| AssistError::SettingsIo(error.to_string()))?;
    fs::write(&temp, body).map_err(|error| AssistError::SettingsIo(error.to_string()))?;
    // Windows rename does not replace an existing destination. The temp file
    // is fully written before removing the old settings so repeated GUI saves
    // work reliably on both Windows and Unix.
    if path.exists() {
        fs::remove_file(path).map_err(|error| AssistError::SettingsIo(error.to_string()))?;
    }
    fs::rename(&temp, path).map_err(|error| AssistError::SettingsIo(error.to_string()))
}

pub fn analyze(
    config: &AssistantConfig,
    api_key: Option<&str>,
    request: &AssistantRequest,
) -> Result<AssistantResult, AssistError> {
    let config = config.normalized()?;
    if !config.enabled {
        return Err(AssistError::Disabled);
    }
    let prompt = build_prompt(&config, request)?;
    let response_text = match config.provider {
        AssistantProvider::Anthropic => request_anthropic(&config, api_key, &prompt)?,
        AssistantProvider::Jev if config.endpoint.starts_with("local://") => {
            request_jev_local(&config, &prompt)?
        }
        AssistantProvider::Jev
        | AssistantProvider::OpenAiCompatible
        | AssistantProvider::Ollama => request_openai_compatible(&config, api_key, &prompt)?,
    };
    parse_result(&config, &response_text, &request.chain)
}

fn build_prompt(
    config: &AssistantConfig,
    request: &AssistantRequest,
) -> Result<String, AssistError> {
    let mut chain = request.chain.clone();
    if config.redact_paths {
        redact_value(&mut chain, false);
    }
    if !config.include_raw_quotes {
        remove_key_recursive(&mut chain, "raw_quote");
    }
    let serialized =
        serde_json::to_string(&chain).map_err(|error| AssistError::Request(error.to_string()))?;
    let language = if request.language.trim().is_empty() {
        "en-US"
    } else {
        request.language.trim()
    };
    let prompt = format!(
        "Analyze this CausativeLens observe-only causal chain. Never invent an event, assertion, process, or timestamp. Every finding must cite existing assertion_ids or event_seqs; if evidence is missing, say unknown. Do not recommend intervention or enforcement. Return JSON only with keys summary, findings (title, severity, explanation, assertion_ids, event_seqs), next_steps, caveats, grounded. Respond in {language}.\n\nUser query: {}\nCausal chain JSON: {}",
        request.query.trim(), serialized
    );
    if prompt.len() > MAX_PROMPT_BYTES {
        return Err(AssistError::Request(
            "evidence prompt exceeds 48 KiB".to_owned(),
        ));
    }
    Ok(prompt)
}

fn request_openai_compatible(
    config: &AssistantConfig,
    api_key: Option<&str>,
    prompt: &str,
) -> Result<String, AssistError> {
    let body = json!({
        "model": config.model,
        "temperature": config.temperature,
        "max_tokens": config.max_tokens,
        "messages": [
            {"role": "system", "content": "You are Jev, an evidence-grounded causal analyst for CausativeLens."},
            {"role": "user", "content": prompt}
        ]
    });
    let mut headers = vec!["Content-Type: application/json".to_owned()];
    if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
        if key.chars().any(char::is_control) {
            return Err(AssistError::Request(
                "API key contains a line break and was rejected".to_owned(),
            ));
        }
        headers.push(format!("Authorization: Bearer {key}"));
    }
    let body = post_json(&config.endpoint, &headers, &body, config.timeout_secs)?;
    body.get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .or_else(|| {
            body.get("message")
                .and_then(|message| message.get("content"))
                .and_then(Value::as_str)
        })
        .ok_or_else(|| AssistError::Response("missing choices[0].message.content".to_owned()))
        .map(ToOwned::to_owned)
}

/// Run a user-selected local Jev-compatible process. The process receives a
/// single JSON request on stdin and returns either the AssistantResult JSON or
/// a plain-text answer on stdout. This keeps the CLI useful on an air-gapped
/// workstation without embedding a model runtime in the daemon.
fn request_jev_local(config: &AssistantConfig, prompt: &str) -> Result<String, AssistError> {
    let command = std::env::var("CAUSATIVELENS_JEV_COMMAND").map_err(|_| {
        AssistError::Configuration(
            "local Jev requires CAUSATIVELENS_JEV_COMMAND (a local model runner)".to_owned(),
        )
    })?;
    if command.trim().is_empty() || command.chars().any(char::is_control) {
        return Err(AssistError::Configuration(
            "CAUSATIVELENS_JEV_COMMAND is invalid".to_owned(),
        ));
    }
    let request = json!({"model": config.model, "prompt": prompt, "max_tokens": config.max_tokens, "temperature": config.temperature});
    let mut child = if cfg!(windows) {
        let mut command_line = Command::new("cmd.exe");
        command_line.args(["/D", "/S", "/C", &command]);
        command_line
    } else {
        let mut command_line = Command::new("sh");
        command_line.args(["-lc", &command]);
        command_line
    }
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| AssistError::Request(format!("local Jev runner failed to start: {error}")))?;
    if let Some(mut stdin) = child.stdin.take() {
        let bytes = serde_json::to_vec(&request)
            .map_err(|error| AssistError::Request(error.to_string()))?;
        stdin
            .write_all(&bytes)
            .map_err(|error| AssistError::Request(error.to_string()))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| AssistError::Request(error.to_string()))?;
    if !output.status.success() {
        return Err(AssistError::Request(format!(
            "local Jev runner exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|error| AssistError::Response(error.to_string()))
}

fn request_anthropic(
    config: &AssistantConfig,
    api_key: Option<&str>,
    prompt: &str,
) -> Result<String, AssistError> {
    let body = json!({
        "model": config.model,
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
        "system": "You are an evidence-grounded causal analyst for CausativeLens.",
        "messages": [{"role": "user", "content": prompt}]
    });
    let mut headers = vec![
        "Content-Type: application/json".to_owned(),
        "anthropic-version: 2023-06-01".to_owned(),
    ];
    if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
        if key.chars().any(char::is_control) {
            return Err(AssistError::Request(
                "API key contains a line break and was rejected".to_owned(),
            ));
        }
        headers.push(format!("x-api-key: {key}"));
    }
    let body = post_json(&config.endpoint, &headers, &body, config.timeout_secs)?;
    body.get("content")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| AssistError::Response("missing content[0].text".to_owned()))
        .map(ToOwned::to_owned)
}

/// Use the Windows-provided curl executable for the optional network path.
/// Keeping this out of the core dependency graph preserves the small offline
/// CLI/GUI build. Headers, including the API key, are supplied through a
/// short-lived per-user file rather than the process command line and are
/// removed after the request.
fn post_json(
    endpoint: &str,
    headers: &[String],
    body: &Value,
    timeout_secs: u64,
) -> Result<Value, AssistError> {
    let payload =
        serde_json::to_vec(body).map_err(|error| AssistError::Request(error.to_string()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let header_path = std::env::temp_dir().join(format!(
        "causativelens-assist-{}-{nonce}.headers",
        std::process::id()
    ));
    let mut header_text = headers.join("\n");
    header_text.push_str("\nAccept: application/json\n");
    fs::write(&header_path, header_text).map_err(|error| {
        AssistError::Request(format!("cannot prepare request headers: {error}"))
    })?;
    let header_arg = format!("@{}", header_path.display());
    let mut command = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    command
        .args(["--silent", "--show-error", "--fail-with-body", "--max-time"])
        .arg(timeout_secs.to_string())
        .args(["--connect-timeout", "10", "-X", "POST"]);
    command.args(["-H", &header_arg, "-d", "@-", endpoint]);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = fs::remove_file(&header_path);
            return Err(AssistError::Request(format!(
                "curl is unavailable: {error}"
            )));
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(&payload) {
            let _ = fs::remove_file(&header_path);
            return Err(AssistError::Request(error.to_string()));
        }
    }
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            let _ = fs::remove_file(&header_path);
            return Err(AssistError::Request(error.to_string()));
        }
    };
    let _ = fs::remove_file(&header_path);
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        let response = String::from_utf8_lossy(&output.stdout);
        return Err(AssistError::Request(format!(
            "curl failed ({}): {}{}",
            output.status,
            error.trim(),
            if response.trim().is_empty() {
                String::new()
            } else {
                format!(" {}", response.trim().chars().take(500).collect::<String>())
            }
        )));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| AssistError::Response(format!("invalid JSON from provider: {error}")))
}

fn parse_result(
    config: &AssistantConfig,
    text: &str,
    chain: &Value,
) -> Result<AssistantResult, AssistError> {
    let candidate = text.trim();
    if let Ok(mut result) = serde_json::from_str::<AssistantResult>(candidate) {
        result.provider = config.provider.wire_name().to_owned();
        result.model = config.model.clone();
        let (assertion_ids, event_seqs) = evidence_ids(chain);
        let mut invalid_citations = 0usize;
        for finding in &result.findings {
            let has_citation = !finding.assertion_ids.is_empty() || !finding.event_seqs.is_empty();
            let citations_exist = finding
                .assertion_ids
                .iter()
                .all(|id| assertion_ids.contains(id))
                && finding
                    .event_seqs
                    .iter()
                    .all(|seq| event_seqs.contains(seq));
            if !has_citation || !citations_exist {
                invalid_citations = invalid_citations.saturating_add(1);
            }
        }
        if invalid_citations > 0 {
            result.caveats.push(format!(
                "{invalid_citations} finding(s) cited an assertion/event outside the submitted evidence; treat them as ungrounded."
            ));
        }
        result.grounded = invalid_citations == 0;
        return Ok(result);
    }
    Ok(AssistantResult {
        provider: config.provider.wire_name().to_owned(),
        model: config.model.clone(),
        summary: candidate.chars().take(4_000).collect(),
        findings: Vec::new(),
        next_steps: vec!["Review the cited causal chain in the Timeline and Graph views.".to_owned()],
        caveats: vec!["The provider did not return the requested JSON shape; the text was kept as an unstructured note.".to_owned()],
        grounded: false,
    })
}

fn valid_endpoint(endpoint: &str) -> bool {
    let lower = endpoint.trim().to_ascii_lowercase();
    if lower == "local://jev" {
        return true;
    }
    let Some(authority) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return false;
    };
    let Some(host) = authority.split('/').next() else {
        return false;
    };
    !host.is_empty()
        && !host.starts_with(':')
        && !host.ends_with(':')
        && !host.chars().any(|character| character.is_whitespace())
        && !endpoint.chars().any(|character| character.is_control())
}

fn evidence_ids(
    chain: &Value,
) -> (
    std::collections::HashSet<u64>,
    std::collections::HashSet<u64>,
) {
    let mut assertion_ids = std::collections::HashSet::new();
    let mut event_seqs = std::collections::HashSet::new();
    collect_evidence_ids(chain, &mut assertion_ids, &mut event_seqs);
    (assertion_ids, event_seqs)
}

fn collect_evidence_ids(
    value: &Value,
    assertion_ids: &mut std::collections::HashSet<u64>,
    event_seqs: &mut std::collections::HashSet<u64>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key == "assertion_id" || key == "assertion_ids" {
                    collect_numbers(child, assertion_ids);
                } else if key == "event_seq" || key == "event_seqs" || key == "seq" {
                    collect_numbers(child, event_seqs);
                } else if key == "trigger_assertion" {
                    if let Some(id) = child.get("id").and_then(Value::as_u64) {
                        assertion_ids.insert(id);
                    }
                }
                collect_evidence_ids(child, assertion_ids, event_seqs);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_evidence_ids(child, assertion_ids, event_seqs);
            }
        }
        _ => {}
    }
}

fn collect_numbers(value: &Value, output: &mut std::collections::HashSet<u64>) {
    match value {
        Value::Number(number) => {
            if let Some(value) = number.as_u64() {
                output.insert(value);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_numbers(item, output);
            }
        }
        _ => {}
    }
}

fn remove_key_recursive(value: &mut Value, key: &str) {
    match value {
        Value::Object(map) => {
            map.remove(key);
            for child in map.values_mut() {
                remove_key_recursive(child, key);
            }
        }
        Value::Array(items) => {
            for child in items {
                remove_key_recursive(child, key);
            }
        }
        _ => {}
    }
}

fn redact_value(value: &mut Value, inside_path: bool) {
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(child) = map.get_mut(&key) {
                    redact_value(child, key.contains("path") || key == "target");
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                redact_value(child, inside_path);
            }
        }
        Value::String(text) if inside_path && !text.trim().is_empty() => {
            let hash = stable_hash(text);
            *text = format!("<path:{hash:016x}>");
        }
        _ => {}
    }
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_default_and_settings_never_contain_a_secret_field() {
        let config = AssistantConfig::default();
        assert!(!config.enabled);
        let json = serde_json::to_string(&config).expect("config json");
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn provider_aliases_include_jev_and_local_ollama() {
        assert_eq!(
            AssistantProvider::parse("Jev"),
            Some(AssistantProvider::Jev)
        );
        assert_eq!(
            AssistantProvider::parse("local"),
            Some(AssistantProvider::Ollama)
        );
    }

    #[test]
    fn prompt_redacts_paths_and_raw_quotes_by_default() {
        let config = AssistantConfig {
            enabled: true,
            endpoint: "https://example.invalid/v1/chat/completions".to_owned(),
            model: "jev-test".to_owned(),
            ..AssistantConfig::default()
        };
        let request = AssistantRequest {
            query: "why".to_owned(),
            chain: json!({"target":"C:\\secret\\config.yaml","raw_quote":"private"}),
            language: "en-US".to_owned(),
        };
        let prompt = build_prompt(&config, &request).expect("prompt");
        assert!(prompt.contains("<path:"));
        assert!(!prompt.contains("private"));
        assert!(!prompt.contains("C:\\secret"));
    }

    #[test]
    fn relative_targets_are_redacted_and_settings_can_be_saved_twice() {
        let config = AssistantConfig::default();
        let request = AssistantRequest {
            query: "why".to_owned(),
            chain: json!({"target":"config.yaml"}),
            language: "en-US".to_owned(),
        };
        let prompt = build_prompt(&config, &request).expect("prompt");
        assert!(prompt.contains("<path:"));

        let directory =
            std::env::temp_dir().join(format!("causativelens-assist-test-{}", std::process::id()));
        let path = directory.join("settings.json");
        let _ = fs::remove_dir_all(&directory);
        save_config_to(&config, &path).expect("first save");
        save_config_to(&config, &path).expect("second save replaces existing file");
        assert_eq!(
            load_config_from(&path).expect("load").max_tokens,
            config.max_tokens
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn malformed_provider_text_is_kept_as_an_explicit_ungrounded_note() {
        let config = AssistantConfig {
            provider: AssistantProvider::Jev,
            model: "jev-test".to_owned(),
            ..AssistantConfig::default()
        };
        let result = parse_result(&config, "plain assistant text", &json!({})).expect("fallback");
        assert!(!result.grounded);
        assert!(result.summary.contains("plain assistant"));
    }

    #[test]
    fn endpoint_and_header_inputs_are_bounded() {
        let config = AssistantConfig {
            enabled: true,
            endpoint: "file:///private/chain".to_owned(),
            model: "jev-test".to_owned(),
            ..AssistantConfig::default()
        };
        assert!(config.normalized().is_err());
        let config = AssistantConfig {
            enabled: true,
            endpoint: "https://example.invalid/v1".to_owned(),
            model: "jev-test".to_owned(),
            ..AssistantConfig::default()
        };
        let error = request_openai_compatible(&config, Some("bad\nkey"), "prompt")
            .expect_err("line breaks must never reach curl headers");
        assert!(error.to_string().contains("line break"));
    }

    #[test]
    fn citations_must_exist_in_the_submitted_chain() {
        let config = AssistantConfig {
            provider: AssistantProvider::Jev,
            model: "jev-test".to_owned(),
            ..AssistantConfig::default()
        };
        let chain = json!({
            "trigger_assertion": {"id": 7},
            "evidence_events": [{"event_seq": 11}]
        });
        let text = r#"{"summary":"ok","findings":[{"title":"grounded","severity":"info","explanation":"a","assertion_ids":[7],"event_seqs":[11]},{"title":"invented","severity":"high","explanation":"b","assertion_ids":[999],"event_seqs":[]}],"next_steps":[],"caveats":[],"grounded":true}"#;
        let result = parse_result(&config, text, &chain).expect("json result");
        assert!(!result.grounded);
        assert!(result.caveats.iter().any(|item| item.contains("outside")));
    }
}
