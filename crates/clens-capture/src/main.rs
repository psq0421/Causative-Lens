fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--start") {
        let mut lifecycle = clens_capture::CaptureLifecycle::default();
        if let Err(error) = lifecycle.start_all() {
            eprintln!("CausativeLens capture start failed: {error}");
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
