use clens_types::{HarnessEvent, Phase, ToolCall};
use serde_json::{json, Value};

/// Vendor records are client reports, never fabricated kernel observations.
pub fn normalize_record(
    bytes: &[u8],
    source_id: u64,
    offset: u64,
    received_ms: i64,
    text_mode: bool,
) -> Result<Vec<HarnessEvent>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .trim_start_matches('\u{feff}')
        .trim();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) if text_mode && !text.starts_with(['{', '[']) => {
            return Ok(vec![make_event(
                None,
                Phase::Reasoning,
                text,
                received_ms,
                source_id,
                offset,
                0,
            )]);
        }
        Err(e) => return Err(format!("invalid_json: {e}")),
    };
    let mut output = Vec::new();
    visit(&value, received_ms, source_id, offset, 0, &mut output)?;
    if output.len() > 512
        || output
            .iter()
            .any(|e| serde_json::to_vec(e).map_or(true, |b| b.len() > 16 * 1024))
    {
        return Err("record_expansion_limit".into());
    }
    if output.is_empty() {
        return Err("unsupported_schema".into());
    }
    Ok(output)
}
fn visit(
    v: &Value,
    now: i64,
    source: u64,
    offset: u64,
    depth: usize,
    out: &mut Vec<HarnessEvent>,
) -> Result<(), String> {
    if depth > 12 || out.len() >= 512 {
        return Err("record_expansion_limit".into());
    }
    if let Some(array) = v.as_array() {
        for item in array {
            visit(item, now, source, offset, depth + 1, out)?;
        }
        return Ok(());
    }
    if !v.is_object() {
        return Ok(());
    }
    let ts = v
        .get("ts_utc_ms")
        .and_then(Value::as_i64)
        .or_else(|| {
            v.get("timestamp")
                .or_else(|| v.get("ts"))
                .and_then(timestamp_ms)
        })
        .unwrap_or(now);
    if v.get("schema_v").is_some() && v.get("phase").is_some() && v.get("text").is_some() {
        let mut event =
            crate::parse_jsonl_event(v.to_string().as_bytes()).map_err(|e| e.to_string())?;
        event.harness_log_id = source;
        event.line_no = offset
            .saturating_mul(1024)
            .saturating_add(out.len() as u64 + 1);
        event.step = event.line_no;
        if event.phase != Phase::ToolCall {
            event.tool = None;
        }
        out.push(event);
        return Ok(());
    }
    if let Some(messages) = v
        .get("messages")
        .or_else(|| v.get("sessions"))
        .and_then(Value::as_array)
    {
        for msg in messages {
            visit(msg, ts, source, offset, depth + 1, out)?;
        }
        return Ok(());
    }
    if let Some(message) = v.get("message").filter(|m| m.is_object()) {
        return visit(message, ts, source, offset, depth + 1, out);
    }
    let event_type = v
        .get("type")
        .or_else(|| v.get("event"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if matches!(
        event_type,
        "tool.execution_start" | "tool.execution_complete"
    ) {
        if let Some(data) = v.get("data") {
            if event_type == "tool.execution_complete" {
                out.push(make_event(
                    None,
                    Phase::ToolResult,
                    &v.to_string(),
                    ts,
                    source,
                    offset,
                    out.len(),
                ));
            } else {
                let name = data
                    .get("toolName")
                    .or_else(|| data.get("name"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let args = data.get("arguments").cloned().unwrap_or(json!({}));
                push_tool(
                    &json!({"name":name,"arguments":args}),
                    ts,
                    source,
                    offset,
                    out,
                );
            }
            return Ok(());
        }
    }
    if let Some(call) = v.get("tool_call").and_then(Value::as_object) {
        if v.get("subtype").and_then(Value::as_str) == Some("completed") {
            out.push(make_event(
                None,
                Phase::ToolResult,
                &v.to_string(),
                ts,
                source,
                offset,
                out.len(),
            ));
        } else {
            for (key, canonical) in [
                ("readToolCall", "read"),
                ("writeToolCall", "write"),
                ("editToolCall", "edit"),
                ("shellToolCall", "shell"),
            ] {
                if let Some(payload) = call.get(key) {
                    push_tool(
                        &json!({"name":canonical,"arguments":payload.get("args").unwrap_or(payload)}),
                        ts,
                        source,
                        offset,
                        out,
                    );
                }
            }
        }
        return Ok(());
    }
    if matches!(event_type, "response_item" | "event_msg") {
        if let Some(payload) = v.get("payload") {
            return visit(payload, ts, source, offset, depth + 1, out);
        }
    }
    if event_type.starts_with("item.") {
        if let Some(item) = v.get("item") {
            let kind = item.get("type").and_then(Value::as_str).unwrap_or("");
            if kind == "command_execution" {
                let command = item.get("command").and_then(Value::as_str).unwrap_or("");
                let tool = (event_type == "item.started").then(|| ToolCall {
                    name: "shell".into(),
                    arguments: json!({"cmd":command}),
                });
                let phase = if tool.is_some() {
                    Phase::ToolCall
                } else {
                    Phase::ToolResult
                };
                out.push(make_event(
                    tool,
                    phase,
                    &item.to_string(),
                    ts,
                    source,
                    offset,
                    out.len(),
                ));
                return Ok(());
            }
            return visit(item, ts, source, offset, depth + 1, out);
        }
    }
    if matches!(
        event_type,
        "tool_result" | "function_call_output" | "post_tool_call" | "tool.execution_complete"
    ) || v.get("role").and_then(Value::as_str) == Some("tool")
    {
        out.push(make_event(
            None,
            Phase::ToolResult,
            &v.to_string(),
            ts,
            source,
            offset,
            out.len(),
        ));
        return Ok(());
    }
    if let Some(calls) = v.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            let call = call.get("function").unwrap_or(call);
            push_tool(call, ts, source, offset, out);
        }
    }
    if let Some(content) = v.get("content").and_then(Value::as_array) {
        for block in content {
            let kind = block.get("type").and_then(Value::as_str).unwrap_or("");
            if matches!(kind, "tool_use" | "toolCall" | "tool_call") {
                push_tool(block, ts, source, offset, out);
            } else if kind == "tool_result" {
                out.push(make_event(
                    None,
                    Phase::ToolResult,
                    &block.to_string(),
                    ts,
                    source,
                    offset,
                    out.len(),
                ));
            } else if let Some(text) = block.get("text").and_then(Value::as_str) {
                out.push(make_event(
                    None,
                    Phase::Reasoning,
                    text,
                    ts,
                    source,
                    offset,
                    out.len(),
                ));
            }
        }
    } else if let Some(text) = v.get("content").and_then(Value::as_str) {
        out.push(make_event(
            None,
            Phase::Reasoning,
            text,
            ts,
            source,
            offset,
            out.len(),
        ));
    }
    if matches!(
        event_type,
        "function_call" | "tool_call" | "tool_use" | "pre_tool_call"
    ) || v.get("tool_name").is_some()
    {
        push_tool(v, ts, source, offset, out);
    } else if v.get("tool").is_some_and(Value::is_object) {
        if let Some(tool) = v.get("tool") {
            push_tool(tool, ts, source, offset, out);
        }
    } else if let Some(text) = v
        .get("text")
        .or_else(|| v.get("message"))
        .and_then(Value::as_str)
    {
        out.push(make_event(
            None,
            Phase::Reasoning,
            text,
            ts,
            source,
            offset,
            out.len(),
        ));
    } else if matches!(
        event_type,
        "thread.started"
            | "turn.started"
            | "turn.completed"
            | "turn.failed"
            | "system"
            | "result"
            | "task.completed"
    ) {
        out.push(make_event(
            None,
            Phase::System,
            &v.to_string(),
            ts,
            source,
            offset,
            out.len(),
        ));
    }
    Ok(())
}
fn push_tool(v: &Value, ts: i64, source: u64, offset: u64, out: &mut Vec<HarnessEvent>) {
    let Some(name) = v
        .get("name")
        .or_else(|| v.get("tool_name"))
        .and_then(Value::as_str)
    else {
        return;
    };
    let args = v
        .get("arguments")
        .or_else(|| v.get("input"))
        .or_else(|| v.get("args"))
        .cloned()
        .unwrap_or(json!({}));
    let mut args = match args {
        Value::String(s) => serde_json::from_str(&s).unwrap_or(json!({"cmd":s})),
        other => other,
    };
    let canonical = match name.to_ascii_lowercase().as_str() {
        "read" | "read_file" | "readfile" | "file_read" => "read",
        "write" | "write_file" | "write_to_file" | "file_write" => "write",
        "edit" | "edit_file" | "replace" | "str_replace" => "edit",
        "bash" | "powershell" | "shell" | "exec" | "exec_command" | "run_command" | "terminal" => {
            "shell"
        }
        "create" | "create_file" => "create",
        "delete" | "delete_file" => "delete",
        "connect" => "connect",
        "bind" => "bind",
        "listen" => "listen",
        _ => name,
    };
    if let Some(object) = args.as_object_mut() {
        if !object.contains_key("path") {
            if let Some(p) = object
                .get("file_path")
                .or_else(|| object.get("filepath"))
                .or_else(|| object.get("target_file"))
                .cloned()
            {
                object.insert("path".into(), p);
            }
        }
        if !object.contains_key("cmd") {
            if let Some(cmd) = object.get("command").cloned() {
                object.insert("cmd".into(), cmd);
            }
        }
    }
    let tool = ToolCall {
        name: canonical.into(),
        arguments: args,
    };
    out.push(make_event(
        Some(tool),
        Phase::ToolCall,
        &v.to_string(),
        ts,
        source,
        offset,
        out.len(),
    ));
}
fn make_event(
    tool: Option<ToolCall>,
    phase: Phase,
    text: &str,
    ts: i64,
    source: u64,
    offset: u64,
    slot: usize,
) -> HarnessEvent {
    let index = offset.saturating_mul(1024).saturating_add(slot as u64 + 1);
    HarnessEvent {
        schema_v: 1,
        ts_utc_ms: ts,
        step: index,
        phase,
        tool,
        text: text.chars().take(4096).collect::<String>().into(),
        harness_log_id: source,
        line_no: index,
        byte_start: offset,
        byte_end: offset.saturating_add(text.len() as u64),
        raw_json: None,
    }
}
fn timestamp_ms(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        return Some(if n.unsigned_abs() < 100_000_000_000 {
            n.saturating_mul(1000)
        } else {
            n
        });
    }
    let s = v.as_str()?;
    // Parse RFC3339 without a platform clock or an additional runtime.
    if s.len() < 20 {
        return None;
    }
    let num = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let y = num(0, 4)?;
    let month = num(5, 7)?;
    let day = num(8, 10)?;
    let hour = num(11, 13)?;
    let minute = num(14, 16)?;
    let second = num(17, 19)?;
    if s.get(4..5)? != "-"
        || s.get(7..8)? != "-"
        || !matches!(s.get(10..11)?, "T" | "t" | " ")
        || s.get(13..14)? != ":"
        || s.get(16..17)? != ":"
    {
        return None;
    }
    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day < 1 || day > days {
        return None;
    }
    let yr = y - i64::from(month <= 2);
    let era = yr.div_euclid(400);
    let yoe = yr - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let epoch_days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    let tail = s.get(19..)?;
    let zone_start = tail.find(['Z', 'z', '+', '-'])?;
    let fraction = &tail[..zone_start];
    let millis = if fraction.is_empty() {
        0
    } else {
        let digits = fraction.strip_prefix('.')?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let padded = format!("{digits:0<3}");
        padded.get(..3)?.parse::<i64>().ok()?
    };
    let zone = &tail[zone_start..];
    let offset = if matches!(zone, "Z" | "z") {
        0
    } else {
        if zone.len() != 6 || zone.get(3..4)? != ":" {
            return None;
        }
        let h = zone.get(1..3)?.parse::<i64>().ok()?;
        let m = zone.get(4..6)?.parse::<i64>().ok()?;
        if h > 23 || m > 59 {
            return None;
        }
        (h * 60 + m) * if zone.starts_with('-') { -1 } else { 1 }
    };
    Some(((epoch_days * 24 + hour) * 3600 + minute * 60 + second - offset * 60) * 1000 + millis)
}
