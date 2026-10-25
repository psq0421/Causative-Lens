use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const PRODUCT_NAME: &str = "CausativeLens";
pub const PRODUCT_SLUG: &str = "causativelens";
pub const DEV_VERSION: &str = "0.1.2";
pub const RPC_PIPE_NAME: &str = r"\\.\pipe\causativelens-rpc";
pub const ETW_SESSION_PREFIX: &str = "CausativeLens_";
pub const CAPTURE_CONTROL_MAX_BYTES: usize = 4096;
pub const RPC_MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const RPC_METHOD_MAX_BYTES: usize = 128;
pub const CAPTURE_HEARTBEAT_INTERVAL_MS: i64 = 2_000;
pub const CAPTURE_HEARTBEAT_MISSES_BEFORE_FAILURE: u8 = 3;
pub const CAPTURE_FAILURE_CODE: &str = "CL-2001";

/// User-facing language used by the CLI, GUI and installer documentation.
///
/// Protocol and JSON field names remain English and stable; this value only
/// selects human-readable copy. Keeping the type in `clens-types` lets both
/// frontends agree on the same three supported locales without sharing UI
/// implementation details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiLanguage {
    #[serde(rename = "en-US")]
    EnglishUs,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
    #[serde(rename = "zh-TW")]
    TraditionalChinese,
}

impl UiLanguage {
    pub const ALL: [Self; 3] = [
        Self::SimplifiedChinese,
        Self::TraditionalChinese,
        Self::EnglishUs,
    ];

    pub const fn tag(self) -> &'static str {
        match self {
            Self::EnglishUs => "en-US",
            Self::SimplifiedChinese => "zh-CN",
            Self::TraditionalChinese => "zh-TW",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::EnglishUs => "English (US)",
            Self::SimplifiedChinese => "简体中文",
            Self::TraditionalChinese => "繁體中文",
        }
    }

    /// Parse BCP-47-ish tags and the short aliases users commonly type.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "en-us" | "english" | "english-us" => Some(Self::EnglishUs),
            "zh" | "zh-cn" | "zh-hans" | "sc" | "simplified" | "简体中文" => {
                Some(Self::SimplifiedChinese)
            }
            "zh-tw" | "zh-hk" | "zh-hant" | "tc" | "traditional" | "繁體中文" => {
                Some(Self::TraditionalChinese)
            }
            _ => None,
        }
    }

    pub fn from_environment() -> Self {
        ["CAUSATIVELENS_LANG", "LC_ALL", "LANG"]
            .into_iter()
            .find_map(|name| {
                std::env::var(name)
                    .ok()
                    .and_then(|value| Self::parse(&value))
            })
            .unwrap_or(Self::SimplifiedChinese)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaptureControlMessage {
    Start { session_id: String },
    Stop { session_id: String },
    Heartbeat { nonce: u64 },
    Metrics,
    Error { code: String, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureControlDecodeError {
    TooLarge,
    InvalidJson(String),
}

/// Decode only the small, enumerated capture control protocol. Unknown tags,
/// extra fields, and oversized input are rejected before any side effect.
pub fn decode_capture_control(
    bytes: &[u8],
) -> Result<CaptureControlMessage, CaptureControlDecodeError> {
    if bytes.len() > CAPTURE_CONTROL_MAX_BYTES {
        return Err(CaptureControlDecodeError::TooLarge);
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| CaptureControlDecodeError::InvalidJson(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        CaptureControlDecodeError::InvalidJson("control message must be a JSON object".to_owned())
    })?;
    let message_type = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            CaptureControlDecodeError::InvalidJson("control message type is required".to_owned())
        })?;
    let allowed = match message_type {
        "start" | "stop" => ["type", "session_id"].as_slice(),
        "heartbeat" => ["type", "nonce"].as_slice(),
        "metrics" => ["type"].as_slice(),
        "error" => ["type", "code", "message"].as_slice(),
        _ => {
            return Err(CaptureControlDecodeError::InvalidJson(
                "unknown capture control type".to_owned(),
            ))
        }
    };
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(CaptureControlDecodeError::InvalidJson(
            "unknown capture control field".to_owned(),
        ));
    }
    serde_json::from_value(value)
        .map_err(|error| CaptureControlDecodeError::InvalidJson(error.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureHeartbeat {
    pub last_seen_ms: i64,
    pub missed: u8,
}

impl CaptureHeartbeat {
    pub fn new(now_ms: i64) -> Self {
        Self {
            last_seen_ms: now_ms,
            missed: 0,
        }
    }

    pub fn observe(&mut self, now_ms: i64) {
        self.last_seen_ms = now_ms;
        self.missed = 0;
    }

    pub fn check(&mut self, now_ms: i64) -> Option<&'static str> {
        let elapsed = now_ms.saturating_sub(self.last_seen_ms);
        if elapsed < CAPTURE_HEARTBEAT_INTERVAL_MS {
            return None;
        }
        let intervals = elapsed / CAPTURE_HEARTBEAT_INTERVAL_MS;
        self.missed = intervals
            .min(i64::from(u8::MAX))
            .try_into()
            .unwrap_or(u8::MAX);
        if self.missed >= CAPTURE_HEARTBEAT_MISSES_BEFORE_FAILURE {
            Some(CAPTURE_FAILURE_CODE)
        } else {
            None
        }
    }
}

pub type PathIdx = u32;

/// Host patterns are kept as normalized plain text in the cross-module IR.
/// The matcher is compiled by `clens-core`; storing a matcher object here
/// would make the model platform-dependent and non-serializable.
pub type HostPat = Box<str>;

/// Canonical textual form used by the serializable target IR. Matching
/// engines may apply case/volume policy later, but the wire value is stable
/// across Windows path separator spellings.
pub fn normalize_path_pattern(value: &str) -> Box<str> {
    value.trim().replace('\\', "/").into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SysKind {
    ProcStart,
    ProcStop,
    ThreadStart,
    ThreadStop,
    ImageLoad,
    FileCreate,
    FileRead,
    FileWrite,
    FileClose,
    FileRename,
    FileSetInfo,
    FileDelete,
    RegOpen,
    RegQuery,
    RegSet,
    RegDelete,
    NetConnect,
    NetAccept,
    NetDisconnect,
    NetSend,
    NetRecv,
    NetListen,
    NetBind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetProto {
    Tcp,
    Udp,
    Other(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    pub host: Box<str>,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventPayload {
    Proc {
        ppid: u32,
        image: Box<str>,
        cmd: Box<str>,
        exit_code: Option<u32>,
    },
    File {
        path: PathIdx,
        desired_access: u32,
        io_len: Option<u32>,
        write_attrs: bool,
    },
    Net {
        proto: NetProto,
        local: Endpoint,
        remote: Endpoint,
    },
    Reg {
        key: Box<str>,
        value: Option<Box<str>>,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysEvent {
    pub schema_v: u16,
    pub seq: u64,
    pub ts_qpc: u64,
    pub pid: u32,
    pub tid: u32,
    pub kind: SysKind,
    #[serde(default)]
    pub is_rundown: bool,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Reasoning,
    ToolCall,
    ToolResult,
    System,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: Box<str>,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessEvent {
    pub schema_v: u16,
    pub ts_utc_ms: i64,
    pub step: u64,
    pub phase: Phase,
    pub tool: Option<ToolCall>,
    pub text: Box<str>,
    /// Stable origin tuple used to derive assertion IDs. Older JSONL
    /// producers may omit these fields; the parser then falls back to step
    /// and line-local offsets.
    #[serde(default)]
    pub harness_log_id: u64,
    #[serde(default)]
    pub line_no: u64,
    #[serde(default)]
    pub byte_start: u64,
    #[serde(default)]
    pub byte_end: u64,
    /// Original JSONL record retained for forward-compatible parsing. This is
    /// especially important when a producer adds a phase or tool shape that
    /// this version does not understand yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_json: Option<Box<str>>,
}

/// The byte span of the source JSONL record that produced an assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSpan {
    pub step: u64,
    pub byte_start: u64,
    pub byte_end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionOp {
    Read,
    Write,
    Create,
    Delete,
    Rename,
    Exec,
    Connect,
    Bind,
    Listen,
    RegRead,
    RegWrite,
    AssertPrecondition,
    AssertPostcondition,
    RollbackPlan,
    Constraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPattern {
    Path(Box<str>),
    RegistryKey(Box<str>),
    Endpoint {
        host: HostPat,
        port: Option<u16>,
        proto: NetProto,
    },
    FreeText(Box<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoMode {
    ReadOnly,
    ReadWrite,
    CreateNew,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expectation {
    MustHappen,
    MustNotHappen,
    HappenAfter(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    pub schema_v: u16,
    pub id: u64,
    pub span: HarnessSpan,
    pub op: AssertionOp,
    pub target: TargetPattern,
    pub io_mode: IoMode,
    pub expect: Expectation,
    pub order_key: Option<(u64, u64)>,
    pub confidence: f32,
    pub raw_quote: Box<str>,
}

/// Deterministic FNV-1a based ID for `(harness_log_id, line_no, slot_no)`.
/// It deliberately has no random or process-local component so replaying the
/// same JSONL produces the same assertion graph.
pub fn stable_assertion_id(harness_log_id: u64, line_no: u64, slot_no: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for value in [harness_log_id, line_no, slot_no] {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

/// Public fulfilment state stored in the session graph. Candidate matching
/// may use an additional internal `Unrelated` state, but it is never emitted
/// as a verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fulfilment {
    Fulfilled,
    Deviated,
    Unobserved,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreComponents {
    pub process: f32,
    pub time: f32,
    pub semantic: f32,
    pub path: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Watermark {
    pub system_wall_ms: i64,
    pub now_ms: i64,
    pub disorder_ms: i64,
    pub assertion_lag_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deviation {
    pub reason: Box<str>,
    pub observed_event_seq: Option<u64>,
    pub expected: Option<Box<str>>,
    pub observed: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceEdge {
    pub event_seq: u64,
    pub score: f32,
    pub components: ScoreComponents,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub schema_v: u16,
    pub assertion_id: u64,
    pub state: Fulfilment,
    pub evidence: Vec<EvidenceEdge>,
    pub deviation: Option<Deviation>,
    pub decided_at: Watermark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Observe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloRequest {
    pub protocol_version: u16,
    pub mode: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloResponse {
    pub protocol_version: u16,
    pub product: String,
    pub version: String,
    pub mode: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTicket {
    pub session_id: Uuid,
    pub mode: Mode,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcError {
    #[error("unsupported protocol version: {0}")]
    UnsupportedProtocol(u16),
    #[error("intervention and decision commands are unavailable in observe mode")]
    InterventionUnavailable,
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("invalid method parameters: {0}")]
    InvalidParams(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_identity_uses_causativelens_names() {
        assert_eq!(PRODUCT_NAME, "CausativeLens");
        assert_eq!(PRODUCT_SLUG, "causativelens");
        assert!(RPC_PIPE_NAME.contains("causativelens-rpc"));
        assert!(ETW_SESSION_PREFIX.starts_with("CausativeLens_"));
    }

    #[test]
    fn ui_language_accepts_supported_aliases_and_keeps_tags_stable() {
        assert_eq!(UiLanguage::parse("en"), Some(UiLanguage::EnglishUs));
        assert_eq!(
            UiLanguage::parse("zh-Hant"),
            Some(UiLanguage::TraditionalChinese)
        );
        assert_eq!(
            UiLanguage::parse("zh-CN").map(UiLanguage::tag),
            Some("zh-CN")
        );
        assert!(UiLanguage::parse("fr-FR").is_none());
    }

    #[test]
    fn capture_control_protocol_rejects_unknown_and_oversized_messages() {
        let heartbeat = serde_json::to_vec(&CaptureControlMessage::Heartbeat { nonce: 7 })
            .expect("test serialization");
        assert_eq!(
            decode_capture_control(&heartbeat),
            Ok(CaptureControlMessage::Heartbeat { nonce: 7 })
        );
        let unknown = decode_capture_control(br#"{"type":"inject_policy"}"#);
        assert!(matches!(
            unknown,
            Err(CaptureControlDecodeError::InvalidJson(_))
        ));
        let extra = decode_capture_control(br#"{"type":"metrics","policy":"enforce"}"#);
        assert!(matches!(
            extra,
            Err(CaptureControlDecodeError::InvalidJson(_))
        ));
        let oversized = vec![b'x'; CAPTURE_CONTROL_MAX_BYTES + 1];
        assert_eq!(
            decode_capture_control(&oversized),
            Err(CaptureControlDecodeError::TooLarge)
        );
    }

    #[test]
    fn capture_heartbeat_reports_three_two_second_misses_as_cl_2001() {
        let mut heartbeat = CaptureHeartbeat::new(0);
        assert_eq!(heartbeat.check(3_999), None);
        assert_eq!(heartbeat.check(6_000), Some(CAPTURE_FAILURE_CODE));
        heartbeat.observe(6_000);
        assert_eq!(heartbeat.missed, 0);
    }

    #[test]
    fn hello_contract_is_json_stable() {
        let request = HelloRequest {
            protocol_version: 1,
            mode: Mode::Observe,
        };
        let json = serde_json::to_string(&request).expect("test serialization");
        assert_eq!(json, r#"{"protocol_version":1,"mode":"observe"}"#);
    }

    #[test]
    fn assertion_id_is_stable_and_slot_sensitive() {
        let first = stable_assertion_id(42, 7, 0);
        assert_eq!(first, stable_assertion_id(42, 7, 0));
        assert_ne!(first, stable_assertion_id(42, 7, 1));
        assert_ne!(first, 0);
    }

    #[test]
    fn path_pattern_normalization_is_serializable_and_platform_neutral() {
        assert_eq!(
            &*normalize_path_pattern(r"  C:\work\config.yaml  "),
            "C:/work/config.yaml"
        );
    }

    #[test]
    fn core_model_round_trip_keeps_source_span_and_verdict_edges() {
        let assertion = Assertion {
            schema_v: 1,
            id: stable_assertion_id(1, 2, 0),
            span: HarnessSpan {
                step: 2,
                byte_start: 10,
                byte_end: 40,
            },
            op: AssertionOp::Read,
            target: TargetPattern::Path("C:/work/*.toml".into()),
            io_mode: IoMode::ReadOnly,
            expect: Expectation::MustHappen,
            order_key: Some((8, 9)),
            confidence: 0.9,
            raw_quote: "read config".into(),
        };
        let verdict = Verdict {
            schema_v: 1,
            assertion_id: assertion.id,
            state: Fulfilment::Fulfilled,
            evidence: vec![EvidenceEdge {
                event_seq: 11,
                score: 0.95,
                components: ScoreComponents {
                    process: 1.0,
                    time: 0.9,
                    semantic: 1.0,
                    path: 0.9,
                    total: 0.95,
                },
            }],
            deviation: None,
            decided_at: Watermark {
                system_wall_ms: 100,
                now_ms: 100,
                disorder_ms: 10,
                assertion_lag_ms: 5,
            },
        };
        let assertion_json = serde_json::to_string(&assertion).expect("assertion serialization");
        let verdict_json = serde_json::to_string(&verdict).expect("verdict serialization");
        assert_eq!(
            serde_json::from_str::<Assertion>(&assertion_json).expect("assertion round trip"),
            assertion
        );
        assert_eq!(
            serde_json::from_str::<Verdict>(&verdict_json).expect("verdict round trip"),
            verdict
        );
    }
}
