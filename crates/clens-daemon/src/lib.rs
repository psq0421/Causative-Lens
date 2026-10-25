mod monitoring;
use clens_assist::{analyze as analyze_with_assistant, AssistantConfig, AssistantRequest};
use clens_capture::{CaptureHealth, ClockAnchor};
use clens_core::{
    align_candidate_at, event_path_score, is_constraint_conflict, semantic_score,
    watermark_value_ms, AlignmentMatch, AlignmentProfile, ProcessTree, TimedAssertion, Watermark,
};
use clens_harnessd::monitor::LiveMonitor;
use clens_harnessd::{extract_rule_assertions, AssertionPool, TailHealth};
use clens_storage::SpoolFrame;
use clens_types::{
    HarnessEvent, HelloRequest, HelloResponse, Mode, RpcError, SessionTicket, SysEvent,
    DEV_VERSION, PRODUCT_NAME, RPC_MAX_MESSAGE_BYTES, RPC_METHOD_MAX_BYTES,
};
use monitoring::{category_of, target_of, TimelineBuffer, TimelineRow};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const RPC_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaemonThreadPlan {
    pub spool_reader: usize,
    pub harness_tail: usize,
    pub assertion_workers: usize,
    pub alignment: usize,
    pub sqlite_writer: usize,
}

pub fn daemon_thread_plan(physical_cpus: usize) -> DaemonThreadPlan {
    DaemonThreadPlan {
        spool_reader: 1,
        harness_tail: 1,
        assertion_workers: (physical_cpus / 2).clamp(1, 4),
        alignment: 1,
        sqlite_writer: 1,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    #[error("session serialization failed: {0}")]
    Serialization(String),
}

#[derive(Debug)]
pub struct ObservationSession {
    monitor: Option<LiveMonitor>,
    timeline: TimelineBuffer,
    seen_harness: HashSet<(u64, u64)>,
    pub capture_health: CaptureHealth,
    pub harness_health: TailHealth,
    pub process_tree: ProcessTree,
    pub path_table: HashMap<u32, String>,
    path_index: HashMap<String, Vec<u64>>,
    pub assertion_pool: AssertionPool,
    pub matches: Vec<AlignmentMatch>,
    pub events: HashMap<u64, SysEvent>,
    pub spool_frames: Vec<SpoolFrame>,
    pub watermark_ms: i64,
    pub duplicate_events: u64,
    assertion_centers_ms: HashMap<u64, i64>,
    assertion_subject_pids: HashMap<u64, u32>,
    clock_anchor: Option<(ClockAnchor, u64)>,
    seen_event_sequences: HashSet<u64>,
    profile: AlignmentProfile,
}

impl Default for ObservationSession {
    fn default() -> Self {
        Self {
            monitor: None,
            timeline: TimelineBuffer::default(),
            seen_harness: HashSet::new(),
            capture_health: CaptureHealth::default(),
            harness_health: TailHealth::default(),
            process_tree: ProcessTree::default(),
            path_table: HashMap::new(),
            path_index: HashMap::new(),
            assertion_pool: AssertionPool::default(),
            matches: Vec::new(),
            events: HashMap::new(),
            spool_frames: Vec::new(),
            watermark_ms: i64::MIN,
            duplicate_events: 0,
            assertion_centers_ms: HashMap::new(),
            assertion_subject_pids: HashMap::new(),
            clock_anchor: None,
            seen_event_sequences: HashSet::new(),
            profile: AlignmentProfile::default(),
        }
    }
}

impl ObservationSession {
    pub fn ingest_harness_event(&mut self, event: &HarnessEvent) -> Option<u64> {
        if !self
            .seen_harness
            .insert((event.harness_log_id, event.line_no.max(event.step)))
        {
            return None;
        }
        if self.seen_harness.len() > 100_000 {
            self.seen_harness.clear();
        }
        let kind = event
            .tool
            .as_ref()
            .map_or_else(|| format!("{:?}", event.phase), |t| t.name.to_string());
        self.timeline.push(TimelineRow {
            seq: 0,
            ts_utc_ms: event.ts_utc_ms,
            category: category_of(event).into(),
            source: "agent".into(),
            kind,
            target: target_of(event),
            text: event.text.to_string(),
            state: if event.phase == clens_types::Phase::ToolCall {
                "reported"
            } else {
                "recorded"
            }
            .into(),
        });
        let assertions = extract_rule_assertions(event, 0);
        if assertions.is_empty() {
            return None;
        }
        let mut first_id = None;
        for assertion in assertions {
            let target = match &assertion.target {
                clens_types::TargetPattern::Path(p) => p.to_string(),
                other => format!("{other:?}"),
            };
            self.timeline.push(TimelineRow {
                seq: 0,
                ts_utc_ms: event.ts_utc_ms,
                category: "assertions".into(),
                source: "assertion".into(),
                kind: format!("{:?}", assertion.op),
                target,
                text: assertion.raw_quote.to_string(),
                state: "unobserved".into(),
            });
            let id = self.assertion_pool.insert(assertion);
            if let Some(candidate) = self.assertion_pool.values().find(|value| value.id == id) {
                if let clens_types::TargetPattern::Path(path) = &candidate.target {
                    self.path_index
                        .entry(path.to_ascii_lowercase())
                        .or_default()
                        .push(id);
                }
            }
            self.assertion_centers_ms.insert(id, event.ts_utc_ms);
            if first_id.is_none() {
                first_id = Some(id);
            }
        }
        first_id
    }

    pub fn record_harness_health(&mut self, health: TailHealth) {
        self.harness_health = health;
    }

    pub fn record_path(&mut self, path_id: u32, path: impl Into<String>) {
        self.path_table.insert(path_id, path.into());
    }

    pub fn record_assertion_subject_pid(&mut self, assertion_id: u64, pid: u32) {
        self.assertion_subject_pids.insert(assertion_id, pid);
    }

    pub fn set_clock_anchor(&mut self, anchor: ClockAnchor, qpc_per_second: u64) {
        self.clock_anchor = Some((anchor, qpc_per_second.max(1)));
    }

    pub fn ingest_system_event(&mut self, event: &SysEvent) -> Result<(), ObservationError> {
        if !self.seen_event_sequences.insert(event.seq) {
            self.duplicate_events = self.duplicate_events.saturating_add(1);
            self.capture_health.events_deduplicated =
                self.capture_health.events_deduplicated.saturating_add(1);
            return Ok(());
        }
        self.capture_health.events_seen = self.capture_health.events_seen.saturating_add(1);
        let kind = format!("{:?}", event.kind);
        let timestamp = self
            .clock_anchor
            .map(|(anchor, frequency)| anchor.qpc_to_utc_ms(event.ts_qpc, frequency));
        self.timeline.push(TimelineRow {
            seq: 0,
            ts_utc_ms: timestamp.unwrap_or_else(clens_harnessd::monitor::now_ms),
            category: if kind.starts_with("File") {
                "files"
            } else if kind.starts_with("Net") {
                "network"
            } else {
                "other"
            }
            .into(),
            source: "system".into(),
            kind,
            target: self.event_target(event),
            text: format!(
                "pid={} seq={} time_basis={}",
                event.pid,
                event.seq,
                if timestamp.is_some() {
                    "anchored_qpc"
                } else {
                    "received"
                }
            ),
            state: "observed".into(),
        });
        self.events.insert(event.seq, event.clone());
        match (&event.kind, &event.payload) {
            (clens_types::SysKind::ProcStart, clens_types::EventPayload::Proc { ppid, .. }) => {
                self.process_tree.observe_start(event.pid, *ppid);
            }
            (clens_types::SysKind::ProcStop, _) => self.process_tree.observe_stop(event.pid),
            _ => {}
        }
        let payload = serde_json::to_vec(event)
            .map_err(|error| ObservationError::Serialization(error.to_string()))?;
        self.spool_frames.push(SpoolFrame {
            schema_v: 1,
            flags: 0,
            raw_len: payload.len() as u32,
            payload,
        });
        let event_ts_ms = self
            .clock_anchor
            .map_or(event.ts_qpc as i64, |(anchor, qpc_per_second)| {
                anchor.qpc_to_utc_ms(event.ts_qpc, qpc_per_second)
            });
        for assertion in self.assertion_pool.values() {
            let timed = TimedAssertion {
                assertion,
                center_ts_ms: self
                    .assertion_centers_ms
                    .get(&assertion.id)
                    .copied()
                    .unwrap_or(event_ts_ms),
                window_end_ms: self
                    .assertion_centers_ms
                    .get(&assertion.id)
                    .copied()
                    .unwrap_or(event_ts_ms)
                    + self.profile.time_center_window_ms,
            };
            let conflict = is_constraint_conflict(assertion, event);
            let semantic = if conflict {
                1.0
            } else {
                semantic_score(assertion, event)
            };
            if semantic == 0.0 {
                continue;
            }
            let process_distance = self
                .assertion_subject_pids
                .get(&assertion.id)
                .and_then(|subject_pid| self.process_tree.distance(*subject_pid, event.pid))
                .unwrap_or(0);
            let path = event_path_score(assertion, event, &self.path_table);
            let path = if path == 0.0
                && !matches!(assertion.target, clens_types::TargetPattern::Path(_))
            {
                1.0
            } else {
                path
            };
            let mut matched = align_candidate_at(
                timed,
                event,
                event_ts_ms,
                process_distance,
                semantic,
                path,
                self.profile,
            );
            if assertion.expect == clens_types::Expectation::MustNotHappen
                && !matches!(matched.state, clens_core::VerdictState::Unrelated)
            {
                matched.state = clens_core::VerdictState::Deviated;
            }
            if !matches!(matched.state, clens_core::VerdictState::Unrelated) {
                self.matches.push(matched);
            }
        }
        Ok(())
    }

    pub fn advance_watermark(&mut self, watermark: Watermark) -> i64 {
        self.watermark_ms = self.watermark_ms.max(watermark_value_ms(watermark));
        self.watermark_ms
    }

    /// Execute the deterministic observe-only `why --path` query against the
    /// in-memory session index. The SQLite writer persists the same five
    /// sections; this method keeps the RPC contract usable before a session is
    /// flushed to disk.
    pub fn query_why(&self, path: &str) -> Value {
        let needle = path.replace('\\', "/").to_ascii_lowercase();
        let indexed_ids = self.path_index.get(&needle).cloned().unwrap_or_default();
        let assertion = if indexed_ids.is_empty() {
            self.assertion_pool.values().find(|candidate| {
                matches!(&candidate.target, clens_types::TargetPattern::Path(value) if value.to_ascii_lowercase().contains(&needle))
            })
        } else {
            self.assertion_pool
                .values()
                .find(|candidate| indexed_ids.contains(&candidate.id))
        };
        let Some(assertion) = assertion else {
            return json!({
                "query": "why",
                "intent": "why",
                "entity": {"path": path},
                "mode": "observe",
                "chain": {
                    "trigger_assertion": null,
                    "evidence_events": [],
                    "verdict": null,
                    "reverse_attribution": [],
                    "confidence_health": self.query_health("no matching assertion")
                }
            });
        };
        let evidence: Vec<Value> = self
            .matches
            .iter()
            .filter(|candidate| candidate.assertion_id == assertion.id)
            .map(|candidate| {
                let event = self.events.get(&candidate.event_seq);
                json!({
                    "event_seq": candidate.event_seq,
                    "score": candidate.score.total,
                    "state": format!("{:?}", candidate.state).to_ascii_lowercase(),
                    "ts_qpc": event.map(|value| value.ts_qpc),
                    "pid": event.map(|value| value.pid),
                    "tid": event.map(|value| value.tid),
                    "operation": event.map(|value| format!("{:?}", value.kind)),
                    "target": event.map(|value| self.event_target(value))
                })
            })
            .collect();
        let state = self
            .matches
            .iter()
            .filter(|candidate| candidate.assertion_id == assertion.id)
            .max_by(|left, right| left.score.total.total_cmp(&right.score.total))
            .map(|candidate| format!("{:?}", candidate.state).to_ascii_lowercase())
            .unwrap_or_else(|| "unobserved".to_owned());
        let deviation = if state == "deviated" {
            Some(json!({"reason": "observed event conflicts with the assertion"}))
        } else {
            None
        };
        let reverse_attribution: Vec<Value> = evidence
            .iter()
            .filter_map(|item| item.get("event_seq").and_then(Value::as_u64))
            .map(|event_seq| {
                let assertion_ids: Vec<u64> = self
                    .matches
                    .iter()
                    .filter(|candidate| {
                        candidate.event_seq == event_seq && candidate.assertion_id != assertion.id
                    })
                    .map(|candidate| candidate.assertion_id)
                    .collect();
                json!({"event_seq": event_seq, "assertion_ids": assertion_ids})
            })
            .collect();
        json!({
            "query": "why",
            "intent": "why",
            "entity": {"path": path},
            "mode": "observe",
            "chain": {
                "trigger_assertion": {
                    "id": assertion.id,
                    "step": assertion.span.step,
                    "op": format!("{:?}", assertion.op),
                    "target": format!("{:?}", assertion.target),
                    "raw_quote": assertion.raw_quote.as_ref()
                },
                "evidence_events": evidence,
                "verdict": {"state": state, "deviation": deviation},
                "reverse_attribution": reverse_attribution,
                "confidence_health": self.query_health("observe-only evidence")
            }
        })
    }

    fn event_target(&self, event: &SysEvent) -> String {
        match &event.payload {
            clens_types::EventPayload::File { path, .. } => self
                .path_table
                .get(path)
                .cloned()
                .unwrap_or_else(|| format!("path#{path}")),
            clens_types::EventPayload::Proc { image, .. } => image.to_string(),
            clens_types::EventPayload::Net { remote, .. } => {
                format!("{}:{}", remote.host, remote.port)
            }
            clens_types::EventPayload::Reg { key, .. } => key.to_string(),
            clens_types::EventPayload::None => String::new(),
        }
    }

    fn query_health(&self, note: &str) -> Value {
        json!({
            "events_seen": self.capture_health.events_seen,
            "spool_gaps": self.capture_health.spool_gaps,
            "harness_quarantine": self.harness_health.quarantine_count,
            "confidence": if self.capture_health.events_seen == 0 { 0.0 } else if self.capture_health.events_lost == 0 { 1.0 } else { 0.5 },
            "note": note
        })
    }
}

#[derive(Debug, Default)]
pub struct DaemonState {
    sessions: HashMap<Uuid, ObservationSession>,
    rejected_controls: u64,
}

impl DaemonState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
    pub fn rejected_controls(&self) -> u64 {
        self.rejected_controls
    }
    pub fn handle(&mut self, request: RpcRequest) -> RpcResponse {
        let params_size = serde_json::to_vec(&request.params)
            .map(|bytes| bytes.len())
            .unwrap_or(RPC_MAX_MESSAGE_BYTES.saturating_add(1));
        if request.method.len() > RPC_METHOD_MAX_BYTES || params_size > RPC_MAX_MESSAGE_BYTES {
            self.rejected_controls = self.rejected_controls.saturating_add(1);
            return RpcResponse::error(
                request.id,
                RpcError::InvalidParams("control message exceeds protocol limits".to_owned()),
            );
        }
        for session in self.sessions.values_mut() {
            session.pump_monitor();
        }
        if request.method.starts_with("monitor.") {
            return match self.monitoring_rpc(&request.method, &request.params) {
                Ok(result) => RpcResponse::ok(request.id, result),
                Err(error) => RpcResponse::error(request.id, RpcError::InvalidParams(error)),
            };
        }
        match request.method.as_str() {
            "hello" => {
                let parsed = serde_json::from_value::<HelloRequest>(request.params);
                match parsed {
                    Ok(hello_request) if hello_request.protocol_version == RPC_PROTOCOL_VERSION => {
                        RpcResponse::ok(request.id, json!(hello(hello_request)))
                    }
                    Ok(hello_request) => RpcResponse::error(
                        request.id,
                        RpcError::UnsupportedProtocol(hello_request.protocol_version),
                    ),
                    Err(_) => RpcResponse::error(request.id, RpcError::UnsupportedProtocol(0)),
                }
            }
            "session.open" => {
                let session_id = Uuid::new_v4();
                self.sessions
                    .insert(session_id, ObservationSession::default());
                RpcResponse::ok(
                    request.id,
                    json!(SessionTicket {
                        session_id,
                        mode: Mode::Observe
                    }),
                )
            }
            "session.status" => {
                let id = session_id(&request.params);
                let session = id.and_then(|value| self.sessions.get(&value));
                RpcResponse::ok(
                    request.id,
                    json!({"mode":"observe", "active": session.is_some(), "sessions": self.sessions.len(), "events_seen": session.map_or(0, |value| value.capture_health.events_seen), "events_deduplicated": session.map_or(0, |value| value.duplicate_events), "matches": session.map_or(0, |value| value.matches.len()), "spool_frames": session.map_or(0, |value| value.spool_frames.len()), "harness_quarantine": session.map_or(0, |value| value.harness_health.quarantine_count), "harness_schema_changes": session.map_or(0, |value| value.harness_health.schema_change_count), "harness_truncates": session.map_or(0, |value| value.harness_health.truncate_count), "control_messages_rejected": self.rejected_controls}),
                )
            }
            "session.close" => {
                let id = session_id(&request.params);
                let removed = id.and_then(|value| self.sessions.remove(&value)).is_some();
                RpcResponse::ok(request.id, json!({"closed": removed, "mode":"observe"}))
            }
            "query.why" => {
                let session = session_id(&request.params)
                    .and_then(|id| self.sessions.get(&id))
                    .or_else(|| self.sessions.values().next());
                let Some(session) = session else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("no open session".to_owned()),
                    );
                };
                let Some(path) = request.params.get("path").and_then(Value::as_str) else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("path".to_owned()),
                    );
                };
                RpcResponse::ok(request.id, session.query_why(path))
            }
            "session.ingest_harness" => {
                let Some(id) = session_id(&request.params) else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("session_id".to_owned()),
                    );
                };
                let Some(session) = self.sessions.get_mut(&id) else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("unknown session_id".to_owned()),
                    );
                };
                let Some(raw) = request.params.get("event").cloned() else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("event".to_owned()),
                    );
                };
                let event = match serde_json::from_value::<HarnessEvent>(raw) {
                    Ok(value) => value,
                    Err(error) => {
                        return RpcResponse::error(
                            request.id,
                            RpcError::InvalidParams(error.to_string()),
                        )
                    }
                };
                let assertion_id = session.ingest_harness_event(&event);
                RpcResponse::ok(
                    request.id,
                    json!({"assertion_id": assertion_id, "mode":"observe"}),
                )
            }
            "session.ingest_system" => {
                let Some(id) = session_id(&request.params) else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("session_id".to_owned()),
                    );
                };
                let Some(session) = self.sessions.get_mut(&id) else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("unknown session_id".to_owned()),
                    );
                };
                let Some(raw) = request.params.get("event").cloned() else {
                    return RpcResponse::error(
                        request.id,
                        RpcError::InvalidParams("event".to_owned()),
                    );
                };
                let event = match serde_json::from_value::<SysEvent>(raw) {
                    Ok(value) => value,
                    Err(error) => {
                        return RpcResponse::error(
                            request.id,
                            RpcError::InvalidParams(error.to_string()),
                        )
                    }
                };
                match session.ingest_system_event(&event) {
                    Ok(()) => RpcResponse::ok(
                        request.id,
                        json!({"events_seen": session.capture_health.events_seen, "events_deduplicated": session.duplicate_events, "matches": session.matches.len(), "spool_frames": session.spool_frames.len(), "mode":"observe"}),
                    ),
                    Err(error) => {
                        RpcResponse::error(request.id, RpcError::InvalidParams(error.to_string()))
                    }
                }
            }
            "assist.analyze" => match assistant_analysis(&request.params) {
                Ok(result) => RpcResponse::ok(request.id, result),
                Err(error) => RpcResponse::error(request.id, RpcError::InvalidParams(error)),
            },
            method
                if method == "advise" || method == "enforce" || method.starts_with("decision") =>
            {
                RpcResponse::error(request.id, RpcError::InterventionUnavailable)
            }
            method => RpcResponse::error(request.id, RpcError::MethodNotFound(method.to_owned())),
        }
    }

    pub fn session_mut(&mut self, session_id: Uuid) -> Option<&mut ObservationSession> {
        self.sessions.get_mut(&session_id)
    }
}

fn assistant_analysis(params: &Value) -> Result<Value, String> {
    let config = params
        .get("config")
        .cloned()
        .map(serde_json::from_value::<AssistantConfig>)
        .transpose()
        .map_err(|error| format!("invalid assistant config: {error}"))?
        .unwrap_or_default();
    let request = params
        .get("request")
        .cloned()
        .ok_or_else(|| "assistant request is required".to_owned())
        .and_then(|value| {
            serde_json::from_value::<AssistantRequest>(value)
                .map_err(|error| format!("invalid assistant request: {error}"))
        })?;
    let api_key = params.get("api_key").and_then(Value::as_str);
    let result =
        analyze_with_assistant(&config, api_key, &request).map_err(|error| error.to_string())?;
    serde_json::to_value(result).map_err(|error| error.to_string())
}

fn session_id(params: &Value) -> Option<Uuid> {
    params
        .get("session_id")
        .and_then(Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorBody>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcErrorBody {
    pub code: String,
    pub message: String,
}

impl RpcResponse {
    fn ok(id: u64, result: Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: u64, error: RpcError) -> Self {
        let code = match &error {
            RpcError::UnsupportedProtocol(_) => "unsupported_protocol",
            RpcError::InterventionUnavailable => "intervention_unavailable",
            RpcError::MethodNotFound(_) => "method_not_found",
            RpcError::InvalidParams(_) => "invalid_params",
        };
        Self {
            id,
            result: None,
            error: Some(RpcErrorBody {
                code: code.to_owned(),
                message: error.to_string(),
            }),
        }
    }
}

pub fn hello(request: HelloRequest) -> HelloResponse {
    HelloResponse {
        protocol_version: request.protocol_version,
        product: PRODUCT_NAME.to_owned(),
        version: DEV_VERSION.to_owned(),
        mode: Mode::Observe,
    }
}

/// Handle one newline-delimited JSON-RPC request without performing I/O.
/// Transport implementations (Named Pipe in G5) can reuse this contract.
pub fn handle_request(request: RpcRequest) -> RpcResponse {
    let mut state = DaemonState::new();
    state.handle(request)
}

pub fn handle_json_line(line: &[u8]) -> Result<Vec<u8>, serde_json::Error> {
    let request: RpcRequest = serde_json::from_slice(line)?;
    serde_json::to_vec(&handle_request(request))
}

#[cfg(windows)]
pub fn run_named_pipe() -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{BufReader, Write};
    use std::os::windows::io::FromRawHandle;
    use std::ptr::null_mut;

    type Handle = *mut std::ffi::c_void;
    const INVALID_HANDLE_VALUE: Handle = (-1isize) as Handle;
    const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;
    const PIPE_TYPE_BYTE: u32 = 0x0000_0000;
    const PIPE_READMODE_BYTE: u32 = 0x0000_0000;
    const PIPE_WAIT: u32 = 0x0000_0000;
    const ERROR_PIPE_CONNECTED: u32 = 535;
    const SDDL_REVISION_1: u32 = 1;

    #[repr(C)]
    struct SecurityAttributes {
        length: u32,
        descriptor: *mut std::ffi::c_void,
        inherit_handle: i32,
    }

    unsafe extern "system" {
        fn CreateNamedPipeW(
            name: *const u16,
            open_mode: u32,
            pipe_mode: u32,
            max_instances: u32,
            out_size: u32,
            in_size: u32,
            timeout: u32,
            security: *mut std::ffi::c_void,
        ) -> Handle;
        fn ConnectNamedPipe(pipe: Handle, overlapped: *mut std::ffi::c_void) -> i32;
        fn GetLastError() -> u32;
        fn DisconnectNamedPipe(pipe: Handle) -> i32;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            string_security_descriptor: *const u16,
            string_sd_revision: u32,
            security_descriptor: *mut *mut std::ffi::c_void,
            security_descriptor_size: *mut u32,
        ) -> i32;
        fn LocalFree(memory: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }

    fn owner_only_security_attributes() -> std::io::Result<(SecurityAttributes, Handle)> {
        // OW resolves to the creating user's owner SID, keeping the pipe
        // private to the current user without granting Users/Everyone access.
        let mut sddl: Vec<u16> = "D:P(A;;GA;;;OW)".encode_utf16().collect();
        sddl.push(0);
        let mut descriptor: Handle = null_mut();
        let mut descriptor_size = 0_u32;
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                &mut descriptor_size,
            )
        };
        if ok == 0 || descriptor.is_null() || descriptor_size == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok((
            SecurityAttributes {
                length: std::mem::size_of::<SecurityAttributes>() as u32,
                descriptor,
                inherit_handle: 0,
            },
            descriptor,
        ))
    }

    let mut wide: Vec<u16> = clens_types::RPC_PIPE_NAME.encode_utf16().collect();
    wide.push(0);
    let mut state = DaemonState::new();
    loop {
        let (security, descriptor) = owner_only_security_attributes()?;
        let handle = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | 0x0000_0008,
                1,
                65_536,
                65_536,
                0,
                &security as *const SecurityAttributes as *mut std::ffi::c_void,
            )
        };
        unsafe {
            LocalFree(descriptor);
        }
        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }
        let connected = unsafe { ConnectNamedPipe(handle, null_mut()) } != 0
            || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
        if connected {
            let mut reader = BufReader::new(unsafe { File::from_raw_handle(handle) });
            while let Ok(Some(line)) = read_bounded_line(&mut reader, RPC_MAX_MESSAGE_BYTES) {
                let response = match serde_json::from_slice::<RpcRequest>(&line) {
                    Ok(request) => serde_json::to_vec(&state.handle(request))
                        .map_err(|error| std::io::Error::other(error.to_string()))?,
                    Err(error) => serde_json::to_vec(
                        &json!({"id":0,"error":{"code":"invalid_json","message":error.to_string()}}),
                    )?,
                };
                if reader
                    .get_mut()
                    .write_all(&response)
                    .and_then(|()| reader.get_mut().write_all(b"\n"))
                    .and_then(|()| reader.get_mut().flush())
                    .is_err()
                {
                    break;
                }
            }
            let _ = unsafe { DisconnectNamedPipe(handle) };
        } else {
            let _file = unsafe { File::from_raw_handle(handle) };
        }
    }
}

#[cfg(windows)]
fn read_bounded_line<R: std::io::Read>(
    reader: &mut R,
    limit: usize,
) -> std::io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match reader.read(&mut byte)? {
            0 if line.is_empty() => return Ok(None),
            0 => return Ok(Some(line)),
            _ if byte[0] == b'\n' => return Ok(Some(line)),
            _ => {
                if line.len() >= limit {
                    while reader.read(&mut byte)? > 0 && byte[0] != b'\n' {}
                    return Ok(Some(Vec::new()));
                }
                line.push(byte[0]);
            }
        }
    }
}

#[cfg(unix)]
pub fn run_unix_socket() -> std::io::Result<()> {
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    let path = std::env::var("CAUSATIVELENS_RPC_SOCKET")
        .unwrap_or_else(|_| "/tmp/causativelens-rpc".to_owned());
    match fs::remove_file(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let listener = UnixListener::bind(&path)?;
    let mut state = DaemonState::new();
    for stream in listener.incoming() {
        let stream = stream?;
        let mut reader = BufReader::new(stream);
        let mut line = Vec::new();
        while reader.read_until(b'\n', &mut line)? > 0 {
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            let response = match serde_json::from_slice::<RpcRequest>(&line) {
                Ok(request) => serde_json::to_vec(&state.handle(request))
                    .map_err(|error| std::io::Error::other(error.to_string()))?,
                Err(error) => format!(
                    r#"{{"id":0,"error":{{"code":"invalid_json","message":"{}"}}}}"#,
                    error
                )
                .into_bytes(),
            };
            let stream = reader.get_mut();
            stream.write_all(&response)?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            line.clear();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_and_session_contracts_are_observe_only() {
        let response = handle_request(RpcRequest {
            id: 7,
            method: "hello".to_owned(),
            params: json!({"protocol_version": 1, "mode": "observe"}),
        });
        assert!(response.error.is_none());
        let session = handle_request(RpcRequest {
            id: 8,
            method: "session.open".to_owned(),
            params: Value::Null,
        });
        assert!(session.result.is_some());
    }

    #[test]
    fn daemon_thread_plan_caps_assertion_workers_at_four() {
        assert_eq!(daemon_thread_plan(1).assertion_workers, 1);
        assert_eq!(daemon_thread_plan(8).assertion_workers, 4);
        assert_eq!(daemon_thread_plan(32).assertion_workers, 4);
    }

    #[test]
    fn intervention_methods_have_stable_error_code() {
        for method in ["advise", "enforce", "decision.open"] {
            let response = handle_request(RpcRequest {
                id: 1,
                method: method.to_owned(),
                params: Value::Null,
            });
            assert_eq!(
                response.error.as_ref().map(|error| error.code.as_str()),
                Some("intervention_unavailable")
            );
        }
    }

    #[test]
    fn stateful_session_ticket_can_be_statused_and_closed() {
        let mut state = DaemonState::new();
        let opened = state.handle(RpcRequest {
            id: 1,
            method: "session.open".to_owned(),
            params: Value::Null,
        });
        let ticket: SessionTicket =
            serde_json::from_value(opened.result.expect("ticket")).expect("ticket shape");
        assert_eq!(state.session_count(), 1);
        let status = state.handle(RpcRequest {
            id: 2,
            method: "session.status".to_owned(),
            params: json!({"session_id": ticket.session_id}),
        });
        assert_eq!(
            status
                .result
                .as_ref()
                .and_then(|value| value.get("active"))
                .and_then(Value::as_bool),
            Some(true)
        );
        let closed = state.handle(RpcRequest {
            id: 3,
            method: "session.close".to_owned(),
            params: json!({"session_id": ticket.session_id}),
        });
        assert_eq!(
            closed
                .result
                .as_ref()
                .and_then(|value| value.get("closed"))
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn observation_session_wires_harness_capture_alignment_and_spool() {
        let mut state = DaemonState::new();
        let opened = state.handle(RpcRequest {
            id: 1,
            method: "session.open".to_owned(),
            params: Value::Null,
        });
        let ticket: SessionTicket =
            serde_json::from_value(opened.result.expect("ticket")).expect("ticket shape");
        let session = state.session_mut(ticket.session_id).expect("session");
        let harness = HarnessEvent {
            schema_v: 1,
            ts_utc_ms: 1_000,
            step: 1,
            phase: clens_types::Phase::ToolCall,
            tool: Some(clens_types::ToolCall {
                name: "read".into(),
                arguments: json!({"path":"config.yaml"}),
            }),
            text: "read config.yaml".into(),
            harness_log_id: 1,
            line_no: 1,
            byte_start: 0,
            byte_end: 16,
            raw_json: None,
        };
        let assertion_id = session
            .ingest_harness_event(&harness)
            .expect("rule assertion");
        assert_eq!(
            assertion_id,
            clens_types::stable_assertion_id(harness.harness_log_id, harness.line_no, 0)
        );
        let event = SysEvent {
            schema_v: 1,
            seq: 1,
            ts_qpc: 1_000,
            pid: 10,
            tid: 11,
            kind: clens_types::SysKind::FileRead,
            is_rundown: false,
            payload: clens_types::EventPayload::File {
                path: 1,
                desired_access: 0,
                io_len: Some(1),
                write_attrs: false,
            },
        };
        session.ingest_system_event(&event).expect("ingest event");
        assert_eq!(session.capture_health.events_seen, 1);
        assert_eq!(session.spool_frames.len(), 1);
        assert_eq!(session.matches.len(), 1);
    }

    #[test]
    fn rpc_ingest_methods_drive_the_same_observe_session() {
        let mut state = DaemonState::new();
        let opened = state.handle(RpcRequest {
            id: 1,
            method: "session.open".to_owned(),
            params: Value::Null,
        });
        let ticket: SessionTicket =
            serde_json::from_value(opened.result.expect("ticket")).expect("ticket shape");
        let harness = json!({"schema_v":1,"ts_utc_ms":1000,"step":1,"phase":"tool_call","text":"read config.yaml","tool":{"name":"read","arguments":{"path":"config.yaml"}}});
        let accepted = state.handle(RpcRequest {
            id: 2,
            method: "session.ingest_harness".to_owned(),
            params: json!({"session_id":ticket.session_id,"event":harness}),
        });
        assert_eq!(accepted.error, None);
        let status = state.handle(RpcRequest {
            id: 3,
            method: "session.status".to_owned(),
            params: json!({"session_id":ticket.session_id}),
        });
        assert_eq!(
            status
                .result
                .as_ref()
                .and_then(|value| value.get("active"))
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn query_why_returns_the_five_part_chain_shape() {
        let mut state = DaemonState::new();
        let opened = state.handle(RpcRequest {
            id: 1,
            method: "session.open".to_owned(),
            params: Value::Null,
        });
        let ticket: SessionTicket =
            serde_json::from_value(opened.result.expect("ticket")).expect("ticket shape");
        let accepted = state.handle(RpcRequest {
            id: 2,
            method: "session.ingest_harness".to_owned(),
            params: json!({
                "session_id": ticket.session_id,
                "event": {"schema_v":1,"ts_utc_ms":1000,"step":1,"phase":"tool_call","text":"read config.yaml","tool":{"name":"read","arguments":{"path":"config.yaml"}}}
            }),
        });
        assert!(accepted.error.is_none());
        let response = state.handle(RpcRequest {
            id: 3,
            method: "query.why".to_owned(),
            params: json!({"session_id": ticket.session_id, "path": "config.yaml"}),
        });
        let result = response.result.expect("query result");
        for key in [
            "trigger_assertion",
            "evidence_events",
            "verdict",
            "reverse_attribution",
            "confidence_health",
        ] {
            assert!(result
                .get("chain")
                .and_then(|chain| chain.get(key))
                .is_some());
        }
    }

    #[test]
    fn assistant_rpc_is_disabled_without_opt_in_and_never_echoes_secret() {
        let mut state = DaemonState::new();
        let response = state.handle(RpcRequest {
            id: 44,
            method: "assist.analyze".to_owned(),
            params: json!({
                "api_key": "super-secret-test-key",
                "config": {"enabled": false},
                "request": {"query":"why","chain":{},"language":"en-US"}
            }),
        });
        let body = serde_json::to_string(&response).expect("response json");
        assert!(body.contains("assistant is disabled"));
        assert!(!body.contains("super-secret-test-key"));
    }

    #[test]
    fn assistant_rpc_rejects_malformed_request_before_provider_access() {
        let mut state = DaemonState::new();
        let response = state.handle(RpcRequest {
            id: 45,
            method: "assist.analyze".to_owned(),
            params: json!({
                "config": {"enabled": true, "endpoint": "https://example.invalid", "model": "jev-test"},
                "request": {"query":"why"}
            }),
        });
        assert_eq!(
            response.error.as_ref().map(|error| error.code.as_str()),
            Some("invalid_params")
        );
        assert!(response
            .error
            .as_ref()
            .is_some_and(|error| error.message.contains("invalid assistant request")));
    }

    #[test]
    fn duplicate_system_sequence_is_counted_once_and_reported_honestly() {
        let mut session = ObservationSession::default();
        let event = SysEvent {
            schema_v: 1,
            seq: 42,
            ts_qpc: 1_000,
            pid: 10,
            tid: 11,
            kind: clens_types::SysKind::FileRead,
            is_rundown: false,
            payload: clens_types::EventPayload::File {
                path: 1,
                desired_access: 0,
                io_len: Some(1),
                write_attrs: false,
            },
        };
        session.ingest_system_event(&event).expect("first event");
        session
            .ingest_system_event(&event)
            .expect("duplicate event");
        assert_eq!(session.capture_health.events_seen, 1);
        assert_eq!(session.capture_health.events_deduplicated, 1);
        assert_eq!(session.spool_frames.len(), 1);
    }
}
