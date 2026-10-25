use super::*;
use clens_harnessd::monitor::{self, LiveMonitor, MonitorConfig};
use std::collections::VecDeque;
use std::path::PathBuf;

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
    pub(super) fn pump_monitor(&mut self) {
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
        json!({
            "session_id":id,
            "runtime_id":config.map(|c|&c.runtime_id),
            "folder":config.map(|c|&c.folder),
            "health":self.monitor.as_ref().map(LiveMonitor::health),
            "capture_connected":false,
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
                let id = Uuid::new_v4();
                let session = ObservationSession {
                    monitor: Some(reader),
                    ..ObservationSession::default()
                };
                self.sessions.insert(id, session);
                Ok(json!({"session_id":id,"mode":"observe","capture_connected":false}))
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
