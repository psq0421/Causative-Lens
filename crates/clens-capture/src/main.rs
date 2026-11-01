#[cfg(windows)]
fn qpc_frequency() -> u64 {
    let mut frequency = 0_i64;
    let ok = unsafe {
        windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut frequency)
    };
    if ok != 0 && frequency > 0 {
        frequency as u64
    } else {
        10_000_000
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--live") {
        #[cfg(windows)]
        {
            let mut lifecycle = clens_capture::CaptureLifecycle::default();
            let attach_pid = match args.windows(2).find(|pair| pair[0] == "--attach-pid") {
                Some(pair) => match pair[1].parse::<u32>() {
                    Ok(pid) if pid != 0 => Some(pid),
                    _ => {
                        eprintln!("CausativeLens attach requires a non-zero numeric PID");
                        std::process::exit(2);
                    }
                },
                None => None,
            };
            let receiver_result = match attach_pid {
                Some(pid) => lifecycle.start_attach(pid),
                None => lifecycle.start_live(),
            };
            let receiver = match receiver_result {
                Ok(receiver) => receiver,
                Err(error) => {
                    eprintln!("CausativeLens realtime capture failed: {error}");
                    std::process::exit(1);
                }
            };
            let output = args
                .windows(2)
                .find(|pair| pair[0] == "--output")
                .map(|pair| std::path::PathBuf::from(&pair[1]));
            let qpc_frequency = qpc_frequency();
            let mut output_file = output.as_ref().and_then(|path| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .ok()
            });
            for record in receiver {
                let line = serde_json::json!({
                    "event": &record.event,
                    "path": &record.path,
                    "provider": &record.provider,
                    "provider_guid": format!(
                        "{:08x}-{:04x}-{:04x}",
                        record.provider_guid.data1,
                        record.provider_guid.data2,
                        record.provider_guid.data3
                    ),
                    "event_id": record.event_id,
                    "version": record.version,
                    "opcode": record.opcode,
                    "tdh_properties": &record.tdh_properties,
                    "user_data_len": record.user_data.len(),
                    "qpc_frequency": qpc_frequency
                });
                match serde_json::to_string(&line) {
                    Ok(line) => {
                        if let Some(file) = output_file.as_mut() {
                            use std::io::Write;
                            let _ = writeln!(file, "{line}");
                            let _ = file.flush();
                        } else {
                            println!("{line}");
                        }
                    }
                    Err(error) => eprintln!("CausativeLens capture serialization failed: {error}"),
                }
            }
            let _ = lifecycle.stop_all();
            return;
        }
        #[cfg(not(windows))]
        {
            eprintln!("CausativeLens realtime ETW capture requires Windows");
            std::process::exit(1);
        }
    }
    if args.iter().any(|arg| arg == "--start") {
        let mut lifecycle = clens_capture::CaptureLifecycle::default();
        if let Err(error) = lifecycle.start_all() {
            eprintln!("CausativeLens capture start failed: {error}");
            std::process::exit(1);
        }
        if let Err(error) = lifecycle.request_rundown() {
            eprintln!("CausativeLens capture rundown failed: {error}");
            std::process::exit(1);
        }
        println!(
            "CausativeLens capture active: {}",
            lifecycle.active_sessions().join(",")
        );
        return;
    }
    for session in clens_capture::session_specs() {
        println!("{} {}", session.name, session.provider);
    }
}
