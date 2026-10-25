pub mod monitor;

use clens_types::{
    normalize_path_pattern, stable_assertion_id, Assertion, AssertionOp, Expectation, HarnessEvent,
    HarnessSpan, IoMode, Mode, Phase, TargetPattern, ToolCall,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process::{Child, ChildStdout, Command, Stdio};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeId {
    HermesAgent,
    ClaudeCode,
    OpenAiCodex,
    GoogleAntigravityCli,
    Openclaw,
    Manus,
    AutoGpt,
    CodeBuddy,
    WorkBuddyCn,
    WorkBuddyInternational,
    TraeAgent,
    Aider,
    ZCode,
    QoderCn,
    CodeArtsAgentCli,
    MiniMaxCodeCli,
    GithubCopilotCli,
    CursorCli,
    DeepSeek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDescriptor {
    pub id: RuntimeId,
    pub display_name: &'static str,
    pub aliases: &'static [&'static str],
    pub protocol: &'static str,
    pub recent_window_months: u8,
}

const RUNTIME_DESCRIPTORS: &[RuntimeDescriptor] = &[
    RuntimeDescriptor {
        id: RuntimeId::HermesAgent,
        display_name: "Hermes Agent",
        aliases: &["hermes", "hermes-agent"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::ClaudeCode,
        display_name: "Claude Code",
        aliases: &["claude", "claude-code"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::OpenAiCodex,
        display_name: "OpenAI Codex",
        aliases: &["codex", "openai-codex"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::GoogleAntigravityCli,
        display_name: "Google Antigravity CLI",
        aliases: &["antigravity", "google-antigravity"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::Openclaw,
        display_name: "OpenClaw",
        aliases: &["openclaw"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::Manus,
        display_name: "Manus",
        aliases: &["manus"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::AutoGpt,
        display_name: "AutoGPT",
        aliases: &["autogpt", "auto-gpt"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::CodeBuddy,
        display_name: "CodeBuddy",
        aliases: &["codebuddy"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::WorkBuddyCn,
        display_name: "WorkBuddy CN",
        aliases: &["workbuddy-cn", "workbuddy-cn-cli"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::WorkBuddyInternational,
        display_name: "WorkBuddy International",
        aliases: &["workbuddy", "workbuddy-intl"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::TraeAgent,
        display_name: "Trae Agent / TRAE CLI",
        aliases: &["trae", "trae-agent", "trae-cli"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::Aider,
        display_name: "Aider",
        aliases: &["aider"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::ZCode,
        display_name: "ZCode",
        aliases: &["zcode"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::QoderCn,
        display_name: "Qoder CN / CLI",
        aliases: &["qoder", "qoder-cn", "qoder-cli"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::CodeArtsAgentCli,
        display_name: "CodeArts Agent CLI",
        aliases: &["codearts", "codearts-agent"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::MiniMaxCodeCli,
        display_name: "MiniMax Code CLI",
        aliases: &["minimax-code", "minimax-code-cli"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::GithubCopilotCli,
        display_name: "GitHub Copilot CLI",
        aliases: &["copilot", "github-copilot"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::CursorCli,
        display_name: "Cursor CLI",
        aliases: &["cursor", "cursor-cli"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
    RuntimeDescriptor {
        id: RuntimeId::DeepSeek,
        display_name: "DeepSeek",
        aliases: &["deepseek"],
        protocol: "jsonl-tier3",
        recent_window_months: 3,
    },
];

pub fn runtime_catalog() -> &'static [RuntimeDescriptor] {
    RUNTIME_DESCRIPTORS
}

pub fn runtime_by_alias(alias: &str) -> Option<RuntimeDescriptor> {
    RUNTIME_DESCRIPTORS.iter().copied().find(|item| {
        item.aliases
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(alias))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachSpec {
    pub source: String,
    pub offset: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailHandle {
    pub source: String,
    pub offset: usize,
}
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HarnessError {
    #[error("attach source is empty")]
    EmptySource,
    #[error("invalid JSONL event: {0}")]
    InvalidJson(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("wrong field type: {0}")]
    WrongType(&'static str),
    #[error("tail I/O failed: {0}")]
    Io(String),
    #[error("process tail has no stdout")]
    MissingStdout,
    #[error("semantic extractor is unavailable: {0}")]
    SemanticExtractorUnavailable(String),
}

/// A malformed JSONL record is retained for diagnostics instead of being
/// silently dropped. The raw bytes are bounded by the caller's batch size and
/// can therefore be persisted or displayed by the session health view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineEntry {
    pub line_no: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub raw: Vec<u8>,
    pub error: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineQueue {
    capacity: usize,
    entries: VecDeque<QuarantineEntry>,
    dropped: u64,
}

impl Default for QuarantineQueue {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}

impl QuarantineQueue {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: VecDeque::new(),
            dropped: 0,
        }
    }

    pub fn push(&mut self, entry: QuarantineEntry) {
        if self.entries.len() >= self.capacity {
            let _ = self.entries.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.entries.push_back(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn iter(&self) -> impl Iterator<Item = &QuarantineEntry> {
        self.entries.iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TailHealth {
    pub bytes_read: u64,
    pub lines_emitted: u64,
    pub last_offset: u64,
    pub partial_bytes: u64,
    pub truncate_count: u64,
    pub quarantine_count: u64,
    pub schema_change_count: u64,
    pub directory_watch_active: bool,
}

/// Windows directory notification used by append-only JSONL tails. The file
/// reader still owns the offset; this handle only wakes it when the directory
/// changes, so a partial line can never be mistaken for a complete record.
#[derive(Debug)]
pub struct DirectoryChangeWatcher {
    #[cfg(windows)]
    handle: isize,
}

impl DirectoryChangeWatcher {
    pub fn new(file_path: &str) -> Result<Self, HarnessError> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use std::path::Path;
            let parent = Path::new(file_path)
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let wide = parent
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            let handle = unsafe {
                win32::create_file_w(
                    wide.as_ptr(),
                    win32::GENERIC_READ,
                    win32::FILE_SHARE_READ | win32::FILE_SHARE_WRITE | win32::FILE_SHARE_DELETE,
                    win32::OPEN_EXISTING,
                    win32::FILE_FLAG_BACKUP_SEMANTICS,
                )
            };
            if handle == win32::INVALID_HANDLE_VALUE {
                return Err(HarnessError::Io(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
            Ok(Self { handle })
        }
        #[cfg(not(windows))]
        {
            let _ = file_path;
            Err(HarnessError::Io(
                "ReadDirectoryChangesW is only available on Windows".to_owned(),
            ))
        }
    }

    pub fn wait(&mut self) -> Result<(), HarnessError> {
        #[cfg(windows)]
        {
            let mut buffer = [0_u8; 16 * 1024];
            let mut returned = 0_u32;
            let ok = unsafe {
                win32::read_directory_changes_w(
                    self.handle,
                    buffer.as_mut_ptr().cast(),
                    buffer.len() as u32,
                    0,
                    win32::FILE_NOTIFY_CHANGE_FILE_NAME
                        | win32::FILE_NOTIFY_CHANGE_SIZE
                        | win32::FILE_NOTIFY_CHANGE_LAST_WRITE,
                    &mut returned,
                )
            };
            if ok == 0 {
                return Err(HarnessError::Io(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err(HarnessError::Io(
                "ReadDirectoryChangesW is only available on Windows".to_owned(),
            ))
        }
    }
}

#[cfg(windows)]
impl Drop for DirectoryChangeWatcher {
    fn drop(&mut self) {
        unsafe {
            let _ = win32::close_handle(self.handle);
        }
    }
}

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;

    pub const GENERIC_READ: u32 = 0x8000_0000;
    pub const FILE_SHARE_READ: u32 = 0x0000_0001;
    pub const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    pub const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    pub const OPEN_EXISTING: u32 = 3;
    pub const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    pub const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    pub const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x0000_0008;
    pub const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    pub const INVALID_HANDLE_VALUE: isize = -1;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *const c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: *const c_void,
        ) -> isize;
        fn ReadDirectoryChangesW(
            directory: isize,
            buffer: *mut c_void,
            buffer_len: u32,
            watch_subtree: i32,
            notify_filter: u32,
            bytes_returned: *mut u32,
            overlapped: *mut c_void,
            completion_routine: *mut c_void,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    pub unsafe fn create_file_w(
        file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        creation_disposition: u32,
        flags_and_attributes: u32,
    ) -> isize {
        // The pointers are assembled from owned, NUL-terminated UTF-16 data
        // above and remain valid for the duration of this call.
        CreateFileW(
            file_name,
            desired_access,
            share_mode,
            std::ptr::null(),
            creation_disposition,
            flags_and_attributes,
            std::ptr::null(),
        )
    }

    pub unsafe fn read_directory_changes_w(
        directory: isize,
        buffer: *mut c_void,
        buffer_len: u32,
        watch_subtree: i32,
        notify_filter: u32,
        bytes_returned: *mut u32,
    ) -> i32 {
        ReadDirectoryChangesW(
            directory,
            buffer,
            buffer_len,
            watch_subtree,
            notify_filter,
            bytes_returned,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }

    pub unsafe fn close_handle(handle: isize) -> i32 {
        CloseHandle(handle)
    }
}

pub trait HarnessAdapter: Send {
    fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError>;
    fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError>;
    fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError>;
    fn name(&self) -> &'static str;
    fn health(&self) -> TailHealth {
        TailHealth::default()
    }
    fn mode(&self) -> Mode {
        Mode::Observe
    }
}

#[derive(Debug, Default)]
pub struct JsonlAdapter {
    source: Option<String>,
    offset: usize,
    read_position: u64,
    file: Option<File>,
    pending: Vec<u8>,
    health: TailHealth,
    watcher: Option<DirectoryChangeWatcher>,
    named_pipe: bool,
}

fn is_named_pipe(source: &str) -> bool {
    source
        .as_bytes()
        .get(..9)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(br"\\.\pipe\"))
}

impl HarnessAdapter for JsonlAdapter {
    fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError> {
        if spec.source.is_empty() {
            return Err(HarnessError::EmptySource);
        }
        let mut file =
            File::open(&spec.source).map_err(|error| HarnessError::Io(error.to_string()))?;
        let named_pipe = is_named_pipe(&spec.source);
        if !named_pipe {
            file.seek(SeekFrom::Start(spec.offset as u64))
                .map_err(|error| HarnessError::Io(error.to_string()))?;
        } else if spec.offset != 0 {
            return Err(HarnessError::Io(
                "named pipe sources cannot resume from a non-zero offset".to_owned(),
            ));
        }
        self.source = Some(spec.source.clone());
        self.offset = spec.offset;
        self.read_position = spec.offset as u64;
        self.file = Some(file);
        self.pending.clear();
        self.watcher = if named_pipe {
            None
        } else {
            DirectoryChangeWatcher::new(&spec.source).ok()
        };
        self.named_pipe = named_pipe;
        self.health = TailHealth {
            last_offset: spec.offset as u64,
            directory_watch_active: self.watcher.is_some(),
            ..TailHealth::default()
        };
        Ok(TailHandle {
            source: spec.source,
            offset: spec.offset,
        })
    }
    fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError> {
        if self.file.is_none() {
            return Err(HarnessError::EmptySource);
        }
        let current_len = if self.named_pipe {
            u64::MAX
        } else {
            self.file
                .as_ref()
                .ok_or(HarnessError::EmptySource)?
                .metadata()
                .map_err(|error| HarnessError::Io(error.to_string()))?
                .len()
        };
        if !self.named_pipe && current_len < self.read_position {
            self.file
                .as_mut()
                .ok_or(HarnessError::EmptySource)?
                .seek(SeekFrom::Start(0))
                .map_err(|error| HarnessError::Io(error.to_string()))?;
            self.read_position = 0;
            self.offset = 0;
            self.pending.clear();
            self.health.truncate_count = self.health.truncate_count.saturating_add(1);
        }
        let mut chunk = [0_u8; 16 * 1024];
        let mut read = self
            .file
            .as_mut()
            .ok_or(HarnessError::EmptySource)?
            .read(&mut chunk)
            .map_err(|error| HarnessError::Io(error.to_string()))?;
        if read == 0 {
            if let Some(watcher) = self.watcher.as_mut() {
                watcher.wait()?;
                read = self
                    .file
                    .as_mut()
                    .ok_or(HarnessError::EmptySource)?
                    .read(&mut chunk)
                    .map_err(|error| HarnessError::Io(error.to_string()))?;
            }
        }
        self.read_position = self.read_position.saturating_add(read as u64);
        self.health.bytes_read = self.health.bytes_read.saturating_add(read as u64);
        self.pending.extend_from_slice(&chunk[..read]);
        buf.clear();
        let mut consumed = 0usize;
        while let Some(relative) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = consumed + relative + 1;
            buf.extend_from_slice(&self.pending[consumed..end]);
            consumed = end;
        }
        if consumed > 0 {
            self.pending.drain(..consumed);
            self.offset = self.offset.saturating_add(consumed);
            self.health.lines_emitted = self
                .health
                .lines_emitted
                .saturating_add(buf.iter().filter(|byte| **byte == b'\n').count() as u64);
        }
        self.health.partial_bytes = self.pending.len() as u64;
        self.health.last_offset = self.offset as u64;
        Ok(buf.len())
    }
    fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError> {
        parse_jsonl_event(line)
    }
    fn name(&self) -> &'static str {
        "jsonl-tier3"
    }
    fn health(&self) -> TailHealth {
        self.health.clone()
    }
}

#[derive(Debug, Default)]
pub struct DeepSeekAdapter(JsonlAdapter);
impl HarnessAdapter for DeepSeekAdapter {
    fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError> {
        self.0.attach(spec)
    }
    fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError> {
        self.0.next_batch(buf)
    }
    fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError> {
        self.0.parse(line)
    }
    fn name(&self) -> &'static str {
        "deepseek"
    }
    fn health(&self) -> TailHealth {
        self.0.health()
    }
}

#[derive(Debug)]
pub struct RuntimeJsonlAdapter {
    runtime: RuntimeId,
    inner: JsonlAdapter,
}

impl RuntimeJsonlAdapter {
    pub fn new(runtime: RuntimeId) -> Self {
        Self {
            runtime,
            inner: JsonlAdapter::default(),
        }
    }
    pub fn runtime(&self) -> RuntimeId {
        self.runtime
    }
}

impl HarnessAdapter for RuntimeJsonlAdapter {
    fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError> {
        self.inner.attach(spec)
    }
    fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError> {
        self.inner.next_batch(buf)
    }
    fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError> {
        self.inner.parse(line)
    }
    fn name(&self) -> &'static str {
        RUNTIME_DESCRIPTORS
            .iter()
            .find(|item| item.id == self.runtime)
            .map_or("jsonl-tier3", |item| item.display_name)
    }
    fn health(&self) -> TailHealth {
        self.inner.health()
    }
}

pub fn adapter_for(runtime: RuntimeId) -> RuntimeJsonlAdapter {
    RuntimeJsonlAdapter::new(runtime)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterHealth {
    pub runtime: RuntimeId,
    pub adapter_name: &'static str,
    pub status: &'static str,
    pub compatibility: &'static str,
}

pub fn adapters_doctor() -> Vec<AdapterHealth> {
    runtime_catalog()
        .iter()
        .map(|descriptor| AdapterHealth {
            runtime: descriptor.id,
            adapter_name: descriptor.display_name,
            status: "fallback-ready",
            compatibility: "tier3-jsonl-unverified",
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionProbe {
    pub runtime: RuntimeId,
    pub reported_version: Option<String>,
    pub sampled_at_utc: Option<String>,
}

pub fn compatibility_note(probe: &VersionProbe) -> &'static str {
    if probe.reported_version.is_some() && probe.sampled_at_utc.is_some() {
        "version-sampled"
    } else {
        "version-unknown-fallback"
    }
}

#[derive(Debug)]
pub struct ProcessTail {
    child: Child,
    stdout: ChildStdout,
    pending: Vec<u8>,
}

impl ProcessTail {
    pub fn spawn(program: &str, args: &[String]) -> Result<Self, HarnessError> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .map_err(|error| HarnessError::Io(error.to_string()))?;
        let stdout = child.stdout.take().ok_or(HarnessError::MissingStdout)?;
        Ok(Self {
            child,
            stdout,
            pending: Vec::new(),
        })
    }

    pub fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError> {
        let mut chunk = [0_u8; 16 * 1024];
        let read = self
            .stdout
            .read(&mut chunk)
            .map_err(|error| HarnessError::Io(error.to_string()))?;
        self.pending.extend_from_slice(&chunk[..read]);
        buf.clear();
        let mut consumed = 0usize;
        while let Some(relative) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = consumed + relative + 1;
            buf.extend_from_slice(&self.pending[consumed..end]);
            consumed = end;
        }
        if consumed > 0 {
            self.pending.drain(..consumed);
        }
        Ok(buf.len())
    }

    pub fn is_running(&mut self) -> Result<bool, HarnessError> {
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|error| HarnessError::Io(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelReport {
    pub runtime: RuntimeId,
    pub fingerprint: u64,
    pub changed: bool,
    pub schema_v: Option<u16>,
    pub status: &'static str,
}

#[derive(Debug, Default)]
pub struct SchemaSentinel {
    fingerprints: HashMap<RuntimeId, u64>,
}

impl SchemaSentinel {
    pub fn observe(
        &mut self,
        runtime: RuntimeId,
        line: &[u8],
    ) -> Result<SentinelReport, HarnessError> {
        let value: Value = serde_json::from_slice(line)
            .map_err(|error| HarnessError::InvalidJson(error.to_string()))?;
        let fingerprint = schema_fingerprint(&value);
        let changed = self
            .fingerprints
            .insert(runtime, fingerprint)
            .is_some_and(|old| old != fingerprint);
        Ok(SentinelReport {
            runtime,
            fingerprint,
            changed,
            schema_v: value
                .get("schema_v")
                .and_then(Value::as_u64)
                .and_then(|raw| u16::try_from(raw).ok()),
            status: if changed {
                "schema-changed"
            } else {
                "stable-or-first-sample"
            },
        })
    }

    pub fn observe_with_health(
        &mut self,
        runtime: RuntimeId,
        line: &[u8],
        health: &mut TailHealth,
    ) -> Result<SentinelReport, HarnessError> {
        let report = self.observe(runtime, line)?;
        if report.changed {
            health.schema_change_count = health.schema_change_count.saturating_add(1);
        }
        Ok(report)
    }
}

fn schema_fingerprint(value: &Value) -> u64 {
    fn visit(value: &Value, mut hash: u64) -> u64 {
        match value {
            Value::Object(object) => {
                let mut keys = object.keys().collect::<Vec<_>>();
                keys.sort_unstable();
                for key in keys {
                    for byte in key.bytes() {
                        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                    }
                    hash = (hash ^ 0x01).wrapping_mul(0x100000001b3);
                    if let Some(child) = object.get(key) {
                        hash = visit(child, hash);
                    }
                }
            }
            Value::Array(values) => {
                hash = (hash ^ 0x02).wrapping_mul(0x100000001b3);
                for child in values.iter().take(1) {
                    hash = visit(child, hash);
                }
            }
            Value::String(_) => hash = (hash ^ 0x03).wrapping_mul(0x100000001b3),
            Value::Number(_) => hash = (hash ^ 0x04).wrapping_mul(0x100000001b3),
            Value::Bool(_) => hash = (hash ^ 0x05).wrapping_mul(0x100000001b3),
            Value::Null => hash = (hash ^ 0x06).wrapping_mul(0x100000001b3),
        }
        hash
    }

    visit(value, 0xcbf29ce484222325)
}

pub fn parse_jsonl_event(line: &[u8]) -> Result<HarnessEvent, HarnessError> {
    parse_jsonl_event_at(line, 0, 0, 0)
}

/// Parse a record and attach the physical origin supplied by the tailer. The
/// producer may include origin fields too, but the tailer wins because it is
/// the component that knows the actual byte offsets in the append-only file.
pub fn parse_jsonl_event_at(
    line: &[u8],
    harness_log_id: u64,
    line_no: u64,
    byte_start: u64,
) -> Result<HarnessEvent, HarnessError> {
    let value: Value = serde_json::from_slice(line)
        .map_err(|error| HarnessError::InvalidJson(error.to_string()))?;
    let number = |name: &'static str| {
        value
            .get(name)
            .and_then(Value::as_i64)
            .ok_or(HarnessError::MissingField(name))
    };
    let text = value
        .get("text")
        .and_then(Value::as_str)
        .ok_or(HarnessError::MissingField("text"))?;
    let phase = match value
        .get("phase")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
    {
        "reasoning" => Phase::Reasoning,
        "tool_call" => Phase::ToolCall,
        "tool_result" => Phase::ToolResult,
        "system" => Phase::System,
        _ => Phase::Unknown,
    };
    let tool = value
        .get("tool")
        .and_then(Value::as_object)
        .and_then(|object| {
            Some(ToolCall {
                name: object.get("name")?.as_str()?.into(),
                arguments: object.get("arguments").cloned().unwrap_or(Value::Null),
            })
        });
    let supplied_line_no = value.get("line_no").and_then(Value::as_u64).unwrap_or(0);
    let supplied_byte_start = value.get("byte_start").and_then(Value::as_u64).unwrap_or(0);
    let supplied_log_id = value
        .get("harness_log_id")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let effective_line_no = if line_no == 0 {
        supplied_line_no
    } else {
        line_no
    };
    let effective_byte_start = if byte_start == 0 {
        supplied_byte_start
    } else {
        byte_start
    };
    Ok(HarnessEvent {
        schema_v: u16::try_from(number("schema_v")?)
            .map_err(|_| HarnessError::WrongType("schema_v"))?,
        ts_utc_ms: number("ts_utc_ms")?,
        step: u64::try_from(number("step")?).map_err(|_| HarnessError::WrongType("step"))?,
        phase,
        tool,
        text: text.into(),
        harness_log_id: if harness_log_id == 0 {
            supplied_log_id
        } else {
            harness_log_id
        },
        line_no: effective_line_no,
        byte_start: effective_byte_start,
        byte_end: effective_byte_start.saturating_add(line.len() as u64),
        raw_json: Some(
            String::from_utf8_lossy(line)
                .trim_end_matches(['\r', '\n'])
                .to_owned()
                .into(),
        ),
    })
}

/// Parse every complete line emitted by a tailer. Bad records are quarantined
/// with their byte origin and do not stop later records from being ingested.
pub fn ingest_batch(
    adapter: &dyn HarnessAdapter,
    batch: &[u8],
    harness_log_id: u64,
    first_line_no: u64,
    first_byte_start: u64,
    quarantine: &mut QuarantineQueue,
    health: &mut TailHealth,
) -> Vec<HarnessEvent> {
    let mut events = Vec::new();
    let mut line_no = first_line_no;
    let mut byte_start = first_byte_start;
    for line in batch.split_inclusive(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let current_line_no = line_no;
        let current_byte_start = byte_start;
        match adapter.parse(line) {
            Ok(mut event) => {
                event.harness_log_id = if harness_log_id == 0 {
                    event.harness_log_id
                } else {
                    harness_log_id
                };
                event.line_no = current_line_no;
                event.byte_start = current_byte_start;
                event.byte_end = current_byte_start.saturating_add(line.len() as u64);
                events.push(event);
            }
            Err(error) => {
                quarantine.push(QuarantineEntry {
                    line_no: current_line_no,
                    byte_start: current_byte_start,
                    byte_end: current_byte_start.saturating_add(line.len() as u64),
                    raw: line.to_vec(),
                    error: error.to_string().into(),
                });
                health.quarantine_count = health.quarantine_count.saturating_add(1);
            }
        }
        line_no = line_no.saturating_add(1);
        byte_start = byte_start.saturating_add(line.len() as u64);
    }
    events
}

fn assertion_target(tool: &ToolCall) -> TargetPattern {
    tool.arguments
        .get("path")
        .or_else(|| tool.arguments.get("target"))
        .and_then(Value::as_str)
        .map(|path| TargetPattern::Path(normalize_path_pattern(path)))
        .unwrap_or_else(|| TargetPattern::FreeText(tool.arguments.to_string().into()))
}

fn assertion_id(event: &HarnessEvent, requested_id: u64, slot: u64) -> u64 {
    let line_no = event.line_no.max(event.step);
    if event.harness_log_id != 0 || event.line_no != 0 {
        stable_assertion_id(event.harness_log_id, line_no, slot)
    } else if requested_id != 0 {
        requested_id.saturating_add(slot)
    } else {
        stable_assertion_id(event.step, event.byte_start, slot)
    }
}

fn make_assertion(
    event: &HarnessEvent,
    requested_id: u64,
    slot: u64,
    op: AssertionOp,
    target: TargetPattern,
    io_mode: IoMode,
    order_key: Option<(u64, u64)>,
) -> Assertion {
    Assertion {
        schema_v: 1,
        id: assertion_id(event, requested_id, slot),
        span: HarnessSpan {
            step: event.step,
            byte_start: event.byte_start,
            byte_end: event.byte_end,
        },
        op,
        target,
        io_mode,
        expect: Expectation::MustHappen,
        order_key,
        confidence: 1.0,
        raw_quote: event.text.clone(),
    }
}

fn shell_command(tool: &ToolCall) -> Option<&str> {
    tool.arguments
        .get("cmd")
        .or_else(|| tool.arguments.get("command"))
        .and_then(Value::as_str)
}

fn shell_tokens(command: &str) -> Vec<&str> {
    command
        .split_whitespace()
        .filter(|token| !token.starts_with('-'))
        .collect()
}

fn shell_target(token: &str) -> TargetPattern {
    if let Some((scheme, rest)) = token.split_once("://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        let (host, port) = authority
            .rsplit_once(':')
            .and_then(|(host, raw_port)| raw_port.parse::<u16>().ok().map(|port| (host, port)))
            .map_or((authority, None), |(host, port)| (host, Some(port)));
        let proto = if scheme.eq_ignore_ascii_case("udp") {
            clens_types::NetProto::Udp
        } else {
            clens_types::NetProto::Tcp
        };
        TargetPattern::Endpoint {
            host: host.to_owned().into_boxed_str(),
            port,
            proto,
        }
    } else {
        TargetPattern::Path(normalize_path_pattern(token))
    }
}

/// Data-driven shell command family names. The table is intentionally small
/// and conservative: an unknown command is left for the semantic extractor.
pub fn shell_template_names() -> &'static [&'static str] {
    &["rm", "mv", "cp", "curl", "ssh"]
}

fn extract_shell_assertions(
    event: &HarnessEvent,
    requested_id: u64,
    command: &str,
) -> Vec<Assertion> {
    let tokens = shell_tokens(command);
    let Some(name) = tokens.first().map(|token| token.to_ascii_lowercase()) else {
        return Vec::new();
    };
    let mut assertions = Vec::new();
    match name.as_str() {
        "rm" => {
            if let Some(target) = tokens.get(1) {
                assertions.push(make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Delete,
                    shell_target(target),
                    IoMode::ReadWrite,
                    None,
                ));
            }
        }
        "mv" => {
            if tokens.len() >= 3 {
                assertions.push(make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Rename,
                    shell_target(tokens[2]),
                    IoMode::ReadWrite,
                    None,
                ));
            }
        }
        "cp" => {
            if tokens.len() >= 3 {
                let read = make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Read,
                    shell_target(tokens[1]),
                    IoMode::ReadOnly,
                    None,
                );
                let write_id = assertion_id(event, requested_id, 1);
                let write = make_assertion(
                    event,
                    requested_id,
                    1,
                    AssertionOp::Write,
                    shell_target(tokens[2]),
                    IoMode::ReadWrite,
                    Some((read.id, write_id)),
                );
                assertions.extend([read, write]);
            }
        }
        "curl" | "ssh" => {
            let target = if name == "ssh" {
                tokens.get(1).copied()
            } else {
                tokens.iter().find(|token| token.contains("://")).copied()
            };
            if let Some(target) = target {
                assertions.push(make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Connect,
                    shell_target(target),
                    IoMode::ReadOnly,
                    None,
                ));
            }
        }
        _ => {}
    }
    assertions
}

/// Deterministic rule extraction. `edit` emits a read precondition and a
/// write assertion with an explicit order edge; shell templates emit one or
/// more conservative assertions. Unknown tools return an empty vector.
pub fn extract_rule_assertions(event: &HarnessEvent, requested_id: u64) -> Vec<Assertion> {
    if event.phase != Phase::ToolCall {
        return Vec::new();
    }
    let Some(tool) = event.tool.as_ref() else {
        return Vec::new();
    };
    let name = tool.name.to_ascii_lowercase();
    if name == "shell" || name == "exec" {
        return shell_command(tool)
            .map(|command| extract_shell_assertions(event, requested_id, command))
            .unwrap_or_default();
    }
    let target = assertion_target(tool);
    match name.as_str() {
        "read" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Read,
            target,
            IoMode::ReadOnly,
            None,
        )],
        "write" | "edit" => {
            if name == "edit" {
                let read = make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Read,
                    target.clone(),
                    IoMode::ReadOnly,
                    None,
                );
                let write_id = assertion_id(event, requested_id, 1);
                let write = make_assertion(
                    event,
                    requested_id,
                    1,
                    AssertionOp::Write,
                    target,
                    IoMode::ReadWrite,
                    Some((read.id, write_id)),
                );
                vec![read, write]
            } else {
                vec![make_assertion(
                    event,
                    requested_id,
                    0,
                    AssertionOp::Write,
                    target,
                    IoMode::ReadWrite,
                    None,
                )]
            }
        }
        "create" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Create,
            target,
            IoMode::CreateNew,
            None,
        )],
        "delete" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Delete,
            target,
            IoMode::ReadWrite,
            None,
        )],
        "connect" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Connect,
            target,
            IoMode::ReadOnly,
            None,
        )],
        "bind" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Bind,
            target,
            IoMode::ReadOnly,
            None,
        )],
        "listen" => vec![make_assertion(
            event,
            requested_id,
            0,
            AssertionOp::Listen,
            target,
            IoMode::ReadOnly,
            None,
        )],
        _ => Vec::new(),
    }
}

pub fn extract_rule_assertion(event: &HarnessEvent, requested_id: u64) -> Option<Assertion> {
    extract_rule_assertions(event, requested_id)
        .into_iter()
        .next()
}

pub fn validate_semantic_assertion(
    source_text: &str,
    assertion: &Assertion,
) -> Result<(), HarnessError> {
    if !source_text.contains(&*assertion.raw_quote) {
        return Err(HarnessError::InvalidJson(
            "semantic assertion raw_quote is not contained in source text".to_owned(),
        ));
    }
    if !(0.0..=0.7).contains(&assertion.confidence) {
        return Err(HarnessError::InvalidJson(
            "semantic assertion confidence must be within 0..=0.7".to_owned(),
        ));
    }
    Ok(())
}

/// The feature is an integration boundary, not a fake model. Until a vetted
/// ONNX backend is supplied, callers receive an explicit unavailable result.
pub fn extract_semantic_assertions(_text: &str) -> Result<Vec<Assertion>, HarnessError> {
    Err(HarnessError::SemanticExtractorUnavailable(
        "Qwen ONNX backend is not linked in this build".to_owned(),
    ))
}

const MAX_ASSERTIONS: usize = 100_000;

#[derive(Debug)]
struct PoolEntry {
    assertion: Assertion,
    duplicates: u64,
}

#[derive(Debug)]
pub struct AssertionPool {
    by_key: HashMap<String, Vec<usize>>,
    entries: Vec<PoolEntry>,
    contradictory: HashSet<u64>,
    max_assertions: usize,
    semantic_enabled: bool,
}

impl Default for AssertionPool {
    fn default() -> Self {
        Self::with_capacity(MAX_ASSERTIONS)
    }
}

impl AssertionPool {
    pub fn with_capacity(max_assertions: usize) -> Self {
        Self {
            by_key: HashMap::new(),
            entries: Vec::new(),
            contradictory: HashSet::new(),
            max_assertions: max_assertions.max(1),
            semantic_enabled: true,
        }
    }

    fn identity(assertion: &Assertion) -> String {
        format!(
            "{:?}|{:?}|{:?}|{}:{}:{}",
            assertion.op,
            assertion.target,
            assertion.io_mode,
            assertion.span.step,
            assertion.span.byte_start,
            assertion.span.byte_end
        )
    }

    pub fn insert(&mut self, mut assertion: Assertion) -> u64 {
        if assertion.id == 0 {
            assertion.id = stable_assertion_id(assertion.span.step, assertion.span.byte_start, 0);
        }
        let key = Self::identity(&assertion);
        if let Some(indexes) = self.by_key.get_mut(&key) {
            if let Some(index) = indexes.iter().copied().find(|index| {
                self.entries
                    .get(*index)
                    .is_some_and(|entry| entry.assertion.expect == assertion.expect)
            }) {
                if let Some(entry) = self.entries.get_mut(index) {
                    entry.duplicates = entry.duplicates.saturating_add(1);
                    return entry.assertion.id;
                }
            }
            for index in indexes.iter().copied() {
                if let Some(entry) = self.entries.get(index) {
                    self.contradictory.insert(entry.assertion.id);
                }
            }
        }
        if self
            .entries
            .iter()
            .any(|entry| entry.assertion.id == assertion.id)
        {
            assertion.id = stable_assertion_id(
                assertion.id,
                assertion.span.byte_start,
                self.entries.len() as u64 + 1,
            );
        }
        let id = assertion.id;
        if self.entries.len() >= self.max_assertions {
            self.semantic_enabled = false;
            return id;
        }
        let index = self.entries.len();
        self.entries.push(PoolEntry {
            assertion,
            duplicates: 1,
        });
        self.by_key.entry(key).or_default().push(index);
        if let Some(indexes) = self
            .by_key
            .values()
            .find(|indexes| indexes.contains(&index))
        {
            if indexes.len() > 1 {
                self.contradictory.insert(id);
            }
        }
        id
    }

    pub fn insert_semantic(&mut self, assertion: Assertion) -> Option<u64> {
        if !self.semantic_enabled || self.entries.len() >= self.max_assertions {
            self.semantic_enabled = false;
            return None;
        }
        Some(self.insert(assertion))
    }

    pub fn values(&self) -> impl Iterator<Item = &Assertion> {
        self.entries.iter().map(|entry| &entry.assertion)
    }

    pub fn duplicate_count(&self, assertion_id: u64) -> u64 {
        self.entries
            .iter()
            .find(|entry| entry.assertion.id == assertion_id)
            .map_or(0, |entry| entry.duplicates)
    }

    pub fn is_contradictory(&self, assertion_id: u64) -> bool {
        self.contradictory.contains(&assertion_id)
    }

    pub fn semantic_enabled(&self) -> bool {
        self.semantic_enabled
    }

    pub fn is_overloaded(&self) -> bool {
        self.entries.len() >= self.max_assertions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_requested_recent_runtime_set() {
        assert_eq!(runtime_catalog().len(), 19);
        assert_eq!(
            runtime_by_alias("claude-code").map(|item| item.id),
            Some(RuntimeId::ClaudeCode)
        );
        assert_eq!(
            runtime_by_alias("workbuddy-cn").map(|item| item.id),
            Some(RuntimeId::WorkBuddyCn)
        );
        assert!(runtime_catalog()
            .iter()
            .all(|item| item.protocol == "jsonl-tier3" && item.recent_window_months == 3));
    }

    #[test]
    fn named_adapter_keeps_common_contract() {
        let adapter = adapter_for(RuntimeId::CursorCli);
        assert_eq!(adapter.name(), "Cursor CLI");
        assert_eq!(adapter.mode(), Mode::Observe);
    }

    #[test]
    fn attach_source_recognizes_windows_named_pipe_without_path_slicing() {
        assert!(is_named_pipe(r"\\.\pipe\agent-events"));
        assert!(!is_named_pipe("日志.jsonl"));
    }

    #[test]
    fn adapters_doctor_keeps_unknown_versions_non_blocking() {
        let health = adapters_doctor();
        assert_eq!(health.len(), 19);
        assert!(health.iter().all(|item| item.status == "fallback-ready"));
        let probe = VersionProbe {
            runtime: RuntimeId::HermesAgent,
            reported_version: None,
            sampled_at_utc: None,
        };
        assert_eq!(compatibility_note(&probe), "version-unknown-fallback");
    }

    #[test]
    fn file_tail_returns_only_complete_lines_and_keeps_partial_line() {
        let path =
            std::env::temp_dir().join(format!("causativelens-tail-{}.jsonl", std::process::id()));
        std::fs::write(&path, b"one\ntwo").expect("write fixture");
        let mut adapter = JsonlAdapter::default();
        adapter
            .attach(AttachSpec {
                source: path.to_string_lossy().into_owned(),
                offset: 0,
            })
            .expect("attach fixture");
        let mut buf = Vec::new();
        assert_eq!(adapter.next_batch(&mut buf).expect("read fixture"), 4);
        assert_eq!(buf, b"one\n");
        std::fs::write(&path, b"one\ntwo\n").expect("append fixture");
        assert_eq!(adapter.next_batch(&mut buf).expect("read append"), 4);
        assert_eq!(buf, b"two\n");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn file_tail_rewinds_after_truncate() {
        let path = std::env::temp_dir().join(format!(
            "causativelens-truncate-{}.jsonl",
            std::process::id()
        ));
        std::fs::write(&path, b"old\n").expect("write fixture");
        let mut adapter = JsonlAdapter::default();
        adapter
            .attach(AttachSpec {
                source: path.to_string_lossy().into_owned(),
                offset: 0,
            })
            .expect("attach fixture");
        let mut buf = Vec::new();
        assert_eq!(adapter.next_batch(&mut buf).expect("read fixture"), 4);
        std::fs::write(&path, b"n\n").expect("truncate fixture");
        assert_eq!(
            adapter.next_batch(&mut buf).expect("read after truncate"),
            2
        );
        assert_eq!(buf, b"n\n");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn schema_sentinel_reports_field_shape_changes() {
        let mut sentinel = SchemaSentinel::default();
        let first = sentinel
            .observe(RuntimeId::ClaudeCode, br#"{"schema_v":1,"text":"x"}"#)
            .expect("first");
        let second = sentinel
            .observe(
                RuntimeId::ClaudeCode,
                br#"{"schema_v":1,"text":"x","tool":{}}"#,
            )
            .expect("second");
        assert!(!first.changed);
        assert!(second.changed);
    }
    #[test]
    fn jsonl_tier3_keeps_unknown_phase_and_raw_text() {
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"new_phase","text":"read config"}"#,
        )
        .expect("event");
        assert_eq!(event.phase, Phase::Unknown);
        assert_eq!(&*event.text, "read config");
        assert!(event
            .raw_json
            .as_deref()
            .is_some_and(|raw| raw.contains("new_phase")));
    }

    #[test]
    fn parser_preserves_unknown_tool_shape_in_raw_record() {
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"tool_call","text":"future","tool":{"kind":"new_tool","payload":{}}}"#,
        )
        .expect("event");
        assert!(event.tool.is_none());
        assert!(event
            .raw_json
            .as_deref()
            .is_some_and(|raw| raw.contains("new_tool")));
    }

    #[test]
    fn ingest_batch_quarantines_bad_lines_and_keeps_good_lines() {
        let adapter = JsonlAdapter::default();
        let mut quarantine = QuarantineQueue::with_capacity(2);
        let mut health = TailHealth::default();
        let batch = b"not-json\n{\"schema_v\":1,\"ts_utc_ms\":2,\"step\":3,\"phase\":\"system\",\"text\":\"ok\"}\n";
        let events = ingest_batch(&adapter, batch, 9, 10, 100, &mut quarantine, &mut health);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].harness_log_id, 9);
        assert_eq!(events[0].line_no, 11);
        assert_eq!(events[0].byte_start, 109);
        assert_eq!(quarantine.len(), 1);
        assert_eq!(health.quarantine_count, 1);
    }
    #[test]
    fn rule_layer_extracts_tool_call_with_full_confidence() {
        let event = parse_jsonl_event(br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"tool_call","text":"reading","tool":{"name":"read","arguments":{"path":"C:\\app\\config.yaml"}}}"#).expect("event");
        let assertion = extract_rule_assertion(&event, 1).expect("assertion");
        assert_eq!(assertion.op, AssertionOp::Read);
        assert_eq!(assertion.confidence, 1.0);
    }

    #[test]
    fn edit_emits_read_precondition_and_ordered_write() {
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"tool_call","text":"edit config","harness_log_id":7,"line_no":3,"tool":{"name":"edit","arguments":{"path":"C:\\app\\config.yaml"}}}"#,
        )
        .expect("event");
        let assertions = extract_rule_assertions(&event, 0);
        assert_eq!(assertions.len(), 2);
        assert_eq!(assertions[0].op, AssertionOp::Read);
        assert_eq!(assertions[1].op, AssertionOp::Write);
        assert_eq!(
            assertions[1].order_key,
            Some((assertions[0].id, assertions[1].id))
        );
    }

    #[test]
    fn shell_templates_cover_file_operations() {
        assert_eq!(shell_template_names(), &["rm", "mv", "cp", "curl", "ssh"]);
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"tool_call","text":"run","tool":{"name":"shell","arguments":{"cmd":"cp input.txt output.txt"}}}"#,
        )
        .expect("event");
        let assertions = extract_rule_assertions(&event, 1);
        assert_eq!(assertions.len(), 2);
        assert_eq!(assertions[0].op, AssertionOp::Read);
        assert_eq!(assertions[1].op, AssertionOp::Write);
        assert!(assertions[1].order_key.is_some());
    }

    #[test]
    fn semantic_contract_rejects_hallucinated_quote() {
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"reasoning","text":"check config"}"#,
        )
        .expect("event");
        let mut assertion = make_assertion(
            &event,
            1,
            0,
            AssertionOp::AssertPrecondition,
            TargetPattern::FreeText("config".into()),
            IoMode::ReadOnly,
            None,
        );
        assertion.confidence = 0.7;
        assertion.raw_quote = "hallucinated".into();
        assert!(validate_semantic_assertion(&event.text, &assertion).is_err());
        assertion.raw_quote = "check config".into();
        assert!(validate_semantic_assertion(&event.text, &assertion).is_ok());
    }
    #[test]
    fn assertion_pool_deduplicates_and_marks_contradictions() {
        let event = HarnessEvent {
            schema_v: 1,
            ts_utc_ms: 0,
            step: 1,
            phase: Phase::ToolCall,
            tool: Some(ToolCall {
                name: "read".into(),
                arguments: serde_json::json!({"path":"x"}),
            }),
            text: "x".into(),
            harness_log_id: 7,
            line_no: 1,
            byte_start: 0,
            byte_end: 1,
            raw_json: None,
        };
        let mut pool = AssertionPool::default();
        let a = extract_rule_assertion(&event, 1).expect("a");
        let id = pool.insert(a.clone());
        assert_eq!(pool.duplicate_count(id), 1);
        assert_eq!(pool.insert(a), id);
        assert_eq!(pool.duplicate_count(id), 2);
        let mut b = extract_rule_assertion(&event, 1).expect("b");
        b.expect = Expectation::MustNotHappen;
        let contradictory_id = pool.insert(b);
        assert_ne!(contradictory_id, id);
        assert_eq!(pool.values().count(), 2);
        assert!(pool.values().all(|stored| pool.is_contradictory(stored.id)));
    }

    #[test]
    fn assertion_pool_disables_semantic_layer_at_capacity() {
        let mut pool = AssertionPool::with_capacity(1);
        let event = parse_jsonl_event(
            br#"{"schema_v":1,"ts_utc_ms":2,"step":3,"phase":"tool_call","text":"x","tool":{"name":"read","arguments":{"path":"x"}}}"#,
        )
        .expect("event");
        let first = extract_rule_assertion(&event, 1).expect("first");
        let second = Assertion {
            confidence: 0.7,
            raw_quote: "x".into(),
            id: 2,
            ..first.clone()
        };
        let _ = pool.insert(first);
        assert!(pool.insert_semantic(second).is_none());
        assert!(pool.is_overloaded());
        assert!(!pool.semantic_enabled());
    }
}
