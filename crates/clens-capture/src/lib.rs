use clens_types::{
    decode_capture_control, CaptureControlMessage, CaptureHeartbeat, Endpoint, EventPayload,
    NetProto, SysEvent, SysKind, ETW_SESSION_PREFIX,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

pub const DATA_RING_MAGIC: u32 = 0x434C_5247;
pub const DATA_RING_SCHEMA_V1: u16 = 1;
pub const DATA_RING_CAPACITY_BYTES: u64 = 256 * 1024 * 1024;
pub const DATA_RING_SPOOL_WATERMARK_PERCENT: u64 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct RingHeader {
    pub magic: u32,
    pub schema_v: u16,
    pub flags: u16,
    pub writer_pos: u64,
    pub reader_pos: u64,
    pub dropped: u64,
    pub heartbeat: u64,
    pub reserved: u64,
}

impl Default for RingHeader {
    fn default() -> Self {
        Self {
            magic: DATA_RING_MAGIC,
            schema_v: DATA_RING_SCHEMA_V1,
            flags: 0,
            writer_pos: 0,
            reader_pos: 0,
            dropped: 0,
            heartbeat: 0,
            reserved: 0,
        }
    }
}

impl RingHeader {
    pub const ENCODED_LEN: usize = 48;

    pub fn encode(self) -> [u8; Self::ENCODED_LEN] {
        let mut bytes = [0_u8; Self::ENCODED_LEN];
        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4..6].copy_from_slice(&self.schema_v.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.flags.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.writer_pos.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.reader_pos.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.dropped.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.heartbeat.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.reserved.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::ENCODED_LEN {
            return None;
        }
        let read_u16 = |start: usize| -> Option<u16> {
            Some(u16::from_le_bytes(
                bytes.get(start..start + 2)?.try_into().ok()?,
            ))
        };
        let read_u32 = |start: usize| -> Option<u32> {
            Some(u32::from_le_bytes(
                bytes.get(start..start + 4)?.try_into().ok()?,
            ))
        };
        let read_u64 = |start: usize| -> Option<u64> {
            Some(u64::from_le_bytes(
                bytes.get(start..start + 8)?.try_into().ok()?,
            ))
        };
        Some(Self {
            magic: read_u32(0)?,
            schema_v: read_u16(4)?,
            flags: read_u16(6)?,
            writer_pos: read_u64(8)?,
            reader_pos: read_u64(16)?,
            dropped: read_u64(24)?,
            heartbeat: read_u64(32)?,
            reserved: read_u64(40)?,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.magic == DATA_RING_MAGIC
            && self.schema_v == DATA_RING_SCHEMA_V1
            && self.reader_pos <= self.writer_pos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingWriteAction {
    Ring,
    Spool,
}

/// Single-writer/single-reader occupancy policy. At 95% the capture hot path
/// switches to `.clspool`; it never blocks an ETW consumer or silently drops.
pub fn ring_write_action(header: RingHeader, frame_len: u64) -> RingWriteAction {
    if frame_len == 0 || frame_len > DATA_RING_CAPACITY_BYTES {
        return RingWriteAction::Spool;
    }
    let used = header.writer_pos.saturating_sub(header.reader_pos);
    let watermark =
        DATA_RING_CAPACITY_BYTES.saturating_mul(DATA_RING_SPOOL_WATERMARK_PERCENT) / 100;
    if used >= watermark || used.saturating_add(frame_len) > DATA_RING_CAPACITY_BYTES {
        RingWriteAction::Spool
    } else {
        RingWriteAction::Ring
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureControlAction {
    Start { session_id: String },
    Stop { session_id: String },
    Heartbeat { nonce: u64 },
    Metrics,
}

#[derive(Debug)]
pub struct CaptureControlGate {
    pub heartbeat: CaptureHeartbeat,
    pub rejected_messages: u64,
}

impl CaptureControlGate {
    pub fn new(now_ms: i64) -> Self {
        Self {
            heartbeat: CaptureHeartbeat::new(now_ms),
            rejected_messages: 0,
        }
    }

    /// Apply only capture lifecycle messages. Policy/injection commands and
    /// producer-side error notifications are rejected and counted.
    pub fn accept(&mut self, bytes: &[u8], now_ms: i64) -> Result<CaptureControlAction, String> {
        let message = match decode_capture_control(bytes) {
            Ok(message) => message,
            Err(error) => {
                self.rejected_messages = self.rejected_messages.saturating_add(1);
                return Err(format!("invalid capture control: {error:?}"));
            }
        };
        let action = match message {
            CaptureControlMessage::Start { session_id } => {
                CaptureControlAction::Start { session_id }
            }
            CaptureControlMessage::Stop { session_id } => CaptureControlAction::Stop { session_id },
            CaptureControlMessage::Heartbeat { nonce } => {
                self.heartbeat.observe(now_ms);
                CaptureControlAction::Heartbeat { nonce }
            }
            CaptureControlMessage::Metrics => CaptureControlAction::Metrics,
            CaptureControlMessage::Error { .. } => {
                self.rejected_messages = self.rejected_messages.saturating_add(1);
                return Err("capture does not accept producer error messages".to_owned());
            }
        };
        Ok(action)
    }
}
use std::sync::RwLock;

pub const PROVIDERS: &[(&str, &str)] = &[
    ("Process", "Microsoft-Windows-Kernel-Process"),
    ("File", "Microsoft-Windows-Kernel-File"),
    ("Registry", "Microsoft-Windows-Kernel-Registry"),
    ("Network", "Microsoft-Windows-Kernel-Network"),
];

/// Machine-verified provider identity and the manifest keyword names that
/// the capture policy is allowed to enable. Numeric masks deliberately stay
/// in the Windows-runner self-check until the installed manifest is queried;
/// a remembered mask is worse than a failed startup because it can silently
/// produce an empty or misleading trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderContract {
    pub logical_name: &'static str,
    pub provider_name: &'static str,
    pub guid: &'static str,
    pub keyword_names: &'static [&'static str],
}

pub const PROVIDER_CONTRACTS: &[ProviderContract] = &[
    ProviderContract {
        logical_name: "Process",
        provider_name: "Microsoft-Windows-Kernel-Process",
        guid: "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716",
        keyword_names: &["WINEVENT_KEYWORD_PROCESS", "THREAD", "IMAGE"],
    },
    ProviderContract {
        logical_name: "File",
        provider_name: "Microsoft-Windows-Kernel-File",
        guid: "edd08927-9cc4-4e65-b970-c2560fb5c289",
        keyword_names: &["FILE_KEYWORD_CREATE", "READ", "WRITE", "CLOSE", "RUNDOWN"],
    },
    ProviderContract {
        logical_name: "Registry",
        provider_name: "Microsoft-Windows-Kernel-Registry",
        guid: "70eb4f03-c1de-4f73-a051-33d13d5413bd",
        keyword_names: &["REGISTRY_READ", "REGISTRY_WRITE"],
    },
    ProviderContract {
        logical_name: "Network",
        provider_name: "Microsoft-Windows-Kernel-Network",
        guid: "7dd42a49-5329-4832-8dfd-43d979153a88",
        keyword_names: &[
            "NETWORK_CONNECT",
            "NETWORK_DISCONNECT",
            "NETWORK_SEND",
            "NETWORK_RECV",
        ],
    },
];

pub fn provider_contracts() -> &'static [ProviderContract] {
    PROVIDER_CONTRACTS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureIntensity {
    Full,
    Lean,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeywordPolicy {
    pub intensity: CaptureIntensity,
    pub process: &'static [&'static str],
    pub file: &'static [&'static str],
    pub registry: &'static [&'static str],
    pub network: &'static [&'static str],
}

pub fn keyword_policy(intensity: CaptureIntensity) -> KeywordPolicy {
    let all = provider_contracts();
    let process = all[0].keyword_names;
    let file = match intensity {
        CaptureIntensity::Full => all[1].keyword_names,
        CaptureIntensity::Lean => &["FILE_KEYWORD_CREATE", "WRITE", "CLOSE", "RUNDOWN"],
        CaptureIntensity::Minimal => &["FILE_KEYWORD_CREATE", "WRITE", "CLOSE"],
    };
    KeywordPolicy {
        intensity,
        process,
        file,
        registry: all[2].keyword_names,
        network: all[3].keyword_names,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtwSessionSpec {
    pub name: String,
    pub provider: String,
    pub keywords: u64,
}

pub fn session_specs() -> Vec<EtwSessionSpec> {
    sessions()
        .into_iter()
        .map(|session| EtwSessionSpec {
            name: session.session_name,
            provider: session.provider_name.to_owned(),
            keywords: 0xFFFF_FFFF_FFFF_FFFF,
        })
        .collect()
}

pub fn logman_create_args(spec: &EtwSessionSpec) -> Vec<String> {
    vec![
        "create".into(),
        "trace".into(),
        spec.name.clone(),
        "-o".into(),
        format!(r"%LOCALAPPDATA%\causativelens\{}.etl", spec.name),
        "-p".into(),
        spec.provider.clone(),
        format!("0x{:X}", spec.keywords),
        "-ets".into(),
    ]
}

#[cfg(windows)]
pub fn start_session(spec: &EtwSessionSpec) -> std::io::Result<std::process::Output> {
    std::process::Command::new("logman")
        .args(logman_create_args(spec))
        .output()
}

#[cfg(windows)]
pub fn stop_session(name: &str) -> std::io::Result<std::process::Output> {
    std::process::Command::new("logman")
        .args(["stop", name, "-ets"])
        .output()
}

#[cfg(windows)]
fn delete_session(name: &str) -> std::io::Result<std::process::Output> {
    std::process::Command::new("logman")
        .args(["delete", name])
        .output()
}

/// Stops and removes remnants from a previous crashed capture. A failed stop
/// is intentionally ignored when the session is already gone; a failed
/// delete is surfaced because reusing a stale session would change the
/// provider/keyword contract underneath the caller.
#[cfg(windows)]
pub fn cleanup_stale_sessions() -> std::io::Result<()> {
    for session in sessions() {
        let _ = stop_session(&session.session_name);
        let output = delete_session(&session.session_name)?;
        if !output.status.success()
            && !String::from_utf8_lossy(&output.stderr)
                .to_ascii_lowercase()
                .contains("not found")
        {
            return Err(std::io::Error::other(format!(
                "stale ETW session {} could not be removed: {}",
                session.session_name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn cleanup_stale_sessions() -> std::io::Result<()> {
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSession {
    pub logical_name: &'static str,
    pub provider_name: &'static str,
    pub session_name: String,
}

pub fn sessions() -> Vec<ProviderSession> {
    PROVIDERS
        .iter()
        .map(|(logical_name, provider_name)| ProviderSession {
            logical_name,
            provider_name,
            session_name: format!("{ETW_SESSION_PREFIX}{logical_name}"),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    MissingField(&'static str),
    WrongType(&'static str),
}

pub trait TdhOracle {
    fn decode(&self, record: &Value) -> Result<SysEvent, DecodeError>;
}

pub fn decode_process_record(record: &Value) -> Result<SysEvent, DecodeError> {
    let number = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_u64)
            .ok_or(DecodeError::MissingField(name))
    };
    let text = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_str)
            .ok_or(DecodeError::MissingField(name))
    };
    let pid = u32::try_from(number("pid")?).map_err(|_| DecodeError::WrongType("pid"))?;
    let tid = u32::try_from(number("tid")?).map_err(|_| DecodeError::WrongType("tid"))?;
    let ppid = u32::try_from(number("ppid")?).map_err(|_| DecodeError::WrongType("ppid"))?;
    Ok(SysEvent {
        schema_v: 1,
        seq: number("seq")?,
        ts_qpc: number("ts_qpc")?,
        pid,
        tid,
        kind: SysKind::ProcStart,
        is_rundown: record
            .get("is_rundown")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        payload: EventPayload::Proc {
            ppid,
            image: text("image")?.into(),
            cmd: text("cmd")?.into(),
            exit_code: None,
        },
    })
}

pub fn decode_file_record(record: &Value, kind: SysKind) -> Result<SysEvent, DecodeError> {
    let number = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_u64)
            .ok_or(DecodeError::MissingField(name))
    };
    let path = u32::try_from(number("path_id")?).map_err(|_| DecodeError::WrongType("path_id"))?;
    let pid = u32::try_from(number("pid")?).map_err(|_| DecodeError::WrongType("pid"))?;
    let tid = u32::try_from(number("tid")?).map_err(|_| DecodeError::WrongType("tid"))?;
    Ok(SysEvent {
        schema_v: 1,
        seq: number("seq")?,
        ts_qpc: number("ts_qpc")?,
        pid,
        tid,
        kind,
        is_rundown: record
            .get("is_rundown")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        payload: EventPayload::File {
            path,
            desired_access: u32::try_from(number("desired_access")?)
                .map_err(|_| DecodeError::WrongType("desired_access"))?,
            io_len: record
                .get("io_len")
                .and_then(Value::as_u64)
                .map(u32::try_from)
                .transpose()
                .map_err(|_| DecodeError::WrongType("io_len"))?,
            write_attrs: record
                .get("write_attrs")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
    })
}

pub fn decode_registry_record(record: &Value, kind: SysKind) -> Result<SysEvent, DecodeError> {
    let number = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_u64)
            .ok_or(DecodeError::MissingField(name))
    };
    let key = record
        .get("key")
        .and_then(Value::as_str)
        .ok_or(DecodeError::MissingField("key"))?;
    let pid = u32::try_from(number("pid")?).map_err(|_| DecodeError::WrongType("pid"))?;
    let tid = u32::try_from(number("tid")?).map_err(|_| DecodeError::WrongType("tid"))?;
    Ok(SysEvent {
        schema_v: 1,
        seq: number("seq")?,
        ts_qpc: number("ts_qpc")?,
        pid,
        tid,
        kind,
        is_rundown: false,
        payload: EventPayload::Reg {
            key: key.into(),
            value: record.get("value").and_then(Value::as_str).map(Into::into),
        },
    })
}

pub fn decode_network_record(record: &Value, kind: SysKind) -> Result<SysEvent, DecodeError> {
    let number = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_u64)
            .ok_or(DecodeError::MissingField(name))
    };
    let text = |name: &'static str| {
        record
            .get(name)
            .and_then(Value::as_str)
            .ok_or(DecodeError::MissingField(name))
    };
    let proto = match record.get("proto").and_then(Value::as_str) {
        Some("tcp") => NetProto::Tcp,
        Some("udp") => NetProto::Udp,
        _ => NetProto::Other(0),
    };
    let endpoint = |host: &'static str, port: &'static str| {
        Ok::<Endpoint, DecodeError>(Endpoint {
            host: text(host)?.into(),
            port: u16::try_from(number(port)?).map_err(|_| DecodeError::WrongType(port))?,
        })
    };
    let pid = u32::try_from(number("pid")?).map_err(|_| DecodeError::WrongType("pid"))?;
    let tid = u32::try_from(number("tid")?).map_err(|_| DecodeError::WrongType("tid"))?;
    Ok(SysEvent {
        schema_v: 1,
        seq: number("seq")?,
        ts_qpc: number("ts_qpc")?,
        pid,
        tid,
        kind,
        is_rundown: false,
        payload: EventPayload::Net {
            proto,
            local: endpoint("local_host", "local_port")?,
            remote: endpoint("remote_host", "remote_port")?,
        },
    })
}

pub fn session_name(suffix: &str) -> String {
    format!("{ETW_SESSION_PREFIX}{suffix}")
}

#[derive(Debug, Default)]
pub struct CaptureLifecycle {
    active_sessions: Vec<String>,
    rundown_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureThreadSpec {
    pub name: &'static str,
    pub role: &'static str,
}

pub const fn capture_thread_plan() -> [CaptureThreadSpec; 6] {
    [
        CaptureThreadSpec {
            name: "etw-process",
            role: "ProcessTrace: Microsoft-Windows-Kernel-Process",
        },
        CaptureThreadSpec {
            name: "etw-file",
            role: "ProcessTrace: Microsoft-Windows-Kernel-File",
        },
        CaptureThreadSpec {
            name: "etw-registry",
            role: "ProcessTrace: Microsoft-Windows-Kernel-Registry",
        },
        CaptureThreadSpec {
            name: "etw-network",
            role: "ProcessTrace: Microsoft-Windows-Kernel-Network",
        },
        CaptureThreadSpec {
            name: "spool-writer",
            role: "append CRC frames to .clspool",
        },
        CaptureThreadSpec {
            name: "capture-control",
            role: "UAC lifecycle and rundown control",
        },
    ]
}

impl CaptureLifecycle {
    pub fn active_sessions(&self) -> &[String] {
        &self.active_sessions
    }

    pub fn rundown_requested(&self) -> bool {
        self.rundown_requested
    }

    /// Records the control-plane request that must be followed by provider
    /// rundown keywords on Windows. Keeping the state explicit prevents an
    /// attach caller from accidentally claiming complete process/file state.
    pub fn request_rundown(&mut self) -> std::io::Result<()> {
        if self.active_sessions.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "cannot request rundown before ETW sessions are active",
            ));
        }
        self.rundown_requested = true;
        Ok(())
    }

    #[cfg(windows)]
    pub fn start_all(&mut self) -> std::io::Result<()> {
        cleanup_stale_sessions()?;
        for spec in session_specs() {
            let output = start_session(&spec)?;
            if !output.status.success() {
                let _ = self.stop_all();
                return Err(std::io::Error::other(format!(
                    "ETW session {} failed to start: {}",
                    spec.name,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            self.active_sessions.push(spec.name);
        }
        self.request_rundown()?;
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn start_all(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "ETW capture requires Windows",
        ))
    }

    pub fn stop_all(&mut self) -> std::io::Result<()> {
        let mut first_error = None;
        #[cfg(windows)]
        for name in self.active_sessions.drain(..).rev() {
            match stop_session(&name) {
                Ok(output) if output.status.success() => {}
                Ok(output) => {
                    first_error.get_or_insert_with(|| {
                        std::io::Error::other(format!(
                            "ETW session {name} failed to stop: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ))
                    });
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        #[cfg(not(windows))]
        self.active_sessions.clear();
        self.rundown_requested = false;
        first_error.map_or(Ok(()), Err)
    }
}

#[derive(Debug, Default)]
pub struct FileObjectTable {
    entries: RwLock<HashMap<u64, (u32, String)>>,
}

impl FileObjectTable {
    pub fn insert(&self, object: u64, path_id: u32, path: impl Into<String>) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(object, (path_id, path.into()));
        }
    }

    pub fn path_id(&self, object: u64) -> Option<u32> {
        self.entries.read().ok()?.get(&object).map(|entry| entry.0)
    }

    pub fn rename(&self, object: u64, path_id: u32, path: impl Into<String>) -> bool {
        if let Ok(mut entries) = self.entries.write() {
            if let Some(entry) = entries.get_mut(&object) {
                *entry = (path_id, path.into());
                return true;
            }
        }
        false
    }

    pub fn remove(&self, object: u64) -> bool {
        self.entries
            .write()
            .map(|mut entries| entries.remove(&object).is_some())
            .unwrap_or(false)
    }
}

/// Capture-side path string table. File events carry only the returned
/// `PathIdx`; the table is snapshotted with each spool block so replay can
/// materialize paths without copying the same UTF-16/UTF-8 text per event.
#[derive(Debug, Default)]
pub struct PathInterner {
    by_path: HashMap<Box<str>, u32>,
    paths: Vec<Box<str>>,
}

impl PathInterner {
    pub fn intern(&mut self, path: &str) -> u32 {
        if let Some(index) = self.by_path.get(path) {
            return *index;
        }
        let index = u32::try_from(self.paths.len()).unwrap_or(u32::MAX);
        let owned: Box<str> = path.into();
        if index != u32::MAX {
            self.by_path.insert(owned.clone(), index);
            self.paths.push(owned);
        }
        index
    }

    pub fn resolve(&self, index: u32) -> Option<&str> {
        self.paths.get(index as usize).map(AsRef::as_ref)
    }

    pub fn snapshot(&self) -> Vec<Box<str>> {
        self.paths.clone()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

/// Process-local monotonic sequence allocator. The spool writer remains the
/// serialization point, while this allocator guarantees that every decoded
/// event has an ID before it crosses the capture boundary.
#[derive(Debug, Default)]
pub struct SequenceAllocator {
    next: AtomicU64,
}

impl SequenceAllocator {
    pub fn next(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed).saturating_add(1)
    }

    pub fn assign(&self, mut event: SysEvent) -> SysEvent {
        event.seq = self.next();
        event
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CaptureHealth {
    pub events_seen: u64,
    pub events_lost: u64,
    pub spool_gaps: u64,
    pub events_deduplicated: u64,
    pub real_time_buffers_lost: u64,
    pub buffer_loss_events: u64,
    pub dropped_events: u64,
    pub ring_spool_frames: u64,
    pub control_messages_rejected: u64,
    pub capture_failure_code: Option<&'static str>,
}

impl CaptureHealth {
    pub fn record_etw_loss(&mut self, events_lost: u64, buffers_lost: u64) {
        self.events_lost = self.events_lost.saturating_add(events_lost);
        self.real_time_buffers_lost = self.real_time_buffers_lost.saturating_add(buffers_lost);
        if events_lost > 0 || buffers_lost > 0 {
            self.buffer_loss_events = self.buffer_loss_events.saturating_add(1);
        }
    }

    pub fn record_drop(&mut self, count: u64) {
        self.dropped_events = self.dropped_events.saturating_add(count);
    }

    pub fn record_ring_spool(&mut self) {
        self.ring_spool_frames = self.ring_spool_frames.saturating_add(1);
    }

    pub fn record_control_rejection(&mut self) {
        self.control_messages_rejected = self.control_messages_rejected.saturating_add(1);
    }

    pub fn record_capture_failure(&mut self) {
        self.capture_failure_code = Some(clens_types::CAPTURE_FAILURE_CODE);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CaptureFilter {
    include_only: Option<HashSet<u32>>,
    excluded_pids: HashSet<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    Capture,
    Excluded,
}

impl CaptureFilter {
    pub fn include_only<I>(&mut self, pids: I)
    where
        I: IntoIterator<Item = u32>,
    {
        self.include_only = Some(pids.into_iter().collect());
    }

    pub fn exclude_pid(&mut self, pid: u32) {
        self.excluded_pids.insert(pid);
    }

    pub fn decision(&self, pid: u32) -> FilterDecision {
        if self.excluded_pids.contains(&pid)
            || self
                .include_only
                .as_ref()
                .is_some_and(|pids| !pids.contains(&pid))
        {
            FilterDecision::Excluded
        } else {
            FilterDecision::Capture
        }
    }

    pub fn excluded_pids(&self) -> impl Iterator<Item = &u32> {
        self.excluded_pids.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockAnchor {
    pub qpc: u64,
    pub utc_100ns: u64,
}

impl ClockAnchor {
    /// Projects an ETW QPC value into the UTC FILETIME 100 ns domain. The
    /// arithmetic is saturating because a malformed provider timestamp must
    /// not take down the capture process.
    pub fn qpc_to_utc_100ns(self, qpc: u64, qpc_per_second: u64) -> u64 {
        let frequency = qpc_per_second.max(1);
        if qpc >= self.qpc {
            self.utc_100ns.saturating_add(
                qpc.saturating_sub(self.qpc)
                    .saturating_mul(10_000_000)
                    .saturating_div(frequency),
            )
        } else {
            self.utc_100ns.saturating_sub(
                self.qpc
                    .saturating_sub(qpc)
                    .saturating_mul(10_000_000)
                    .saturating_div(frequency),
            )
        }
    }

    pub fn qpc_to_utc_ms(self, qpc: u64, qpc_per_second: u64) -> i64 {
        i64::try_from(self.qpc_to_utc_100ns(qpc, qpc_per_second) / 10_000).unwrap_or(i64::MAX)
    }
}

pub const REORDER_WINDOW_MS: u64 = 64;
pub const CLOCK_DRIFT_THRESHOLD_100NS_PER_SEC: i64 = 2_000;

/// Buffers ETW events until the configured QPC disorder window has elapsed.
/// The buffer is intentionally independent from ETW and can be fed by a TDH
/// decoder or a fixture in exactly the same way.
#[derive(Debug)]
pub struct ReorderBuffer {
    window_qpc: u64,
    max_seen_qpc: Option<u64>,
    pending: Vec<SysEvent>,
}

impl ReorderBuffer {
    pub fn with_qpc_frequency(qpc_per_second: u64) -> Self {
        let window_qpc = qpc_per_second
            .saturating_mul(REORDER_WINDOW_MS)
            .saturating_div(1_000)
            .max(1);
        Self {
            window_qpc,
            max_seen_qpc: None,
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, event: SysEvent) -> Vec<SysEvent> {
        self.max_seen_qpc = Some(
            self.max_seen_qpc
                .map_or(event.ts_qpc, |value| value.max(event.ts_qpc)),
        );
        self.pending.push(event);
        self.drain_ready()
    }

    pub fn flush_all(&mut self) -> Vec<SysEvent> {
        self.pending.sort_by_key(|event| (event.ts_qpc, event.seq));
        std::mem::take(&mut self.pending)
    }

    fn drain_ready(&mut self) -> Vec<SysEvent> {
        let Some(max_seen_qpc) = self.max_seen_qpc else {
            return Vec::new();
        };
        let watermark = max_seen_qpc.saturating_sub(self.window_qpc);
        self.pending.sort_by_key(|event| (event.ts_qpc, event.seq));
        let split = self
            .pending
            .partition_point(|event| event.ts_qpc <= watermark);
        self.pending.drain(..split).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockCalibration {
    pub anchor: ClockAnchor,
    pub drift_100ns_per_sec: i64,
    pub refreshed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockCalibrator {
    pub anchor: ClockAnchor,
    pub qpc_per_second: u64,
    pub last_sample: Option<(u64, u64)>,
}

impl ClockCalibrator {
    pub fn new(anchor: ClockAnchor, qpc_per_second: u64) -> Self {
        Self {
            anchor,
            qpc_per_second: qpc_per_second.max(1),
            last_sample: None,
        }
    }

    pub fn observe(&mut self, qpc: u64, utc_100ns: u64) -> ClockCalibration {
        let Some((previous_qpc, previous_utc)) = self.last_sample.replace((qpc, utc_100ns)) else {
            return ClockCalibration {
                anchor: self.anchor,
                drift_100ns_per_sec: 0,
                refreshed: false,
            };
        };
        let qpc_delta = qpc.saturating_sub(previous_qpc);
        let utc_delta = utc_100ns.saturating_sub(previous_utc);
        let expected = qpc_delta.saturating_mul(10_000_000) / self.qpc_per_second;
        let drift = i64::try_from(utc_delta).unwrap_or(i64::MAX)
            - i64::try_from(expected).unwrap_or(i64::MAX);
        let drift_per_second = if qpc_delta == 0 {
            0
        } else {
            drift.saturating_mul(i64::try_from(self.qpc_per_second).unwrap_or(i64::MAX))
                / i64::try_from(qpc_delta).unwrap_or(i64::MAX).max(1)
        };
        let refreshed = drift_per_second.abs() > CLOCK_DRIFT_THRESHOLD_100NS_PER_SEC;
        if refreshed {
            self.anchor = ClockAnchor { qpc, utc_100ns };
        }
        ClockCalibration {
            anchor: self.anchor,
            drift_100ns_per_sec: drift_per_second,
            refreshed,
        }
    }

    pub fn utc_100ns(&self, qpc: u64) -> u64 {
        let delta = qpc.saturating_sub(self.anchor.qpc);
        self.anchor
            .utc_100ns
            .saturating_add(delta.saturating_mul(10_000_000) / self.qpc_per_second.max(1))
    }
}

pub fn choose_clock_anchor(samples: &[(u64, u64, u64)]) -> Option<ClockAnchor> {
    samples
        .iter()
        .min_by_key(|(before, _, after)| after.saturating_sub(*before))
        .map(|(before, utc, after)| ClockAnchor {
            qpc: before.saturating_add(after.saturating_sub(*before) / 2),
            utc_100ns: *utc,
        })
}

pub fn choose_clock_anchor_16(samples: &[(u64, u64, u64)]) -> Option<ClockAnchor> {
    (samples.len() == 16)
        .then(|| choose_clock_anchor(samples))
        .flatten()
}

impl CaptureHealth {
    #[allow(clippy::manual_checked_ops)]
    pub fn loss_ratio_millionths(self) -> u64 {
        let total = self.events_seen.saturating_add(self.events_lost);
        if total == 0 {
            0
        } else {
            self.events_lost.saturating_mul(1_000_000) / total
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn session_names_use_new_product_prefix() {
        assert_eq!(session_name("capture"), "CausativeLens_capture");
    }

    #[test]
    fn four_provider_sessions_use_new_prefix() {
        let names: Vec<_> = sessions().into_iter().map(|s| s.session_name).collect();
        assert_eq!(
            names,
            [
                "CausativeLens_Process",
                "CausativeLens_File",
                "CausativeLens_Registry",
                "CausativeLens_Network"
            ]
        );
    }

    #[test]
    fn provider_contracts_match_the_four_session_names_and_have_keyword_whitelists() {
        assert_eq!(provider_contracts().len(), 4);
        for contract in provider_contracts() {
            assert!(contract
                .provider_name
                .starts_with("Microsoft-Windows-Kernel-"));
            assert!(!contract.guid.is_empty());
            assert!(!contract.keyword_names.is_empty());
        }
        assert_eq!(keyword_policy(CaptureIntensity::Lean).file.len(), 4);
        assert_eq!(keyword_policy(CaptureIntensity::Minimal).file.len(), 3);
    }

    #[test]
    fn ring_header_round_trip_and_magic_validation_are_deterministic() {
        let header = RingHeader {
            writer_pos: 99,
            reader_pos: 20,
            dropped: 0,
            heartbeat: 7,
            ..RingHeader::default()
        };
        let encoded = header.encode();
        assert_eq!(RingHeader::decode(&encoded), Some(header));
        assert!(header.is_valid());
        let mut corrupt = encoded;
        corrupt[0] ^= 0xFF;
        assert!(!RingHeader::decode(&corrupt).is_some_and(|value| value.is_valid()));
    }

    #[test]
    fn ring_switches_to_spool_at_ninety_five_percent_without_dropping() {
        let watermark = DATA_RING_CAPACITY_BYTES * DATA_RING_SPOOL_WATERMARK_PERCENT / 100;
        let header = RingHeader {
            writer_pos: watermark,
            reader_pos: 0,
            ..RingHeader::default()
        };
        assert_eq!(ring_write_action(header, 1), RingWriteAction::Spool);
        let room = RingHeader {
            writer_pos: 1,
            reader_pos: 0,
            ..RingHeader::default()
        };
        assert_eq!(ring_write_action(room, 1), RingWriteAction::Ring);
    }

    #[test]
    fn capture_control_gate_accepts_lifecycle_and_rejects_policy_messages() {
        let mut gate = CaptureControlGate::new(0);
        let start = serde_json::to_vec(&clens_types::CaptureControlMessage::Start {
            session_id: "session-1".to_owned(),
        })
        .expect("test serialization");
        assert_eq!(
            gate.accept(&start, 1),
            Ok(CaptureControlAction::Start {
                session_id: "session-1".to_owned()
            })
        );
        let rejected = gate.accept(br#"{"type":"error","code":"inject"}"#, 2);
        assert!(rejected.is_err());
        assert_eq!(gate.rejected_messages, 1);
    }

    #[test]
    fn capture_thread_plan_has_four_provider_threads_and_two_control_threads() {
        let plan = capture_thread_plan();
        assert_eq!(plan.len(), 6);
        assert_eq!(plan[0].name, "etw-process");
        assert_eq!(plan[4].name, "spool-writer");
        assert_eq!(plan[5].name, "capture-control");
    }

    #[test]
    fn logman_command_contains_provider_and_session() {
        let spec = &session_specs()[0];
        let args = logman_create_args(spec);
        assert_eq!(args[0..3], ["create", "trace", "CausativeLens_Process"]);
        assert!(args.contains(&"Microsoft-Windows-Kernel-Process".to_owned()));
        assert!(args.contains(&"-ets".to_owned()));
    }

    #[test]
    fn process_decoder_matches_fixture_shape() {
        let event = decode_process_record(&json!({"seq": 1, "ts_qpc": 2, "pid": 3, "tid": 4, "ppid": 0, "image": "agent.exe", "cmd": "agent.exe --test", "is_rundown": true})).expect("fixture decodes");
        assert_eq!(event.kind, SysKind::ProcStart);
        assert!(event.is_rundown);
    }

    #[test]
    fn file_registry_network_decoders_preserve_kind_and_metadata() {
        let file = decode_file_record(&json!({"seq": 1, "ts_qpc": 2, "pid": 3, "tid": 4, "path_id": 8, "desired_access": 1, "io_len": 9, "write_attrs": false}), SysKind::FileRead).expect("file fixture");
        assert_eq!(file.kind, SysKind::FileRead);
        let reg = decode_registry_record(&json!({"seq": 1, "ts_qpc": 2, "pid": 3, "tid": 4, "key": "HKCU\\Software\\Example", "value": "x"}), SysKind::RegQuery).expect("registry fixture");
        assert_eq!(reg.kind, SysKind::RegQuery);
        let net = decode_network_record(&json!({"seq": 1, "ts_qpc": 2, "pid": 3, "tid": 4, "proto": "tcp", "local_host": "127.0.0.1", "local_port": 1, "remote_host": "example.com", "remote_port": 443}), SysKind::NetConnect).expect("network fixture");
        assert_eq!(net.kind, SysKind::NetConnect);
    }

    #[test]
    fn file_object_table_tracks_rename_and_close() {
        let table = FileObjectTable::default();
        table.insert(7, 1, r"C:\old.txt");
        assert_eq!(table.path_id(7), Some(1));
        assert!(table.rename(7, 2, r"C:\new.txt"));
        assert_eq!(table.path_id(7), Some(2));
        assert!(table.remove(7));
        assert_eq!(table.path_id(7), None);
    }

    #[test]
    fn path_interner_reuses_ids_and_snapshots_strings() {
        let mut interner = PathInterner::default();
        let first = interner.intern(r"C:\work\config.yaml");
        let again = interner.intern(r"C:\work\config.yaml");
        let second = interner.intern(r"C:\work\other.yaml");
        assert_eq!(first, again);
        assert_ne!(first, second);
        assert_eq!(interner.resolve(first), Some(r"C:\work\config.yaml"));
        assert_eq!(interner.len(), 2);
        assert_eq!(interner.snapshot().len(), 2);
    }

    #[test]
    fn sequence_allocator_is_monotonic_and_overwrites_decoder_sequence() {
        let allocator = SequenceAllocator::default();
        assert_eq!(allocator.next(), 1);
        assert_eq!(allocator.next(), 2);
        let event = allocator.assign(fixture_event(1, 900));
        assert_eq!(event.seq, 3);
    }

    #[test]
    fn health_reports_loss_without_hiding_gaps() {
        let health = CaptureHealth {
            events_seen: 95,
            events_lost: 5,
            spool_gaps: 2,
            events_deduplicated: 0,
            real_time_buffers_lost: 0,
            buffer_loss_events: 1,
            dropped_events: 0,
            ..CaptureHealth::default()
        };
        assert_eq!(health.loss_ratio_millionths(), 50_000);
        assert_eq!(health.spool_gaps, 2);
    }

    #[test]
    fn health_records_etw_buffer_loss_and_explicit_drops() {
        let mut health = CaptureHealth::default();
        health.record_etw_loss(3, 2);
        health.record_drop(5);
        assert_eq!(health.events_lost, 3);
        assert_eq!(health.real_time_buffers_lost, 2);
        assert_eq!(health.buffer_loss_events, 1);
        assert_eq!(health.dropped_events, 5);
    }

    #[test]
    fn pid_filter_keeps_exclusions_explicit() {
        let mut filter = CaptureFilter::default();
        filter.include_only([10, 20]);
        filter.exclude_pid(20);
        assert_eq!(filter.decision(10), FilterDecision::Capture);
        assert_eq!(filter.decision(20), FilterDecision::Excluded);
        assert_eq!(filter.decision(30), FilterDecision::Excluded);
        assert_eq!(
            filter.excluded_pids().copied().collect::<Vec<_>>(),
            vec![20]
        );
    }

    #[test]
    fn clock_anchor_chooses_lowest_round_trip_sample() {
        let anchor = choose_clock_anchor(&[(100, 1_000, 140), (200, 2_000, 202)]).expect("anchor");
        assert_eq!(
            anchor,
            ClockAnchor {
                qpc: 201,
                utc_100ns: 2_000
            }
        );
    }

    #[test]
    fn clock_anchor_projects_qpc_into_utc_without_mixing_domains() {
        let anchor = ClockAnchor {
            qpc: 1_000,
            utc_100ns: 10_000_000,
        };
        assert_eq!(anchor.qpc_to_utc_100ns(2_000, 1_000), 20_000_000);
        assert_eq!(anchor.qpc_to_utc_ms(2_000, 1_000), 2_000);
    }

    fn fixture_event(ts_qpc: u64, seq: u64) -> SysEvent {
        SysEvent {
            schema_v: 1,
            seq,
            ts_qpc,
            pid: 1,
            tid: 1,
            kind: SysKind::ProcStart,
            is_rundown: false,
            payload: EventPayload::None,
        }
    }

    #[test]
    fn reorder_buffer_holds_the_64ms_tail_and_orders_released_events() {
        let mut buffer = ReorderBuffer::with_qpc_frequency(1_000);
        assert!(buffer.push(fixture_event(100, 1)).is_empty());
        assert!(buffer.push(fixture_event(50, 2)).is_empty());
        let released = buffer.push(fixture_event(200, 3));
        assert_eq!(
            released.iter().map(|event| event.seq).collect::<Vec<_>>(),
            [2, 1]
        );
        assert_eq!(
            buffer
                .flush_all()
                .iter()
                .map(|event| event.seq)
                .collect::<Vec<_>>(),
            [3]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn capture_lifecycle_is_explicitly_windows_only() {
        let mut lifecycle = CaptureLifecycle::default();
        assert_eq!(lifecycle.active_sessions(), &[] as &[String]);
        assert!(lifecycle.request_rundown().is_err());
        assert_eq!(
            lifecycle.start_all().expect_err("non-Windows ETW").kind(),
            std::io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn clock_calibrator_refreshes_when_drift_exceeds_threshold() {
        let anchor = ClockAnchor {
            qpc: 0,
            utc_100ns: 0,
        };
        let mut calibrator = ClockCalibrator::new(anchor, 1_000);
        assert!(!calibrator.observe(1_000, 10_000_000).refreshed);
        let calibration = calibrator.observe(2_000, 20_020_000);
        assert!(calibration.refreshed);
        assert_eq!(calibration.anchor.qpc, 2_000);
        assert_eq!(calibrator.utc_100ns(2_000), 20_020_000);
    }

    #[test]
    fn sixteen_round_anchor_rejects_incomplete_sampling() {
        let samples = vec![(0, 100, 1); 15];
        assert!(choose_clock_anchor_16(&samples).is_none());
        let samples = vec![(0, 100, 1); 16];
        assert_eq!(
            choose_clock_anchor_16(&samples).map(|anchor| anchor.utc_100ns),
            Some(100)
        );
    }
}
