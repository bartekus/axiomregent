use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct EncoreTools;

impl Default for EncoreTools {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoreTools {
    pub fn new() -> Self {
        Self
    }

    pub fn env_check(&self) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }

    pub fn parse(&self, _root: &Path) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }

    pub fn meta(&self, _root: &Path) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }

    pub fn run_start(
        &self,
        _root: &Path,
        _env: Option<HashMap<String, String>>,
        _profile: Option<String>,
    ) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }

    pub fn run_stop(&self, _run_id: &str) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }

    pub fn logs_stream(&self, _run_id: &str, _from_seq: Option<u64>) -> Result<Value> {
        Err(anyhow!("Not Implemented"))
    }
}
