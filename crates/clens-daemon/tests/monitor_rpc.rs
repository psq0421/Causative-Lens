use clens_daemon::{DaemonState, RpcRequest};
use serde_json::{json, Value};
use std::{
    fs, thread,
    time::{Duration, Instant},
};

fn call(state: &mut DaemonState, method: &str, params: Value) -> Value {
    let response = state.handle(RpcRequest {
        id: 1,
        method: method.into(),
        params,
    });
    assert!(response.error.is_none(), "{:?}", response.error);
    response.result.expect("result")
}
#[test]
fn monitoring_roundtrip_filters_categories_and_retains_after_stop() {
    let path =
        std::env::temp_dir().join(format!("causativelens-rpc-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&path).expect("dir");
    fs::write(
        path.join("trace.jsonl"),
        concat!(
            "{\"tool_name\":\"read\",\"args\":{\"path\":\"C:\\\\demo\\\\a.txt\"}}\n",
            "{\"tool_name\":\"connect\",\"args\":{\"target\":\"api.example.com:443\"}}\n",
            "{\"new_unknown_schema\":true}\n"
        ),
    )
    .expect("trace");
    let mut state = DaemonState::new();
    let catalog = call(&mut state, "monitor.templates", json!({}));
    assert!(catalog["templates"].as_array().expect("templates").len() >= 19);
    let ticket = call(
        &mut state,
        "monitor.start",
        json!({"runtime_id":"others","folder":path,"include_profile":false,"read_existing":true}),
    );
    let id = ticket["session_id"].clone();
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut rows = Value::Null;
    while Instant::now() < deadline {
        rows = call(
            &mut state,
            "monitor.events",
            json!({"session_id":id,"after":0,"limit":100}),
        );
        if rows["rows"].as_array().is_some_and(|r| r.len() >= 4) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let all = rows["rows"].as_array().expect("rows");
    assert!(all
        .iter()
        .any(|r| r["category"] == "files" && r["source"] == "agent"));
    assert!(all.iter().any(|r| r["category"] == "network"));
    assert!(all.iter().any(|r| r["category"] == "assertions"));
    assert!(
        !all.iter().any(|r| r["source"] == "system"),
        "never invent ETW"
    );
    let cursor = rows["next_cursor"].clone();
    let empty = call(
        &mut state,
        "monitor.events",
        json!({"session_id":id,"after":cursor}),
    );
    assert!(empty["rows"].as_array().expect("rows").is_empty());
    let status = call(&mut state, "monitor.status", json!({"session_id":id}));
    assert_eq!(status["capture_connected"], false);
    assert!(status["health"]["rejected"].as_u64().expect("rejected") >= 1);
    call(&mut state, "monitor.stop", json!({"session_id":id}));
    call(&mut state, "monitor.stop", json!({"session_id":id}));
    let status = call(&mut state, "monitor.status", json!({"session_id":id}));
    assert_eq!(status["health"]["running"], false);
    assert!(
        !call(&mut state, "monitor.events", json!({"session_id":id}))["rows"]
            .as_array()
            .expect("rows")
            .is_empty()
    );
    call(&mut state, "session.close", json!({"session_id":id}));
    assert_eq!(state.session_count(), 0);
    fs::remove_dir_all(path).expect("cleanup");
}
#[test]
fn missing_or_unknown_session_and_runtime_return_errors() {
    let mut state = DaemonState::new();
    for (method, params) in [
        ("monitor.events", json!({})),
        ("monitor.stop", json!({"session_id":uuid::Uuid::new_v4()})),
        ("monitor.probe", json!({"runtime_id":"fake","folder":"."})),
        (
            "monitor.start",
            json!({"runtime_id":"others","folder":"missing-folder-do-not-create"}),
        ),
    ] {
        assert!(
            state
                .handle(RpcRequest {
                    id: 1,
                    method: method.into(),
                    params
                })
                .error
                .is_some(),
            "{method}"
        );
    }
}
