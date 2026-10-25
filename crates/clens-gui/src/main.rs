#![cfg_attr(not(feature = "tauri-shell"), allow(dead_code))]

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
fn rpc_call(method: String, params: Value) -> Result<Value, String> {
    let mut client = RpcClient::new();
    client
        .call(&method, params)
        .map_err(|error| error.to_string())
}

#[cfg(feature = "tauri-shell")]
fn main() {
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", clens_types::DEV_VERSION);
        return;
    }
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![bootstrap, rpc_call])
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
