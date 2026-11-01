use clens_types::{
    decode_capture_control, CaptureControlMessage, CaptureHeartbeat, Endpoint, EventPayload,
    NetProto, SysEvent, SysKind, ETW_SESSION_PREFIX,
};
#[cfg(windows)]
mod realtime;
#[cfg(windows)]
pub use realtime::{ManualEventDecoder, RawEtwRecord, RealtimeConsumer};
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
        let diagnostics = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .to_ascii_lowercase();
        if !output.status.success() && !diagnostics.contains("not found") {
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
    attached_pid: Option<u32>,
    #[cfg(windows)]
    realtime_consumers: Vec<RealtimeConsumer>,
    #[cfg(windows)]
    control_sessions: Vec<NativeEtwSession>,
}

#[cfg(windows)]
struct NativeEtwSession {
    name: String,
    name_wide: Vec<u16>,
    handle: windows_sys::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE,
}

#[cfg(windows)]
impl std::fmt::Debug for NativeEtwSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeEtwSession")
            .field("name", &self.name)
            .field("handle", &self.handle.Value)
            .finish()
    }
}

#[cfg(windows)]
fn provider_guid(logical_name: &str) -> Option<windows_sys::core::GUID> {
    match logical_name {
        "Process" => Some(windows_sys::core::GUID::from_u128(
            0x22fb2cd6_0e7b_422b_a0c7_2fad1fd0e716,
        )),
        "File" => Some(windows_sys::core::GUID::from_u128(
            0xedd08927_9cc4_4e65_b970_c2560fb5c289,
        )),
        "Registry" => Some(windows_sys::core::GUID::from_u128(
            0x70eb4f03_c1de_4f73_a051_33d13d5413bd,
        )),
        "Network" => Some(windows_sys::core::GUID::from_u128(
            0x7dd42a49_5329_4832_8dfd_43d979153a88,
        )),
        _ => None,
    }
}

#[cfg(windows)]
fn native_start_session(
    spec: &EtwSessionSpec,
    attached_pid: Option<u32>,
) -> std::io::Result<NativeEtwSession> {
    use std::mem::size_of;
    use windows_sys::Win32::System::Diagnostics::Etw::{
        EnableTraceEx2, StartTraceW, ENABLE_TRACE_PARAMETERS, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
        EVENT_FILTER_DESCRIPTOR, EVENT_FILTER_TYPE_PID, EVENT_TRACE_PROPERTIES,
        EVENT_TRACE_REAL_TIME_MODE, WNODE_FLAG_TRACED_GUID,
    };
    let name_wide: Vec<u16> = spec.name.encode_utf16().chain(std::iter::once(0)).collect();
    let properties_size = size_of::<EVENT_TRACE_PROPERTIES>();
    let property_words = (properties_size + name_wide.len() * 2).div_ceil(size_of::<u64>());
    let mut properties_buffer = vec![0_u64; property_words];
    let properties = properties_buffer
        .as_mut_ptr()
        .cast::<EVENT_TRACE_PROPERTIES>();
    unsafe {
        (*properties).Wnode.BufferSize = (properties_buffer.len() * size_of::<u64>()) as u32;
        (*properties).Wnode.Flags = WNODE_FLAG_TRACED_GUID;
        (*properties).Wnode.ClientContext = 1;
        (*properties).LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
        (*properties).MinimumBuffers = 64;
        (*properties).MaximumBuffers = 256;
        (*properties).LoggerNameOffset = properties_size as u32;
        std::ptr::copy_nonoverlapping(
            name_wide.as_ptr().cast::<u8>(),
            properties_buffer
                .as_mut_ptr()
                .cast::<u8>()
                .add(properties_size),
            name_wide.len() * 2,
        );
    }
    let mut handle = windows_sys::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE::default();
    let start = unsafe { StartTraceW(&mut handle, name_wide.as_ptr(), properties) };
    if start != 0 {
        return Err(std::io::Error::from_raw_os_error(start as i32));
    }
    let guid = provider_guid(
        sessions()
            .iter()
            .find(|session| session.session_name == spec.name)
            .map_or("", |session| session.logical_name),
    )
    .ok_or_else(|| std::io::Error::other("provider GUID is not in the capture contract"))?;
    let pid_filter = attached_pid.map(|pid| vec![pid]);
    let filter = pid_filter.as_ref().map(|pids| EVENT_FILTER_DESCRIPTOR {
        Ptr: pids.as_ptr() as usize as u64,
        Size: (pids.len() * size_of::<u32>()) as u32,
        Type: EVENT_FILTER_TYPE_PID,
    });
    let parameters = filter.as_ref().map(|descriptor| ENABLE_TRACE_PARAMETERS {
        Version: 1,
        EnableProperty: 0,
        ControlFlags: 0,
        SourceId: windows_sys::core::GUID::from_u128(0),
        EnableFilterDesc: descriptor as *const EVENT_FILTER_DESCRIPTOR
            as *mut EVENT_FILTER_DESCRIPTOR,
        FilterDescCount: 1,
    });
    let enable = unsafe {
        EnableTraceEx2(
            handle,
            &guid,
            EVENT_CONTROL_CODE_ENABLE_PROVIDER,
            0xff,
            spec.keywords,
            0,
            0,
            parameters.as_ref().map_or(std::ptr::null(), |value| {
                value as *const ENABLE_TRACE_PARAMETERS
            }),
        )
    };
    if enable != 0 {
        let _ = unsafe {
            windows_sys::Win32::System::Diagnostics::Etw::ControlTraceW(
                handle,
                name_wide.as_ptr(),
                std::ptr::null_mut(),
                windows_sys::Win32::System::Diagnostics::Etw::EVENT_TRACE_CONTROL_STOP,
            )
        };
        return Err(std::io::Error::from_raw_os_error(enable as i32));
    }
    Ok(NativeEtwSession {
        name: spec.name.clone(),
        name_wide,
        handle,
    })
}

#[cfg(windows)]
fn native_stop_session(session: &NativeEtwSession) -> std::io::Result<()> {
    use windows_sys::Win32::System::Diagnostics::Etw::{ControlTraceW, EVENT_TRACE_CONTROL_STOP};
    let result = unsafe {
        ControlTraceW(
            session.handle,
            session.name_wide.as_ptr(),
            std::ptr::null_mut(),
            EVENT_TRACE_CONTROL_STOP,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(result as i32))
    }
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

    pub fn attached_pid(&self) -> Option<u32> {
        self.attached_pid
    }

    /// Records the control-plane request that must be followed by provider
    /// rundown capture-state request on Windows. Keeping the state explicit
    /// prevents an attach caller from accidentally claiming complete
    /// process/file state.
    pub fn request_rundown(&mut self) -> std::io::Result<()> {
        if self.active_sessions.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "cannot request rundown before ETW sessions are active",
            ));
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Diagnostics::Etw::{
                EnableTraceEx2, EVENT_CONTROL_CODE_CAPTURE_STATE,
            };
            for session in &self.control_sessions {
                let Some(logical_name) = sessions()
                    .iter()
                    .find(|candidate| candidate.session_name == session.name)
                    .map(|candidate| candidate.logical_name)
                else {
                    continue;
                };
                // Process and File providers expose the state needed for
                // attach: process/image rundown and FileObject/path rundown.
                // Registry and Network have no equivalent state snapshot in
                // the contract, so they remain live-only.
                if !matches!(logical_name, "Process" | "File") {
                    continue;
                }
                let guid = provider_guid(logical_name).ok_or_else(|| {
                    std::io::Error::other("provider GUID is not in the capture contract")
                })?;
                let result = unsafe {
                    EnableTraceEx2(
                        session.handle,
                        &guid,
                        EVENT_CONTROL_CODE_CAPTURE_STATE,
                        0xff,
                        0,
                        0,
                        0,
                        std::ptr::null(),
                    )
                };
                if result != 0 {
                    return Err(std::io::Error::from_raw_os_error(result as i32));
                }
            }
        }
        self.rundown_requested = true;
        Ok(())
    }

    #[cfg(windows)]
    pub fn start_all(&mut self) -> std::io::Result<()> {
        self.start_all_for_pid(None)
    }

    #[cfg(windows)]
    fn start_all_for_pid(&mut self, attached_pid: Option<u32>) -> std::io::Result<()> {
        cleanup_stale_sessions()?;
        for spec in session_specs() {
            let native = match native_start_session(&spec, attached_pid) {
                Ok(native) => native,
                Err(error) => {
                    let _ = self.stop_all();
                    return Err(error);
                }
            };
            self.control_sessions.push(native);
            self.active_sessions.push(spec.name);
        }
        self.attached_pid = attached_pid;
        Ok(())
    }

    /// Start a real attach capture scoped by the ETW PID filter. The rundown
    /// request is sent after the provider sessions are enabled so process and
    /// file state already present at attach time can be decoded before new
    /// events are aligned.
    #[cfg(windows)]
    pub fn start_attach(
        &mut self,
        pid: u32,
    ) -> std::io::Result<std::sync::mpsc::Receiver<RawEtwRecord>> {
        if pid == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "attach PID must be non-zero",
            ));
        }
        let (sender, receiver) = std::sync::mpsc::sync_channel(8_192);
        self.start_all_for_pid(Some(pid))?;
        if let Err(error) = self.start_realtime_consumers(sender) {
            let _ = self.stop_all();
            return Err(error);
        }
        if let Err(error) = self.request_rundown() {
            let _ = self.stop_all();
            return Err(error);
        }
        Ok(receiver)
    }

    /// Attach one ProcessTrace consumer to every realtime logger. Session
    /// Creation remains separate so a caller can diagnose provider startup
    /// without consuming. Production callers invoke this before requesting
    /// rundown, so capture-state events cannot be emitted into an unconsumed
    /// realtime session.
    #[cfg(windows)]
    pub fn start_realtime_consumers(
        &mut self,
        sender: std::sync::mpsc::SyncSender<RawEtwRecord>,
    ) -> std::io::Result<()> {
        if self.active_sessions.len() != PROVIDERS.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "ETW sessions are not active",
            ));
        }
        for session in sessions() {
            match RealtimeConsumer::spawn(
                &session.session_name,
                session.logical_name,
                sender.clone(),
            ) {
                Ok(consumer) => self.realtime_consumers.push(consumer),
                Err(error) => {
                    self.stop_realtime_consumers();
                    return Err(std::io::Error::other(error));
                }
            }
        }
        Ok(())
    }

    /// Start the four sessions and return the bounded decoded-record stream.
    /// The caller owns the receiver and must drain it continuously; backpressure
    /// is explicit so ETW loss is reported instead of hidden.
    #[cfg(windows)]
    pub fn start_live(&mut self) -> std::io::Result<std::sync::mpsc::Receiver<RawEtwRecord>> {
        let (sender, receiver) = std::sync::mpsc::sync_channel(8_192);
        self.start_all()?;
        if let Err(error) = self.start_realtime_consumers(sender) {
            let _ = self.stop_all();
            return Err(error);
        }
        if let Err(error) = self.request_rundown() {
            let _ = self.stop_all();
            return Err(error);
        }
        Ok(receiver)
    }

    #[cfg(windows)]
    fn stop_realtime_consumers(&mut self) {
        for consumer in self.realtime_consumers.drain(..) {
            let _ = consumer.stop();
        }
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
        self.stop_realtime_consumers();
        #[cfg(windows)]
        for session in self.control_sessions.drain(..).rev() {
            if let Err(error) = native_stop_session(&session) {
                first_error.get_or_insert(error);
            }
        }
        #[cfg(windows)]
        self.active_sessions.clear();
        #[cfg(not(windows))]
        self.active_sessions.clear();
        self.attached_pid = None;
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
    pub reorder_late_events: u64,
    pub reorder_max_lateness_qpc: u64,
    pub reorder_non_monotonic_releases: u64,
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

    pub fn record_reorder_health(&mut self, health: ReorderHealth) {
        self.reorder_late_events = self.reorder_late_events.max(health.late_events);
        self.reorder_max_lateness_qpc = self.reorder_max_lateness_qpc.max(health.max_lateness_qpc);
        self.reorder_non_monotonic_releases = self
            .reorder_non_monotonic_releases
            .max(health.non_monotonic_releases);
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

/// Health facts emitted by the bounded reordering stage. A late event is
/// retained with its original QPC timestamp, but arrived after an older
/// watermark had already been released. Consumers must avoid presenting a
/// cross-thread before/after claim as absolute when this counter is non-zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReorderHealth {
    pub released_events: u64,
    pub late_events: u64,
    pub max_lateness_qpc: u64,
    pub non_monotonic_releases: u64,
}

/// Buffers ETW events until the configured QPC disorder window has elapsed.
/// The buffer is intentionally independent from ETW and can be fed by a TDH
/// decoder or a fixture in exactly the same way.
#[derive(Debug)]
pub struct ReorderBuffer {
    window_qpc: u64,
    max_seen_qpc: Option<u64>,
    pending: Vec<SysEvent>,
    last_emitted_qpc: Option<u64>,
    health: ReorderHealth,
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
            last_emitted_qpc: None,
            health: ReorderHealth::default(),
        }
    }

    pub fn health(&self) -> ReorderHealth {
        self.health
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
        let pending = std::mem::take(&mut self.pending);
        self.record_releases(&pending);
        pending
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
        let released: Vec<_> = self.pending.drain(..split).collect();
        self.record_releases(&released);
        released
    }

    fn record_releases(&mut self, events: &[SysEvent]) {
        for event in events {
            self.health.released_events = self.health.released_events.saturating_add(1);
            if let Some(last) = self.last_emitted_qpc {
                if event.ts_qpc < last {
                    self.health.non_monotonic_releases =
                        self.health.non_monotonic_releases.saturating_add(1);
                    self.health.late_events = self.health.late_events.saturating_add(1);
                    self.health.max_lateness_qpc = self
                        .health
                        .max_lateness_qpc
                        .max(last.saturating_sub(event.ts_qpc));
                }
            }
            self.last_emitted_qpc = Some(
                self.last_emitted_qpc
                    .map_or(event.ts_qpc, |last| last.max(event.ts_qpc)),
            );
        }
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
    use proptest::prelude::*;
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
    fn health_keeps_reorder_uncertainty_visible() {
        let mut health = CaptureHealth::default();
        health.record_reorder_health(ReorderHealth {
            released_events: 10,
            late_events: 2,
            max_lateness_qpc: 90,
            non_monotonic_releases: 1,
        });
        health.record_reorder_health(ReorderHealth {
            released_events: 12,
            late_events: 1,
            max_lateness_qpc: 120,
            non_monotonic_releases: 1,
        });
        assert_eq!(health.reorder_late_events, 2);
        assert_eq!(health.reorder_max_lateness_qpc, 120);
        assert_eq!(health.reorder_non_monotonic_releases, 1);
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

    #[test]
    fn reorder_buffer_converges_for_bounded_cross_provider_permutations() {
        for seed in 0_u64..256 {
            let mut order: Vec<usize> = (0..32).collect();
            let mut state = seed.wrapping_add(1);
            for index in (1..order.len()).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let swap = (state as usize) % (index + 1);
                order.swap(index, swap);
            }
            let mut buffer = ReorderBuffer::with_qpc_frequency(1_000);
            let mut released = Vec::new();
            for index in order {
                released.extend(buffer.push(fixture_event(index as u64, index as u64)));
            }
            released.extend(buffer.flush_all());
            let timestamps: Vec<_> = released.iter().map(|event| event.ts_qpc).collect();
            let mut sorted = timestamps.clone();
            sorted.sort_unstable();
            assert_eq!(timestamps, sorted, "seed {seed} reordered timestamps");
            let mut sequences: Vec<_> = released.iter().map(|event| event.seq).collect();
            sequences.sort_unstable();
            assert_eq!(sequences, (0_u64..32).collect::<Vec<_>>());
        }
    }

    #[test]
    fn reorder_buffer_exposes_window_overflow_without_rewriting_qpc() {
        let mut buffer = ReorderBuffer::with_qpc_frequency(1_000);
        let _ = buffer.push(fixture_event(0, 0));
        let _ = buffer.push(fixture_event(100, 1));
        let _ = buffer.push(fixture_event(200, 3));
        let released = buffer.push(fixture_event(10, 2));
        assert_eq!(
            released
                .iter()
                .map(|event| event.ts_qpc)
                .collect::<Vec<_>>(),
            [10]
        );
        assert_eq!(buffer.health().late_events, 1);
        assert_eq!(buffer.health().max_lateness_qpc, 90);
        assert_eq!(buffer.flush_all()[0].ts_qpc, 200);
    }

    proptest! {
        #[test]
        fn reorder_buffer_property_converges_for_generated_permutations(
            keys in prop::collection::vec(any::<u32>(), 32)
        ) {
            let mut keyed: Vec<_> = keys
                .into_iter()
                .enumerate()
                .map(|(index, key)| (key, index))
                .collect();
            keyed.sort_by_key(|(key, _)| *key);
            let mut buffer = ReorderBuffer::with_qpc_frequency(1_000);
            let mut released = Vec::new();
            for (_, index) in keyed {
                released.extend(buffer.push(fixture_event(index as u64, index as u64)));
            }
            released.extend(buffer.flush_all());
            let timestamps: Vec<_> = released.iter().map(|event| event.ts_qpc).collect();
            let mut sorted = timestamps.clone();
            sorted.sort_unstable();
            prop_assert_eq!(timestamps, sorted);
            let mut sequences: Vec<_> = released.iter().map(|event| event.seq).collect();
            sequences.sort_unstable();
            prop_assert_eq!(sequences, (0_u64..32).collect::<Vec<_>>());
        }

        #[test]
        fn reorder_buffer_always_converges_to_the_same_event_set(
            order in prop::collection::vec(0_u64..128, 1..128)
        ) {
            let mut buffer = ReorderBuffer::with_qpc_frequency(1_000);
            let mut released = Vec::new();
            for (position, seq) in order.iter().copied().enumerate() {
                released.extend(buffer.push(fixture_event(position as u64 * 100, seq)));
            }
            released.extend(buffer.flush_all());
            let mut actual: Vec<_> = released.iter().map(|event| event.seq).collect();
            let mut expected = order;
            actual.sort_unstable();
            expected.sort_unstable();
            prop_assert_eq!(actual, expected);
        }
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
