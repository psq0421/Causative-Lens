mod normalize;
pub use normalize::normalize_record;

use clens_types::HarnessEvent;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_FILES: usize = 128;
const MAX_ENTRIES: usize = 8192;
const READ_BUDGET: u64 = 256 * 1024;
const MAX_LINE: usize = 256 * 1024;
const QUEUE_CAPACITY: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTemplate {
    pub id: String,
    pub name: String,
    pub profile_dirs: Vec<String>,
    pub formats: Vec<String>,
    pub guidance: HashMap<String, String>,
}
pub fn templates() -> Result<Vec<RuntimeTemplate>, String> {
    serde_json::from_str(include_str!("../../templates/monitoring.json")).map_err(|e| e.to_string())
}
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub runtime_id: String,
    pub folder: PathBuf,
    pub include_profile: bool,
    pub read_existing: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub runtime_id: String,
    pub roots: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub recognized: Vec<String>,
    pub scan_limited: bool,
    pub guidance: HashMap<String, String>,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct MonitorHealth {
    pub running: bool,
    pub files: usize,
    pub lines: u64,
    pub recognized: u64,
    pub rejected: u64,
    pub rotations: u64,
    pub skipped_bytes: u64,
    pub dropped: u64,
    pub io_errors: u64,
    pub scan_limited: bool,
    pub last_error: Option<String>,
    pub last_activity_ms: Option<i64>,
}
#[derive(Debug, Default)]
struct Shared {
    health: MonitorHealth,
    queue: VecDeque<HarnessEvent>,
}

#[derive(Debug)]
pub struct LiveMonitor {
    config: MonitorConfig,
    shared: Arc<Mutex<Shared>>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl LiveMonitor {
    pub fn start(config: MonitorConfig) -> Result<Self, String> {
        let report = probe(&config)?;
        let shared = Arc::new(Mutex::new(Shared {
            health: MonitorHealth {
                running: true,
                files: report.files.len(),
                scan_limited: report.scan_limited,
                ..MonitorHealth::default()
            },
            ..Shared::default()
        }));
        let stopped = Arc::new(AtomicBool::new(false));
        // Establish historical offsets before returning so an immediate append is never skipped.
        let mut cursors = HashMap::new();
        for path in &report.files {
            if let Ok(metadata) = fs::metadata(path) {
                cursors.insert(
                    path.clone(),
                    new_cursor(&metadata, config.read_existing, &shared),
                );
            }
        }
        let state = Arc::clone(&shared);
        let signal = Arc::clone(&stopped);
        let roots = report.roots;
        let worker = thread::Builder::new()
            .name("causativelens-tail".into())
            .spawn(move || {
                run_reader(roots, cursors, &signal, &state);
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            config,
            shared,
            stopped,
            worker: Some(worker),
        })
    }
    pub fn config(&self) -> &MonitorConfig {
        &self.config
    }
    pub fn drain(&self) -> Vec<HarnessEvent> {
        self.shared
            .lock()
            .map(|mut s| s.queue.drain(..).collect())
            .unwrap_or_default()
    }
    pub fn health(&self) -> MonitorHealth {
        self.shared
            .lock()
            .map(|s| s.health.clone())
            .unwrap_or_else(|_| MonitorHealth {
                last_error: Some("reader_state_unavailable".into()),
                ..MonitorHealth::default()
            })
    }
    pub fn stop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
        if let Ok(mut s) = self.shared.lock() {
            s.health.running = false;
        }
    }
}
impl Drop for LiveMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn configuration(config: &MonitorConfig) -> Result<(RuntimeTemplate, Vec<PathBuf>), String> {
    let template = templates()?
        .into_iter()
        .find(|t| t.id == config.runtime_id)
        .ok_or("unknown_runtime")?;
    if !config.folder.is_dir() {
        return Err("folder_not_found".into());
    }
    let selected = config
        .folder
        .canonicalize()
        .map_err(|e| format!("folder_unreadable: {e}"))?;
    if fs::symlink_metadata(&config.folder).is_ok_and(|m| is_link(&m)) {
        return Err("linked_folder_not_allowed".into());
    }
    let mut roots = vec![selected];
    if config.include_profile {
        if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
            for relative in &template.profile_dirs {
                let path = PathBuf::from(&home).join(relative);
                if path.is_dir() && !fs::symlink_metadata(&path).is_ok_and(|m| is_link(&m)) {
                    if let Ok(p) = path.canonicalize() {
                        if !roots.contains(&p) {
                            roots.push(p);
                        }
                    }
                }
            }
        }
    }
    Ok((template, roots))
}
pub fn probe(config: &MonitorConfig) -> Result<ProbeResult, String> {
    let (template, roots) = configuration(config)?;
    let (files, limited) = discover(&roots);
    let mut recognized = HashSet::new();
    for path in files.iter().take(12) {
        if let Ok(file) = File::open(path) {
            let mut bytes = Vec::new();
            if file.take(READ_BUDGET).read_to_end(&mut bytes).is_ok() {
                let text_mode = text_mode(path);
                let records = if snapshot(path) {
                    vec![bytes.as_slice()]
                } else {
                    bytes.split(|b| *b == b'\n').take(32).collect()
                };
                for line in records {
                    if let Ok(events) = normalize_record(line, 1, 0, now_ms(), text_mode) {
                        if !events.is_empty() {
                            recognized.insert(if events.iter().any(|e| e.tool.is_some()) {
                                "structured-tools".to_string()
                            } else {
                                "text-or-status".to_string()
                            });
                        }
                    }
                }
            }
        }
    }
    let mut recognized: Vec<_> = recognized.into_iter().collect();
    recognized.sort();
    Ok(ProbeResult {
        runtime_id: config.runtime_id.clone(),
        roots,
        files,
        recognized,
        scan_limited: limited,
        guidance: template.guidance,
    })
}
fn snapshot(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
}
fn text_mode(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| matches!(s.to_ascii_lowercase().as_str(), "md" | "txt" | "log"))
}
fn candidate(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if [
        "auth",
        "credential",
        "secret",
        "token",
        "settings",
        "config",
    ]
    .iter()
    .any(|p| name.contains(p))
    {
        return false;
    }
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "jsonl" | "ndjson" => true,
        "json" => [
            "session", "trace", "event", "export", "history", "message", "task",
        ]
        .iter()
        .any(|s| name.contains(s)),
        "md" | "log" | "txt" => [
            "history",
            "chat",
            "trace",
            "event",
            "output",
            "session",
            "conversation",
            "log",
        ]
        .iter()
        .any(|s| name.contains(s)),
        _ => false,
    }
}
fn discover(roots: &[PathBuf]) -> (Vec<PathBuf>, bool) {
    let mut stack: Vec<_> = roots.iter().cloned().map(|p| (p, 0)).collect();
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    let mut entries = 0;
    let mut limited = false;
    while let Some((dir, depth)) = stack.pop() {
        if !seen.insert(dir.clone()) {
            continue;
        }
        let Ok(read) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            entries += 1;
            if entries > MAX_ENTRIES {
                limited = true;
                break;
            }
            let path = entry.path();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if fs::symlink_metadata(&path).is_ok_and(|m| is_link(&m)) {
                continue;
            }
            if metadata.is_dir() {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !matches!(
                    name.as_str(),
                    ".git"
                        | "node_modules"
                        | "target"
                        | ".trash"
                        | "venv"
                        | ".venv"
                        | "cache"
                        | "caches"
                ) {
                    if depth < 6 {
                        stack.push((path, depth + 1));
                    } else {
                        limited = true;
                    }
                }
            } else if metadata.is_file() && candidate(&path) {
                files.push(path);
            }
        }
        if entries > MAX_ENTRIES {
            break;
        }
    }
    files.sort_by_key(|p| std::cmp::Reverse(fs::metadata(p).and_then(|m| m.modified()).ok()));
    files.dedup();
    if files.len() > MAX_FILES {
        files.truncate(MAX_FILES);
        limited = true;
    }
    (files, limited)
}
#[cfg(windows)]
fn is_link(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_link(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[derive(Debug)]
struct Cursor {
    offset: u64,
    pending: Vec<u8>,
    start: u64,
    generation: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    skip_partial: bool,
}
fn new_cursor(metadata: &fs::Metadata, read_existing: bool, shared: &Mutex<Shared>) -> Cursor {
    let len = metadata.len();
    let offset = if read_existing {
        len.saturating_sub(READ_BUDGET)
    } else {
        len
    };
    if let Ok(mut s) = shared.lock() {
        s.health.skipped_bytes += offset;
    }
    Cursor {
        offset,
        pending: Vec::new(),
        start: offset,
        generation: 0,
        modified: metadata.modified().ok(),
        created: metadata.created().ok(),
        skip_partial: read_existing && offset > 0,
    }
}
fn run_reader(
    roots: Vec<PathBuf>,
    mut cursors: HashMap<PathBuf, Cursor>,
    stop: &AtomicBool,
    shared: &Mutex<Shared>,
) {
    while !stop.load(Ordering::Acquire) {
        let (files, limited) = discover(&roots);
        if let Ok(mut s) = shared.lock() {
            s.health.files = files.len();
            s.health.scan_limited |= limited;
        }
        let active: HashSet<_> = files.iter().cloned().collect();
        for path in files {
            if stop.load(Ordering::Acquire) {
                break;
            }
            if let Err(error) = read_file(&path, &mut cursors, shared) {
                if let Ok(mut s) = shared.lock() {
                    s.health.io_errors += 1;
                    s.health.last_error = Some(error.to_string());
                }
            }
        }
        if cursors.len() > MAX_FILES {
            cursors.retain(|p, _| active.contains(p));
        }
        thread::park_timeout(Duration::from_millis(250));
    }
}
fn read_file(
    path: &Path,
    cursors: &mut HashMap<PathBuf, Cursor>,
    shared: &Mutex<Shared>,
) -> std::io::Result<()> {
    // Reopen each turn so renamed/deleted files are not held open.
    if fs::symlink_metadata(path).is_ok_and(|m| is_link(&m)) {
        return Ok(());
    }
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let len = metadata.len();
    let modified = metadata.modified().ok();
    let created = metadata.created().ok();
    let cursor = cursors
        .entry(path.to_path_buf())
        .or_insert_with(|| new_cursor(&metadata, true, shared));
    if len < cursor.offset
        || created != cursor.created
        || (modified != cursor.modified && (snapshot(path) || len == cursor.offset))
    {
        cursor.generation = cursor.generation.saturating_add(1);
        cursor.offset = 0;
        cursor.start = 0;
        cursor.pending.clear();
        cursor.skip_partial = false;
        if let Ok(mut s) = shared.lock() {
            s.health.rotations += 1;
        }
    }
    cursor.modified = modified;
    cursor.created = created;
    if snapshot(path) {
        if cursor.offset == len {
            return Ok(());
        }
        if len > READ_BUDGET {
            if let Ok(mut s) = shared.lock() {
                s.health.rejected += 1;
                s.health.skipped_bytes += len;
                s.health.last_error = Some("snapshot_too_large".into());
            }
            cursor.offset = len;
            return Ok(());
        }
        let mut bytes = Vec::new();
        file.take(READ_BUDGET).read_to_end(&mut bytes)?;
        // A writer may still be replacing the snapshot. Retry incomplete JSON on the next turn.
        match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(_) => emit(
                &bytes,
                stable_source(path, cursor.generation),
                0,
                false,
                shared,
            ),
            Err(e) if e.is_eof() => return Ok(()),
            Err(_) => emit(
                &bytes,
                stable_source(path, cursor.generation),
                0,
                false,
                shared,
            ),
        }
        cursor.offset = len;
        return Ok(());
    }
    file.seek(SeekFrom::Start(cursor.offset))?;
    let mut bytes = Vec::new();
    file.take(READ_BUDGET).read_to_end(&mut bytes)?;
    cursor.offset = cursor.offset.saturating_add(bytes.len() as u64);
    cursor.pending.extend_from_slice(&bytes);
    let mut used = 0;
    while let Some(end) = cursor.pending[used..].iter().position(|b| *b == b'\n') {
        let end = used + end + 1;
        if cursor.skip_partial {
            cursor.skip_partial = false;
        } else {
            let source = stable_source(path, cursor.generation);
            emit(
                &cursor.pending[used..end],
                source,
                cursor.start + used as u64,
                text_mode(path),
                shared,
            );
        }
        used = end;
    }
    if used > 0 {
        cursor.pending.drain(..used);
        cursor.start += used as u64;
    }
    if cursor.pending.len() > MAX_LINE {
        if let Ok(mut s) = shared.lock() {
            s.health.rejected += 1;
            s.health.skipped_bytes += cursor.pending.len() as u64;
            s.health.last_error = Some("line_too_large".into());
        }
        cursor.pending.clear();
        cursor.start = cursor.offset;
        cursor.skip_partial = true;
    }
    Ok(())
}
fn emit(bytes: &[u8], source: u64, offset: u64, text_mode: bool, shared: &Mutex<Shared>) {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return;
    }
    let result = if bytes.len() > MAX_LINE {
        Err("line_too_large".into())
    } else {
        normalize_record(bytes, source, offset, now_ms(), text_mode)
    };
    if let Ok(mut s) = shared.lock() {
        s.health.lines += 1;
        match result {
            Ok(events) => {
                for event in events {
                    if s.queue.len() >= QUEUE_CAPACITY {
                        s.queue.pop_front();
                        s.health.dropped += 1;
                    }
                    s.health.recognized += 1;
                    s.health.last_activity_ms = Some(now_ms());
                    s.queue.push_back(event);
                }
            }
            Err(error) => {
                s.health.rejected += 1;
                s.health.last_error = Some(error);
            }
        }
    }
}
fn stable_source(path: &Path, generation: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path
        .to_string_lossy()
        .bytes()
        .chain(generation.to_le_bytes())
    {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    hash
}
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}
