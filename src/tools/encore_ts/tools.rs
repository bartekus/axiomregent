// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use crate::tools::encore_ts::state::EncoreState;
use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct EncoreTools {
    state: Arc<Mutex<EncoreState>>,
}

impl Default for EncoreTools {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoreTools {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(EncoreState::new())),
        }
    }

    pub fn env_check(&self) -> Result<Value> {
        let info = crate::tools::encore_ts::env::check()?;
        Ok(serde_json::to_value(info)?)
    }

    pub fn parse(&self, root: &Path) -> Result<Value> {
        let snapshot = crate::tools::encore_ts::parse::parse(root)?;
        Ok(serde_json::to_value(snapshot)?)
    }

    pub fn meta(&self, root: &Path) -> Result<Value> {
        self.parse(root)
    }

    pub fn run_start(
        &self,
        root: &Path,
        env: Option<HashMap<String, String>>,
        _profile: Option<String>,
    ) -> Result<Value> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| anyhow!("State lock failed: {}", e))?;
        let run_id = crate::tools::encore_ts::run::start(&mut state, root, env)?;
        Ok(serde_json::json!({ "run_id": run_id }))
    }

    pub fn run_stop(&self, run_id: &str) -> Result<Value> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| anyhow!("State lock failed: {}", e))?;
        let stopped = crate::tools::encore_ts::run::stop(&mut state, run_id)?;
        Ok(serde_json::json!({ "stopped": stopped }))
    }

    pub fn logs_stream(&self, run_id: &str, from_seq: Option<u64>) -> Result<Value> {
        let state = self
            .state
            .lock()
            .map_err(|e| anyhow!("State lock failed: {}", e))?;

        if let Some(process) = state.processes.get(run_id) {
            let buffer = process
                .log_buffer
                .lock()
                .map_err(|e| anyhow!("Log lock failed: {}", e))?;
            let start_idx = from_seq.unwrap_or(0) as usize;

            if start_idx >= buffer.len() {
                return Ok(serde_json::json!({ "logs": [], "next_seq": buffer.len() }));
            }

            let logs = buffer[start_idx..].to_vec();
            let next_seq = buffer.len();

            Ok(serde_json::json!({ "logs": logs, "next_seq": next_seq }))
        } else {
            // Try reading from disk for replay
            let cwd = std::env::current_dir().context("Failed to get current directory")?;
            let run_dir = cwd.join(".axiomregent").join("runs").join(run_id);
            let logs_path = run_dir.join("logs.ndjson");

            if logs_path.exists() {
                let file = std::fs::File::open(&logs_path)?;
                let reader = std::io::BufReader::new(file);
                // We need to collect all lines to slice. Not efficient for huge files but ok for tool implementation baseline.
                let lines: Result<Vec<String>, _> = reader.lines().collect();
                let lines = lines.context("Failed to read logs lines")?;

                let start_idx = from_seq.unwrap_or(0) as usize;
                if start_idx >= lines.len() {
                    return Ok(serde_json::json!({ "logs": [], "next_seq": lines.len() }));
                }
                let logs = lines[start_idx..].to_vec();
                let next_seq = lines.len();
                Ok(serde_json::json!({ "logs": logs, "next_seq": next_seq }))
            } else {
                Err(anyhow!("Process not found: {}", run_id))
            }
        }
    }
}
