fn main() {
    #[cfg(windows)]
    {
        if let Err(error) = clens_daemon::run_named_pipe() {
            eprintln!("CausativeLens daemon pipe stopped: {error}");
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        if let Err(error) = clens_daemon::run_unix_socket() {
            eprintln!("CausativeLens daemon socket stopped: {error}");
            std::process::exit(1);
        }
    }
}
