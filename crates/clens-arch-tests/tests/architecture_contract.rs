#[test]
fn observe_mode_does_not_expose_intervention_commands() {
    let commands = clens_cli::OBSERVE_COMMANDS;
    assert!(!commands
        .iter()
        .any(|command| ["advise", "enforce", "decision"].contains(command)));
    assert!(clens_cli::DISABLED_COMMANDS
        .iter()
        .all(|command| !commands.contains(command)));
}

#[test]
fn gui_boundary_is_rpc_only_by_contract() {
    assert_eq!(clens_types::RPC_PIPE_NAME, r"\\.\pipe\causativelens-rpc");
}

#[test]
fn gui_crate_cannot_link_engine_or_storage_crates() {
    let manifest = include_str!("../../clens-gui/Cargo.toml");
    for forbidden in [
        "clens-core",
        "clens-storage",
        "clens-capture",
        "clens-harnessd",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "GUI must stay RPC-only: {forbidden}"
        );
    }
    assert!(manifest.contains("tauri"));
}

#[test]
fn cli_is_a_daemon_client_and_does_not_link_the_engine_process() {
    let manifest = include_str!("../../clens-cli/Cargo.toml");
    assert!(!manifest.contains("clens-daemon ="));
    assert!(manifest.contains("clens-daemon-client"));
}

#[test]
fn capture_manifest_keeps_the_privileged_surface_explicit() {
    let manifest = include_str!("../../clens-capture/src/clens-capture.manifest");
    assert!(manifest.contains("level=\"requireAdministrator\""));
    assert!(manifest.contains("longPathAware"));
}

#[test]
fn daemon_has_a_cross_platform_transport_entrypoint() {
    let source = include_str!("../../clens-daemon/src/lib.rs");
    assert!(source.contains("pub fn run_named_pipe"));
    assert!(source.contains("pub fn run_unix_socket"));
}

#[test]
fn ipc_security_contract_keeps_ring_limits_and_owner_only_pipe() {
    assert_eq!(clens_capture::DATA_RING_CAPACITY_BYTES, 256 * 1024 * 1024);
    assert_eq!(clens_capture::DATA_RING_SPOOL_WATERMARK_PERCENT, 95);
    assert_eq!(clens_types::CAPTURE_CONTROL_MAX_BYTES, 4096);
    let source = include_str!("../../clens-daemon/src/lib.rs");
    assert!(source.contains("D:P(A;;GA;;;OW)"));
    assert!(source.contains("read_bounded_line"));
}

#[test]
fn capture_control_protocol_is_explicitly_observe_only() {
    let source = include_str!("../../clens-types/src/lib.rs");
    assert!(source.contains("CaptureControlMessage"));
    assert!(source.contains("CAPTURE_FAILURE_CODE"));
}

#[test]
fn storage_exposes_append_only_spool_recovery() {
    let source = include_str!("../../clens-storage/src/lib.rs");
    assert!(source.contains("pub struct SpoolWriter"));
    assert!(source.contains("pub fn recover_file"));
}

#[test]
fn thread_topology_matches_the_fixed_thread_model() {
    assert_eq!(clens_capture::capture_thread_plan().len(), 6);
    let daemon = clens_daemon::daemon_thread_plan(8);
    assert_eq!(daemon.spool_reader, 1);
    assert_eq!(daemon.harness_tail, 1);
    assert_eq!(daemon.assertion_workers, 4);
    assert_eq!(daemon.alignment, 1);
    assert_eq!(daemon.sqlite_writer, 1);
}

#[test]
fn windows_release_targets_request_static_crt() {
    let config = include_str!("../../../.cargo/config.toml");
    assert!(config.contains("x86_64-pc-windows-msvc"));
    assert!(config.contains("target-feature=+crt-static"));
}

#[test]
fn gui_frontend_has_fluent_tokens_and_performance_guards() {
    let styles = include_str!("../../clens-gui/frontend/styles.css");
    let app = include_str!("../../clens-gui/frontend/app.js");
    assert!(styles.contains("Segoe UI"));
    assert!(styles.contains("--accent"));
    assert!(app.contains("slice(-500)"));
    assert!(app.contains("setTimeout(() =>"));
}

#[test]
fn all_frontends_expose_the_three_supported_locales() {
    assert_eq!(clens_types::UiLanguage::ALL.len(), 3);
    let i18n = include_str!("../../clens-gui/frontend/i18n.js");
    let html = include_str!("../../clens-gui/frontend/index.html");
    assert!(i18n.contains("zh-CN") && i18n.contains("zh-TW") && i18n.contains("en-US"));
    assert!(html.contains("language-select"));
    assert!(html.contains("data-i18n"));
}
