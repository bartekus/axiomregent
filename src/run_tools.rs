// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::error;
use run::{RunConfig, Runner, StateStore, registry};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

pub struct RunTools {
    runs: Arc<Mutex<HashMap<String, RunContext>>>,
    state_dir: PathBuf,
}

#[derive(Clone)]
struct RunContext {
    status: String, // "pending", "running", "completed", "failed"
    logs_path: PathBuf,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    exit_code: Option<i32>,
}

impl RunTools {
    pub fn new(root: &Path) -> Self {
        let state_dir = root.join(".axiomregent/run");
        let logs_dir = state_dir.join("logs");
        fs::create_dir_all(&logs_dir).unwrap_or(());

        Self {
            runs: Arc::new(Mutex::new(HashMap::new())),
            state_dir,
        }
    }

    pub fn execute(
        &self,
        skill: String,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let run_id = Uuid::new_v4().to_string();
        let logs_path = self.state_dir.join("logs").join(format!("{}.log", run_id));

        let context = RunContext {
            status: "pending".to_string(),
            logs_path: logs_path.clone(),
            start_time: Some(Utc::now()),
            end_time: None,
            exit_code: None,
        };

        {
            let mut runs = self.runs.lock().unwrap();
            runs.insert(run_id.clone(), context);
        }

        let runs_handle = self.runs.clone();
        let run_id_clone = run_id.clone();
        let state_dir_str = self.state_dir.to_string_lossy().into_owned();
        let logs_path_clone = logs_path.clone();

        thread::spawn(move || {
            {
                let mut runs = runs_handle.lock().unwrap();
                if let Some(ctx) = runs.get_mut(&run_id_clone) {
                    ctx.status = "running".to_string();
                }
            }

            let log_file = match File::create(&logs_path_clone) {
                Ok(f) => f,
                Err(e) => {
                    let mut runs = runs_handle.lock().unwrap();
                    if let Some(ctx) = runs.get_mut(&run_id_clone) {
                        ctx.status = "failed".to_string();
                        ctx.end_time = Some(Utc::now());
                    }
                    error!("Failed to create log file: {}", e);
                    return;
                }
            };

            // Setup RunConfig
            let current_exe = env::current_exe().unwrap_or_else(|_| "axiomregent".into());
            let bin_path = current_exe.to_string_lossy().into_owned();

            let config = RunConfig {
                json: false,
                state_dir: state_dir_str.clone(),
                fail_on_warning: false,
                files0: false, // Not supported in remote execution yet
                bin_path,
                stdin_buffer: None,
                env: env_vars.unwrap_or_default(),
            };

            let store = StateStore::new(&state_dir_str);
            let registry = registry::get_registry();
            let runner = Runner::new(registry, store, config, Some(Box::new(log_file)));

            let result = runner.run_specific(&[skill]);

            let mut runs = runs_handle.lock().unwrap();
            if let Some(ctx) = runs.get_mut(&run_id_clone) {
                ctx.end_time = Some(Utc::now());
                match result {
                    Ok(true) => {
                        ctx.status = "completed".to_string();
                        ctx.exit_code = Some(0);
                    }
                    Ok(false) => {
                        ctx.status = "failed".to_string();
                        ctx.exit_code = Some(1);
                    }
                    Err(_) => {
                        ctx.status = "failed".to_string();
                        ctx.exit_code = Some(1);
                    }
                }
            }
        });

        Ok(run_id)
    }

    pub fn status(&self, run_id: &str) -> Result<serde_json::Value> {
        let runs = self.runs.lock().unwrap();
        if let Some(ctx) = runs.get(run_id) {
            Ok(serde_json::json!({
                "status": ctx.status,
                "start_time": ctx.start_time,
                "end_time": ctx.end_time,
                "exit_code": ctx.exit_code
            }))
        } else {
            Ok(serde_json::json!({ "status": "unknown" }))
        }
    }

    pub fn logs(&self, run_id: &str, offset: Option<u64>, limit: Option<u64>) -> Result<String> {
        let logs_path = {
            let runs = self.runs.lock().unwrap();
            if let Some(ctx) = runs.get(run_id) {
                ctx.logs_path.clone()
            } else {
                return Ok("Run ID not found".to_string());
            }
        };

        if logs_path.exists() {
            let mut file = File::open(logs_path).context("opening log file")?;

            if let Some(off) = offset {
                file.seek(SeekFrom::Start(off))
                    .context("seeking log file")?;
            }

            let mut contents = String::new();
            if let Some(lim) = limit {
                // take() takes u64, read_to_string reads until limit.
                let mut handle = file.take(lim);
                handle
                    .read_to_string(&mut contents)
                    .context("reading log file")?;
            } else {
                file.read_to_string(&mut contents)
                    .context("reading log file")?;
            }

            Ok(contents)
        } else {
            Ok("".to_string())
        }
    }
}
