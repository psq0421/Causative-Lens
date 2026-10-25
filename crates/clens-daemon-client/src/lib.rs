#[cfg(windows)]
use clens_types::RPC_PIPE_NAME;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("daemon transport failed: {0}")]
    Transport(String),
    #[error("daemon returned invalid JSON: {0}")]
    InvalidResponse(String),
    #[error("daemon returned error: {0}")]
    Remote(String),
}

#[derive(Debug, Default)]
pub struct RpcClient {
    next_id: u64,
}

impl RpcClient {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value, ClientError> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);
        let mut request =
            serde_json::to_vec(&serde_json::json!({"id": id, "method": method, "params": params}))
                .map_err(|error| ClientError::InvalidResponse(error.to_string()))?;
        request.push(b'\n');
        let response = round_trip(&request)?;
        let value: Value = serde_json::from_slice(&response)
            .map_err(|error| ClientError::InvalidResponse(error.to_string()))?;
        if let Some(error) = value.get("error") {
            return Err(ClientError::Remote(error.to_string()));
        }
        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }
}

#[cfg(windows)]
fn round_trip(request: &[u8]) -> Result<Vec<u8>, ClientError> {
    use std::io::{BufRead, BufReader, Read, Write};

    // The daemon serializes requests. A concurrent GUI poll may briefly occupy
    // its single pipe instance; retry only opening, never replay a sent mutation.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut stream = loop {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(RPC_PIPE_NAME)
        {
            Ok(stream) => break stream,
            Err(error)
                if error.raw_os_error() == Some(231) && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(std::time::Duration::from_millis(15));
            }
            Err(error) => return Err(ClientError::Transport(error.to_string())),
        }
    };
    stream
        .write_all(request)
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    stream
        .flush()
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    let mut reader = BufReader::new(stream);
    let mut response = Vec::new();
    reader
        .by_ref()
        .take(clens_types::RPC_MAX_MESSAGE_BYTES as u64 + 1)
        .read_until(b'\n', &mut response)
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    if response.len() > clens_types::RPC_MAX_MESSAGE_BYTES {
        return Err(ClientError::InvalidResponse("response_too_large".into()));
    }
    Ok(response)
}

#[cfg(unix)]
fn round_trip(request: &[u8]) -> Result<Vec<u8>, ClientError> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    let path = std::env::var("CAUSATIVELENS_RPC_SOCKET")
        .unwrap_or_else(|_| "/tmp/causativelens-rpc".to_owned());
    let mut stream =
        UnixStream::connect(path).map_err(|error| ClientError::Transport(error.to_string()))?;
    stream
        .write_all(request)
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    stream
        .flush()
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    let mut reader = BufReader::new(stream);
    let mut response = Vec::new();
    reader
        .read_until(b'\n', &mut response)
        .map_err(|error| ClientError::Transport(error.to_string()))?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_starts_request_ids_at_one() {
        let client = RpcClient::new();
        assert_eq!(client.next_id, 1);
    }

    #[cfg(windows)]
    #[test]
    fn client_uses_the_causativelens_pipe_contract() {
        assert_eq!(RPC_PIPE_NAME, r"\\.\pipe\causativelens-rpc");
    }
}
