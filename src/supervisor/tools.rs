use crate::supervisor::SupervisorHandle;
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct SupervisorTools {
    handle: SupervisorHandle,
}

impl SupervisorTools {
    pub fn new(handle: SupervisorHandle) -> Self {
        Self { handle }
    }

    pub fn status(&self) -> Result<Value> {
        let status = self.handle.get_status();
        Ok(serde_json::to_value(status)?)
    }

    pub fn restart(&self, _force: bool) -> Result<Value> {
        self.handle
            .restart()
            .map_err(|e| anyhow!("Failed to send restart command: {}", e))?;
        Ok(serde_json::json!({ "accepted": true }))
    }

    pub fn logs(&self, limit: usize, offset: usize) -> Result<Value> {
        let all_logs = self.handle.log_buffer.read();

        let total = all_logs.len();
        if offset >= total {
            return Ok(
                serde_json::json!({ "logs": [], "total": total, "offset": offset, "limit": limit }),
            );
        }

        let end = std::cmp::min(offset + limit, total);
        let logs = all_logs[offset..end].to_vec();

        Ok(serde_json::json!({
            "logs": logs,
            "total": total,
            "offset": offset,
            "limit": limit
        }))
    }
}
