use super::*;
use clens_harnessd::monitor::{self, LiveMonitor, MonitorConfig};
use clens_types::SysEvent;
use std::collections::VecDeque;
#[cfg(windows)]
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::mpsc::{self, Receiver};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, TerminateProcess};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

#[cfg(windows)]
#[derive(Debug, serde::Deserialize)]
pub(super) struct CaptureLine {
    event: SysEvent,
    #[serde(default)]
    path: Option<String>,
}

#[cfg(windows)]
pub(super) struct CaptureBridge {
    process: HANDLE,
    output: PathBuf,
    lines: Receiver<Result<CaptureLine, String>>,
}

#[cfg(windows)]
impl std::fmt::Debug for CaptureBridge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CaptureBridge")
            .field("process", &self.process)
            .finish_non_exhaustive()
    }
}

#[cfg(windows)]
impl CaptureBridge {
    pub(super) fn start() -> Result<Option<Self>, String> {
        let path = std::env::var_os("CAUSATIVELENS_CAPTURE_PATH")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|parent| parent.join("clens-capture.exe")))
            })
            .or_else(|| Some(PathBuf::from("clens-capture.exe")));
        let Some(path) = path else {
            return Ok(None);
        };
        if !path.is_file() {
            return Ok(None);
        }
        let output = std::env::temp_dir().join(format!(
            "CausativeLens-capture-{}.jsonl",
            uuid::Uuid::new_v4()
        ));
        let path_wide: Vec<u16> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let parameters = format!("--live --output \"{}\"", output.display());
        let parameters_wide: Vec<u16> = parameters
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let verb: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();
        let mut execute = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: verb.as_ptr(),
            lpFile: path_wide.as_ptr(),
            lpParameters: parameters_wide.as_ptr(),
            nShow: SW_HIDE,
            ..Default::default()
        };
        if unsafe { ShellExecuteExW(&mut execute) } == 0 || execute.hProcess.is_null() {
            return Err(format!(
                "capture host elevation was declined or failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let (line_tx, line_rx) = mpsc::channel();
        let process = execute.hProcess;
        let process_for_thread = process as usize;
        let output_for_thread = output.clone();
        std::thread::spawn(move || {
            let mut offset = 0_u64;
            loop {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .read(true)
                    .open(&output_for_thread)
                {
                    let _ = file.seek(SeekFrom::Start(offset));
                    let mut bytes = Vec::new();
                    if file.read_to_end(&mut bytes).is_ok() {
                        offset = offset.saturating_add(bytes.len() as u64);
                        for line in bytes
                            .split(|byte| *byte == b'\n')
                            .filter(|line| !line.is_empty())
                        {
                            let result = serde_json::from_slice::<CaptureLine>(line)
                                .map_err(|error| format!("capture event decode failed: {error}"));
                            if line_tx.send(result).is_err() {
                                return;
                            }
                        }
                    }
                }
                let mut code = 0_u32;
                let exited = unsafe {
                    GetExitCodeProcess(process_for_thread as HANDLE, &mut code) != 0 && code != 259
                };
                if exited {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        });
        Ok(Some(Self {
            process,
            output,
            lines: line_rx,
        }))
    }

    pub(super) fn drain(&self) -> Vec<Result<CaptureLine, String>> {
        let mut result = Vec::new();
        while let Ok(line) = self.lines.try_recv() {
            result.push(line);
        }
        result
    }

    pub(super) fn running(&mut self) -> bool {
        let mut code = 0_u32;
        unsafe { GetExitCodeProcess(self.process, &mut code) != 0 && code == 259 }
    }

    pub(super) fn stop(&mut self) {
        if self.running() {
            let _ = unsafe { TerminateProcess(self.process, 0) };
        }
        unsafe {
            CloseHandle(self.process);
        }
        let _ = std::fs::remove_file(&self.output);
    }
}

#[cfg(windows)]
impl Drop for CaptureBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineRow {
    pub seq: u64,
    pub ts_utc_ms: i64,
    pub category: String,
    pub source: String,
    pub kind: String,
    pub target: String,
    pub text: String,
    pub state: String,
}
#[derive(Debug, Default)]
pub struct TimelineBuffer {
    next: u64,
    rows: VecDeque<TimelineRow>,
}
impl TimelineBuffer {
    pub fn push(&mut self, mut row: TimelineRow) {
        self.next = self.next.saturating_add(1);
        if self.rows.len() >= 5000 {
            self.rows.pop_front();
        }
        row.seq = self.next;
        row.kind = row.kind.chars().take(80).collect();
        row.target = row.target.chars().take(512).collect();
        row.text = row.text.chars().take(1024).collect();
        self.rows.push_back(row);
    }
    fn page(&self, after: u64, limit: usize) -> Value {
        let mut rows = Vec::new();
        let mut size = 0;
        let mut next = after;
        for row in self.rows.iter().filter(|r| r.seq > after) {
            let bytes = serde_json::to_vec(row).map_or(8192, |b| b.len());
            if rows.len() >= limit || size + bytes > 48 * 1024 {
                break;
            }
            size += bytes;
            next = row.seq;
            rows.push(row);
        }
        json!({"rows":rows,"next_cursor":next,"has_more":self.rows.back().is_some_and(|r|r.seq>next),"retained_from":self.rows.front().map_or(0,|r|r.seq)})
    }
}
fn config(params: &Value) -> Result<MonitorConfig, String> {
    let runtime_id = params
        .get("runtime_id")
        .and_then(Value::as_str)
        .ok_or("runtime_id_required")?
        .into();
    let folder = params
        .get("folder")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 32767)
        .ok_or("folder_required")?;
    Ok(MonitorConfig {
        runtime_id,
        folder: PathBuf::from(folder),
        include_profile: params
            .get("include_profile")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        read_existing: params
            .get("read_existing")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    })
}
impl ObservationSession {
    #[cfg(windows)]
    pub(super) fn pump_capture(&mut self) {
        let (records, running) = if let Some(capture) = self.capture.as_mut() {
            (capture.drain(), capture.running())
        } else {
            (Vec::new(), false)
        };
        for record in records {
            match record {
                Ok(record) => {
                    if let Some(path) = record.path {
                        if let clens_types::EventPayload::File { path: path_id, .. } =
                            &record.event.payload
                        {
                            self.record_path(*path_id, path);
                        }
                    }
                    if self.ingest_system_event(&record.event).is_err() {
                        self.capture_health.record_capture_failure();
                    }
                }
                Err(_) => self.capture_health.record_capture_failure(),
            }
        }
        if self.capture.is_some() && !running {
            self.capture_health.record_capture_failure();
            self.capture.take();
        }
    }

    pub(super) fn pump_monitor(&mut self) {
        #[cfg(windows)]
        self.pump_capture();
        let events = self
            .monitor
            .as_ref()
            .map(LiveMonitor::drain)
            .unwrap_or_default();
        if let Some(monitor) = &self.monitor {
            let health = monitor.health();
            self.harness_health.quarantine_count = health.rejected;
            self.harness_health.truncate_count = health.rotations;
        }
        for event in events {
            self.ingest_harness_event(&event);
        }
    }
    pub(super) fn monitor_status(&self, id: Uuid) -> Value {
        let config = self.monitor.as_ref().map(LiveMonitor::config);
        #[cfg(windows)]
        let capture_connected = self.capture.is_some();
        #[cfg(not(windows))]
        let capture_connected = false;
        json!({
            "session_id":id,
            "runtime_id":config.map(|c|&c.runtime_id),
            "folder":config.map(|c|&c.folder),
            "health":self.monitor.as_ref().map(LiveMonitor::health),
            "capture_connected":capture_connected,
            "mode":"observe",
            "timeline_count":self.timeline.rows.len(),
            "assertions":self.assertion_pool.values().count()
        })
    }
}
impl DaemonState {
    pub(super) fn monitoring_rpc(&mut self, method: &str, params: &Value) -> Result<Value, String> {
        match method {
            "monitor.templates" => Ok(json!({"templates":monitor::templates()?})),
            "monitor.probe" => Ok(json!(monitor::probe(&config(params)?)?)),
            "monitor.start" => {
                if self
                    .sessions
                    .values()
                    .filter_map(|s| s.monitor.as_ref())
                    .filter(|m| m.health().running)
                    .count()
                    >= 8
                {
                    return Err("monitor_limit_reached".into());
                }
                if self.sessions.len() >= 32 {
                    return Err("session_limit_reached: close an old session".into());
                }
                let reader = LiveMonitor::start(config(params)?)?;
                #[cfg(windows)]
                let capture = if params
                    .get("capture_etw")
                    .and_then(Value::as_bool)
                    .unwrap_or(true)
                {
                    CaptureBridge::start()?
                } else {
                    None
                };
                #[cfg(windows)]
                let capture_connected = capture.is_some();
                #[cfg(not(windows))]
                let capture_connected = false;
                let id = Uuid::new_v4();
                let session = ObservationSession {
                    monitor: Some(reader),
                    #[cfg(windows)]
                    capture,
                    ..ObservationSession::default()
                };
                self.sessions.insert(id, session);
                Ok(json!({"session_id":id,"mode":"observe","capture_connected":capture_connected}))
            }
            "monitor.list" => Ok(
                json!({"monitors":self.sessions.iter().filter(|(_,s)|s.monitor.is_some()).map(|(id,s)|s.monitor_status(*id)).collect::<Vec<_>>()}),
            ),
            "monitor.status" | "monitor.events" | "monitor.stop" => {
                let id = session_id(params).ok_or("session_id_required")?;
                let session = self.sessions.get_mut(&id).ok_or("unknown_session")?;
                if method == "monitor.stop" {
                    if let Some(reader) = &mut session.monitor {
                        reader.stop();
                    }
                    #[cfg(windows)]
                    if let Some(mut capture) = session.capture.take() {
                        capture.stop();
                    }
                    session.pump_monitor();
                    return Ok(session.monitor_status(id));
                }
                if method == "monitor.events" {
                    let after = params.get("after").and_then(Value::as_u64).unwrap_or(0);
                    let limit = params
                        .get("limit")
                        .and_then(Value::as_u64)
                        .unwrap_or(100)
                        .clamp(1, 100) as usize;
                    return Ok(session.timeline.page(after, limit));
                }
                Ok(session.monitor_status(id))
            }
            _ => Err("unknown_monitor_method".into()),
        }
    }
}
pub(super) fn target_of(event: &HarnessEvent) -> String {
    event
        .tool
        .as_ref()
        .and_then(|t| {
            t.arguments
                .get("path")
                .or_else(|| t.arguments.get("target"))
                .or_else(|| t.arguments.get("cmd"))
        })
        .and_then(Value::as_str)
        .unwrap_or("")
        .into()
}
pub(super) fn category_of(event: &HarnessEvent) -> &'static str {
    match event.tool.as_ref().map(|t| t.name.as_ref()) {
        Some("read" | "write" | "edit" | "create" | "delete") => "files",
        Some("connect" | "bind" | "listen") => "network",
        _ => "other",
    }
}
