use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

pub struct DaemonClient {
    stdin: tokio::process::ChildStdin,
    stdout_reader: BufReader<tokio::process::ChildStdout>,
    request_id: u64,
}

impl DaemonClient {
    pub fn new(child: &mut Child) -> Result<Self> {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Child has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Child has no stdout"))?;

        Ok(Self {
            stdin,
            stdout_reader: BufReader::new(stdout),
            request_id: 0,
        })
    }

    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<JsonRpcResponse> {
        self.request_id += 1;
        let id = self.request_id;

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        // Write request (Content-Length framed or line-delimited)
        // For now, let's use Line-Delimited JSON as per plan Phase 1 ("Robust framing: line-delimited JSON is OK")
        let mut json = serde_json::to_string(&req)?;
        json.push('\n');

        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;

        // Read response
        let mut line = String::new();
        let bytes_read = self.stdout_reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(anyhow!("Daemon closed stdout"));
        }

        let resp: JsonRpcResponse = serde_json::from_str(&line)?;

        // Check ID match
        if resp.id != Some(id) {
            return Err(anyhow!(
                "Response ID mismatch: expected {}, got {:?}",
                id,
                resp.id
            ));
        }

        Ok(resp)
    }

    pub async fn hello(&mut self) -> Result<()> {
        let resp = self
            .send_request(
                "daemon.hello",
                Some(json!({
                    "client": "axiomregent",
                    "version": "0.1.0"
                })),
            )
            .await?;

        if let Some(err) = resp.error {
            return Err(anyhow!("Daemon Hello failed: {}", err.message));
        }
        Ok(())
    }
}
