#![cfg_attr(not(feature = "tauri-shell"), allow(dead_code))]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(feature = "tauri-shell")]
use clens_daemon_client::RpcClient;
#[cfg(feature = "tauri-shell")]
use clens_types::Mode;
#[cfg(feature = "tauri-shell")]
use serde_json::Value;

#[cfg(feature = "tauri-shell")]
#[derive(Debug, serde::Serialize)]
struct UiBootstrap {
    product: &'static str,
    version: &'static str,
    mode: Mode,
    transport: &'static str,
}

#[cfg(feature = "tauri-shell")]
#[tauri::command]
fn bootstrap() -> UiBootstrap {
    UiBootstrap {
        product: clens_types::PRODUCT_NAME,
        version: clens_types::DEV_VERSION,
        mode: Mode::Observe,
        transport: if cfg!(windows) {
            "named_pipe"
        } else {
            "unix_socket"
        },
    }
}

#[cfg(feature = "tauri-shell")]
#[tauri::command]
async fn rpc_call(method: String, params: Value) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        RpcClient::new()
            .call(&method, params)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(feature = "tauri-shell")]
fn hello() -> Result<Value, String> {
    RpcClient::new()
        .call(
            "hello",
            serde_json::json!({"protocol_version":1,"mode":"observe"}),
        )
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri-shell")]
#[tauri::command]
async fn connect_daemon() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        if let Ok(value) = hello() {
            return Ok(value);
        }
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let directory = executable.parent().ok_or("daemon_directory_missing")?;
        let daemon = directory.join(if cfg!(windows) {
            "clens-daemon.exe"
        } else {
            "clens-daemon"
        });
        if !daemon.is_file() {
            return Err("daemon_binary_missing: reinstall CausativeLens".into());
        }
        let mut command = std::process::Command::new(daemon);
        command.current_dir(directory);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let _child = command
            .spawn()
            .map_err(|e| format!("daemon_start_failed: {e}"))?;
        for _ in 0..40 {
            if let Ok(value) = hello() {
                return Ok(value);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Err("daemon_connect_timeout".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(feature = "tauri-shell")]
#[tauri::command]
async fn pick_monitor_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_folder()
            .map(|folder| {
                folder
                    .into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(feature = "tauri-shell")]
fn main() {
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", clens_types::DEV_VERSION);
        return;
    }
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            rpc_call,
            connect_daemon,
            pick_monitor_folder
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("CausativeLens GUI stopped: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "tauri-shell"))]
fn main() {
    println!("CausativeLens GUI shell source is ready; build with --features tauri-shell");
}
