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

    pub async fn ensure(&mut self, req: serde_json::Value) -> Result<serde_json::Value> {
        let resp = self.send_request("infra.ensure", Some(req)).await?;
        match resp.result {
            Some(val) => Ok(val),
            None => {
                if let Some(err) = resp.error {
                    Err(anyhow!("Infra Ensure failed: {}", err.message))
                } else {
                    Err(anyhow!("Infra Ensure returned empty result"))
                }
            }
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        let resp = self.send_request("daemon.shutdown", None).await?;
        if let Some(err) = resp.error {
            return Err(anyhow!("Daemon Shutdown failed: {}", err.message));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rpc_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "infra.ensure".to_string(),
            params: Some(json!({"foo": "bar"})),
        };
        let s = serde_json::to_string(&req).unwrap();
        assert_eq!(
            s,
            r#"{"jsonrpc":"2.0","id":1,"method":"infra.ensure","params":{"foo":"bar"}}"#
        );
    }
}
