use clens_harnessd::monitor::{normalize_record, probe, templates, LiveMonitor, MonitorConfig};
use clens_types::Phase;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn temp_dir() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "causativelens-monitor-{}-{}",
        std::process::id(),
        SystemTimeSeed::next()
    ));
    fs::create_dir_all(&p).expect("test directory");
    p
}
struct SystemTimeSeed;
impl SystemTimeSeed {
    fn next() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    }
}
#[test]
fn all_requested_templates_and_other_are_present() {
    let all = templates().expect("valid templates");
    for id in [
        "hermes",
        "claude-code",
        "codex",
        "antigravity",
        "openclaw",
        "manus",
        "autogpt",
        "codebuddy",
        "workbuddy-cn",
        "workbuddy-international",
        "trae",
        "aider",
        "zcode",
        "qoder",
        "codearts",
        "minimax",
        "copilot",
        "cursor",
        "others",
    ] {
        assert!(all.iter().any(|t| t.id == id), "missing {id}");
    }
    assert_eq!(
        all.iter()
            .map(|t| &t.id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        all.len()
    );
}
#[test]
fn native_transcripts_extract_intentions_and_do_not_duplicate_results() {
    let cases = [
        (
            r#"{"timestamp":"2026-10-25T04:00:00Z","message":{"role":"assistant","content":[{"type":"tool_use","id":"1","name":"Read","input":{"file_path":"C:\\demo\\a.txt"}}]}}"#,
            "read",
        ),
        (
            r#"{"type":"item.started","item":{"type":"command_execution","command":"cat a.txt"}}"#,
            "shell",
        ),
        (
            r#"{"message":{"role":"assistant","content":[{"type":"toolCall","id":"x","name":"read","arguments":{"path":"C:\\demo\\a.txt"}}]}}"#,
            "read",
        ),
        (
            r#"{"messages":[{"role":"assistant","tool_calls":[{"id":"x","function":{"name":"read_file","arguments":"{\"path\":\"C:\\\\demo\\\\a.txt\"}"}}]}]}"#,
            "read",
        ),
        (
            r#"{"tool_name":"write_file","args":{"path":"C:\\demo\\a.txt"},"event":"pre_tool_call"}"#,
            "write",
        ),
    ];
    for (raw, tool) in cases {
        let events = normalize_record(raw.as_bytes(), 77, 1, 1234, false).expect("recognized");
        assert!(
            events
                .iter()
                .any(|e| e.tool.as_ref().is_some_and(|t| t.name.as_ref() == tool)),
            "{raw}"
        );
        assert!(events
            .iter()
            .all(|e| e.harness_log_id == 77 && e.line_no >= e.step));
    }
    let events=normalize_record(br#"{"type":"item.completed","item":{"type":"command_execution","command":"cat a.txt","status":"completed"}}"#,77,1,1234,false).expect("result");
    assert!(events
        .iter()
        .all(|e| e.tool.is_none() && e.phase == Phase::ToolResult));
}
#[test]
fn timestamps_identity_unknown_and_text_are_honest() {
    let raw =
        br#"{"timestamp":"2026-10-25T12:00:00+08:00","tool_name":"read","args":{"path":"a"}}"#;
    let a = normalize_record(raw, 7, 21, 0, false).expect("event");
    let b = normalize_record(raw, 7, 21, 0, false).expect("replay");
    assert_eq!(a, b);
    assert_eq!(a[0].ts_utc_ms, 1_792_900_800_000);
    assert!(normalize_record(b"{bad", 7, 0, 0, false).is_err());
    assert!(normalize_record(br#"{"new_unknown_schema":true}"#, 7, 0, 0, false).is_err());
    let text = normalize_record(b"I might write a.txt", 7, 0, 123, true).expect("text");
    assert!(text.iter().all(|e| e.tool.is_none()));
}

#[test]
fn additional_vendor_calls_results_status_and_time_edges_are_preserved() {
    for raw in [
        r#"{"type":"tool.execution_start","data":{"toolName":"read_file","arguments":{"path":"a"}}}"#,
        r#"{"type":"tool_call","subtype":"started","tool_call":{"readToolCall":{"args":{"path":"a"}}}}"#,
        r#"{"type":"response_item","payload":{"type":"function_call","name":"exec_command","arguments":"{\"command\":\"dir\"}"}}"#,
        r#"{"schema_v":1,"ts_utc_ms":100,"step":1,"phase":"tool_call","text":"read a","tool":{"name":"read","arguments":{"path":"a"}}}"#,
    ] {
        assert!(normalize_record(raw.as_bytes(), 8, 10, 100, false)
            .expect("tool")
            .iter()
            .any(|e| e.tool.is_some()));
    }
    for raw in [
        r#"{"type":"tool.execution_complete","data":{"result":"done"}}"#,
        r#"{"tool_call":{},"subtype":"completed"}"#,
        r#"{"type":"function_call_output","output":"done"}"#,
        r#"{"role":"tool","content":"done"}"#,
        r#"{"content":[{"type":"tool_result","content":"done"}]}"#,
    ] {
        let events = normalize_record(raw.as_bytes(), 8, 10, 100, false).expect("result");
        assert!(events
            .iter()
            .all(|e| e.phase == Phase::ToolResult && e.tool.is_none()));
    }
    for raw in [
        r#"{"content":[{"type":"text","text":"hello"}]}"#,
        r#"{"content":"hello"}"#,
        r#"{"type":"thread.started"}"#,
        r#"[{"text":"a"},{"text":"b"}]"#,
        r#"{"sessions":[{"messages":[{"text":"a"}]}]}"#,
    ] {
        assert!(!normalize_record(raw.as_bytes(), 8, 10, 100, false)
            .expect("report")
            .is_empty());
    }
    for timestamp in [
        "invalid",
        "2026-02-30T00:00:00Z",
        "2026-01-01T25:00:00Z",
        "2026-01-01T00:00:00+30:00",
    ] {
        let raw = serde_json::json!({"timestamp":timestamp,"text":"hello"}).to_string();
        assert_eq!(
            normalize_record(raw.as_bytes(), 8, 10, 99, false).expect("fallback")[0].ts_utc_ms,
            99
        );
    }
    let raw =
        serde_json::json!({"timestamp":"2026-10-25T04:00:00.123Z","text":"hello"}).to_string();
    assert_eq!(
        normalize_record(raw.as_bytes(), 8, 10, 99, false).expect("fraction")[0].ts_utc_ms,
        1_792_900_800_123
    );
    assert!(normalize_record(&[0xff], 8, 0, 0, false).is_err());
    assert!(normalize_record(b"   ", 8, 0, 0, false)
        .expect("empty")
        .is_empty());
    let expansion = serde_json::json!({"messages":(0..513).map(|_|serde_json::json!({"text":"x"})).collect::<Vec<_>>()});
    assert!(normalize_record(expansion.to_string().as_bytes(), 8, 0, 0, false).is_err());
}
#[test]
fn directory_probe_rejects_non_directory_and_reports_files() {
    let dir = temp_dir();
    fs::write(
        dir.join("trace.jsonl"),
        r#"{"tool_name":"read","args":{"path":"a"}}"#,
    )
    .expect("fixture");
    let config = MonitorConfig {
        runtime_id: "others".into(),
        folder: dir.clone(),
        include_profile: false,
        read_existing: true,
    };
    let report = probe(&config).expect("probe");
    assert_eq!(report.files.len(), 1);
    assert!(!report.recognized.is_empty());
    let mut bad = config.clone();
    bad.folder = dir.join("trace.jsonl");
    assert!(probe(&bad).is_err());
    fs::remove_dir_all(dir).expect("cleanup");
}

fn wait_events(monitor: &LiveMonitor) -> Vec<clens_types::HarnessEvent> {
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        let events = monitor.drain();
        if !events.is_empty() {
            return events;
        }
        thread::sleep(Duration::from_millis(30));
    }
    Vec::new()
}

#[test]
fn pretty_json_snapshot_is_one_record_and_rewrite_is_detected() {
    let dir = temp_dir();
    let path = dir.join("session.json");
    fs::write(
        &path,
        "{\n  \"messages\": [\n    {\"tool_name\":\"read\",\"args\":{\"path\":\"a\"}}\n  ]\n}",
    )
    .expect("snapshot");
    let config = MonitorConfig {
        runtime_id: "others".into(),
        folder: dir.clone(),
        include_profile: false,
        read_existing: true,
    };
    assert!(probe(&config)
        .expect("probe")
        .recognized
        .contains(&"structured-tools".into()));
    let mut monitor = LiveMonitor::start(config).expect("start");
    assert_eq!(wait_events(&monitor).len(), 1);
    assert_eq!(monitor.health().rejected, 0);
    thread::sleep(Duration::from_millis(20));
    fs::write(
        &path,
        "{\n  \"messages\": [\n    {\"tool_name\":\"write\",\"args\":{\"path\":\"b\"}}\n  ]\n}",
    )
    .expect("replace");
    let events = wait_events(&monitor);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].tool.as_ref().expect("tool").name.as_ref(),
        "write"
    );
    assert!(monitor.health().rotations > 0);
    monitor.stop();
    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn start_without_history_has_no_race_and_new_files_start_at_zero() {
    let dir = temp_dir();
    let path = dir.join("trace.jsonl");
    let line = b"{\"tool_name\":\"read\",\"args\":{\"path\":\"a\"}}\n";
    fs::write(&path, line).expect("history");
    let mut monitor = LiveMonitor::start(MonitorConfig {
        runtime_id: "others".into(),
        folder: dir.clone(),
        include_profile: false,
        read_existing: false,
    })
    .expect("start");
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("append")
        .write_all(line)
        .expect("line");
    assert_eq!(
        wait_events(&monitor).len(),
        1,
        "append immediately after start must be read"
    );
    fs::write(dir.join("new.jsonl"), line).expect("new file");
    assert_eq!(
        wait_events(&monitor).len(),
        1,
        "a newly created file has no historical bytes"
    );
    monitor.stop();
    fs::remove_dir_all(dir).expect("cleanup");
}
#[test]
fn live_reader_handles_partial_append_and_stops_without_agent_execution() {
    let dir = temp_dir();
    let file = dir.join("trace.jsonl");
    fs::write(&file, b"").expect("empty trace");
    let mut monitor = LiveMonitor::start(MonitorConfig {
        runtime_id: "others".into(),
        folder: dir.clone(),
        include_profile: false,
        read_existing: true,
    })
    .expect("monitor");
    let mut writer = fs::OpenOptions::new()
        .append(true)
        .open(&file)
        .expect("writer");
    writer
        .write_all(br#"{"tool_name":"read","args":{"path":"a"}}"#)
        .expect("partial");
    writer.flush().expect("flush");
    thread::sleep(Duration::from_millis(350));
    assert!(
        monitor.drain().is_empty(),
        "partial line must stay buffered"
    );
    writer.write_all(b"\n").expect("newline");
    writer.flush().expect("flush");
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut received = Vec::new();
    while Instant::now() < deadline && received.is_empty() {
        received.extend(monitor.drain());
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(received.len(), 1);
    assert_eq!(monitor.health().recognized, 1);
    monitor.stop();
    assert!(!monitor.health().running);
    drop(writer);
    drop(monitor);
    fs::remove_dir_all(dir).expect("cleanup");
}
