use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueryIntent {
    Why,
    When,
    WhatIf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct QueryEntity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuredQuery {
    pub intent: QueryIntent,
    pub entity: QueryEntity,
    pub source: String,
    pub mode: &'static str,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TriggerAssertion {
    pub id: u64,
    pub step: u64,
    pub op: String,
    pub target: String,
    pub raw_quote: String,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceEvent {
    pub seq: u64,
    pub ts_utc_ms: i64,
    pub pid: u32,
    pub operation: String,
    pub target: String,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerdictView {
    pub state: String,
    pub evidence_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deviation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReverseAttribution {
    pub event_seq: u64,
    pub assertion_ids: Vec<u64>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureHealthNote {
    pub events_seen: u64,
    pub spool_gaps: u64,
    pub harness_quarantine: u64,
    pub confidence: f32,
    pub note: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct CausalRecord {
    pub trigger_assertion: Option<TriggerAssertion>,
    pub evidence_events: Vec<EvidenceEvent>,
    pub verdict: Option<VerdictView>,
    pub reverse_attribution: Vec<ReverseAttribution>,
    pub health: Option<CaptureHealthNote>,
}

/// Stable five-part response shape shared by CLI, GUI and RPC query consumers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CausalChain {
    pub trigger_assertion: Option<TriggerAssertion>,
    pub evidence_events: Vec<EvidenceEvent>,
    pub verdict: Option<VerdictView>,
    pub reverse_attribution: Vec<ReverseAttribution>,
    pub confidence_health: CaptureHealthNote,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QueryResponse {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<QueryIntent>,
    pub entity: QueryEntity,
    pub mode: &'static str,
    pub chain: CausalChain,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedQuery {
    pub query: StructuredQuery,
    pub response: QueryResponse,
}

/// Deterministic v0 router. It intentionally does not invoke an LLM.
pub fn parse_restricted_query(input: &str) -> ParsedQuery {
    let source = input.trim().to_owned();
    let lower = source.to_ascii_lowercase();
    let intent = if lower.contains("what-if")
        || lower.contains("what if")
        || source.contains("\u{5982}\u{679c}")
        || source.contains("\u{5047}\u{5982}")
        || source.contains("\u{5982}\u{4f55}")
    {
        Some(QueryIntent::WhatIf)
    } else if lower.contains("when")
        || source.contains("\u{4ec0}\u{4e48}\u{65f6}\u{5019}")
        || source.contains("\u{4f55}\u{65f6}")
        || source.contains("\u{4ec0}\u{9ebc}\u{6642}\u{5019}")
        || source.contains("\u{4f55}\u{6642}")
    {
        Some(QueryIntent::When)
    } else if lower.contains("why")
        || source.contains("\u{4e3a}\u{4ec0}\u{4e48}")
        || source.contains("\u{4e3a}\u{4f55}")
        || source.contains("\u{600e}\u{4e48}\u{4f1a}")
        || source.contains("\u{70ba}\u{4ec0}\u{9ebc}")
        || source.contains("\u{70ba}\u{4f55}")
        || source.contains("\u{600e}\u{9ebc}\u{6703}")
    {
        Some(QueryIntent::Why)
    } else {
        None
    };
    let entity = extract_entities(&source);
    let matched = intent.is_some()
        && (entity.path.is_some() || entity.endpoint.is_some() || entity.step.is_some());
    let effective_intent = intent.unwrap_or(QueryIntent::Why);
    let query = StructuredQuery {
        intent: effective_intent,
        entity: entity.clone(),
        source,
        mode: "observe",
    };
    let response = if matched {
        QueryResponse {
            query: intent_label(effective_intent).to_owned(),
            intent,
            entity,
            mode: "observe",
            chain: empty_chain(),
            hint: None,
        }
    } else {
        QueryResponse {
            query: "unsupported".to_owned(),
            intent: None,
            entity,
            mode: "observe",
            chain: empty_chain(),
            hint: Some("use clens why --path <p>".to_owned()),
        }
    };
    ParsedQuery { query, response }
}

fn empty_chain() -> CausalChain {
    CausalChain {
        trigger_assertion: None,
        evidence_events: Vec::new(),
        verdict: None,
        reverse_attribution: Vec::new(),
        confidence_health: CaptureHealthNote {
            events_seen: 0,
            spool_gaps: 0,
            harness_quarantine: 0,
            confidence: 0.0,
            note: "no matching session records".to_owned(),
        },
    }
}
fn intent_label(intent: QueryIntent) -> &'static str {
    match intent {
        QueryIntent::Why => "why",
        QueryIntent::When => "when",
        QueryIntent::WhatIf => "what-if",
    }
}

fn extract_entities(source: &str) -> QueryEntity {
    let tokens: Vec<&str> = source.split_whitespace().collect();
    let mut entity = QueryEntity::default();
    for (index, raw) in tokens.iter().enumerate() {
        let token = trim_punctuation(raw);
        let lower = token.to_ascii_lowercase();
        if *raw == "--path"
            || lower == "path"
            || lower == "file"
            || *raw == "\u{8def}\u{5f91}"
            || *raw == "\u{8def}\u{5f84}"
        {
            if let Some(next) = tokens.get(index + 1) {
                if *raw == "--path" {
                    let joined = tokens[index + 1..].join(" ");
                    entity.path = Some(trim_punctuation(&joined).to_owned());
                } else {
                    entity.path = Some(trim_punctuation(next).to_owned());
                }
            }
        }
        if entity.path.is_none() && looks_like_path(token) {
            entity.path = Some(token.to_owned());
        }
        if entity.endpoint.is_none() && looks_like_endpoint(token) {
            entity.endpoint = Some(token.to_owned());
        }
        if entity.step.is_none()
            && (lower == "step" || *raw == "\u{6b65}\u{9aa4}" || *raw == "\u{6b65}\u{9a5f}")
        {
            if let Some(next) = tokens.get(index + 1) {
                entity.step = parse_number(next);
            }
        }
    }
    if entity.step.is_none() {
        if let Some(start) = source.find("\u{7b2c}") {
            let tail = &source[start + "\u{7b2c}".len()..];
            let digits: String = tail.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            if !digits.is_empty() {
                entity.step = digits.parse().ok();
            }
        }
    }
    entity
}
fn trim_punctuation(value: &str) -> &str {
    value.trim_matches(|ch: char| {
        matches!(
            ch,
            '`' | '"'
                | '\''
                | ','
                | '.'
                | '?'
                | '!'
                | ':'
                | '('
                | ')'
                | '\u{ff0c}'
                | '\u{ff01}'
                | '\u{ff1f}'
                | '\u{3002}'
                | '\u{ff1a}'
                | '\u{ff08}'
                | '\u{ff09}'
        )
    })
}
fn parse_number(value: &str) -> Option<u64> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u64>().ok()
    }
}
fn looks_like_path(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.contains('/')
        || value.contains('\\')
        || [".yaml", ".yml", ".json", ".toml", ".ini", ".rs"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
}
fn looks_like_endpoint(value: &str) -> bool {
    let Some((host, port)) = value.rsplit_once(':') else {
        return false;
    };
    !host.is_empty() && port.parse::<u16>().is_ok() && !host.contains('/')
}

pub fn render_causal_chain(_query: &StructuredQuery, records: &[CausalRecord]) -> CausalChain {
    let mut chain = empty_chain();
    if let Some(record) = records.first() {
        chain.trigger_assertion = record.trigger_assertion.clone();
        chain.evidence_events = record.evidence_events.clone();
        chain.verdict = record.verdict.clone();
        chain.reverse_attribution = record.reverse_attribution.clone();
        if let Some(health) = &record.health {
            chain.confidence_health = health.clone();
        }
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chinese_why_path_routes_without_llm() {
        let parsed = parse_restricted_query("\u{4e3a}\u{4ec0}\u{4e48} config.yaml\u{ff1f}");
        assert_eq!(parsed.query.intent, QueryIntent::Why);
        assert_eq!(parsed.query.entity.path.as_deref(), Some("config.yaml"));
        assert!(parsed.response.hint.is_none());
    }
    #[test]
    fn english_when_endpoint_and_step_are_extracted() {
        let parsed = parse_restricted_query("when did step 12 connect api.example.com:443");
        assert_eq!(parsed.query.intent, QueryIntent::When);
        assert_eq!(parsed.query.entity.step, Some(12));
        assert_eq!(
            parsed.query.entity.endpoint.as_deref(),
            Some("api.example.com:443")
        );
    }
    #[test]
    fn chinese_step_entity_is_extracted() {
        let parsed = parse_restricted_query(
            "\u{4ec0}\u{4e48}\u{65f6}\u{5019}\u{7b2c}12\u{6b65} connect api.example.com:443",
        );
        assert_eq!(parsed.query.entity.step, Some(12));
    }
    #[test]
    fn traditional_chinese_routes_and_extracts_path() {
        let parsed = parse_restricted_query("為什麼它修改了 config.yaml？");
        assert_eq!(parsed.query.intent, QueryIntent::Why);
        assert_eq!(parsed.query.entity.path.as_deref(), Some("config.yaml"));
    }
    #[test]
    fn unknown_text_gets_safe_path_hint() {
        let parsed = parse_restricted_query("tell me everything");
        assert_eq!(parsed.response.query, "unsupported");
        assert_eq!(
            parsed.response.hint.as_deref(),
            Some("use clens why --path <p>")
        );
    }
    #[test]
    fn response_has_all_five_chain_sections() {
        let value = serde_json::to_value(parse_restricted_query("why --path config.yaml").response)
            .expect("json");
        for key in [
            "trigger_assertion",
            "evidence_events",
            "verdict",
            "reverse_attribution",
            "confidence_health",
        ] {
            assert!(value
                .get("chain")
                .and_then(|chain| chain.get(key))
                .is_some());
        }
    }
}
