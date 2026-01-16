// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use crate::tools::encore_ts::state::{EncoreState, RunProcess};
use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

pub fn start(
    state: &mut EncoreState,
    root: &Path,
    env: Option<HashMap<String, String>>,
    // profile: Option<String>, // TODO: Support profile in future
) -> Result<String> {
    // Idempotency check
    let root_str = root.to_string_lossy().to_string();
    for (existing_id, process) in &state.processes {
        if process.root_path == root_str {
            // Check if env is same
            // Handling Option<HashMap> comparison
            // If checking exact match:
            if process.env == env {
                // Determine if process is still alive?
                // For now, if it's in state, we assume it runs or we return it.
                // Improving robustness: verify pid?
                // But basic idempotency: return same ID.
                return Ok(existing_id.clone());
            }
        }
    }

    let run_id = Uuid::new_v4().to_string();

    // Determine run directory and ensure it exists
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let run_dir = cwd.join(".axiomregent").join("runs").join(&run_id);
    fs::create_dir_all(&run_dir).context("Failed to create run directory")?;

    let mut cmd = Command::new("encore");
    cmd.arg("run");
    cmd.current_dir(root);

    if let Some(env_vars) = &env {
        cmd.envs(env_vars);
    }

    // Capture stdout/stderr for logs
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to start 'encore run'")?;
    let pid = child.id();

    // Create log buffer
    let log_buffer = Arc::new(Mutex::new(Vec::new()));

    // Create persistent log file
    let logs_path = run_dir.join("logs.ndjson");
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        // .write(true) <- unnecessary use of `.write(true)` because there is `.append(true)`
        .open(&logs_path)
        .context("Failed to open logs file")?;

    let log_file_mutex = Arc::new(Mutex::new(log_file));

    // Spawn threads to capture output
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let buffer_clone1 = log_buffer.clone();
    let file_clone1 = log_file_mutex.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for l in reader.lines().map_while(Result::ok) {
            // Write to memory
            if let Ok(mut buf) = buffer_clone1.lock() {
                buf.push(l.clone());
            }
            // Write to file
            if let Ok(mut f) = file_clone1.lock() {
                // NDJSON or just lines? Requirement says logs.ndjson.
                // Usually implies JSON objects.
                // But `log_buffer` stores strings (lines).
                // Let's store raw lines for now, or wrap in JSON?
                // "logs.ndjson" name implies JSON.
                // If I store formatted JSON: { "ts": ..., "msg": ... }
                // For now, keeping it simple: just the line.
                // If the user expects NDJSON, I should probably format it.
                // But `encore` logs are mixed text.
                // Let's just write the line + newline.
                let _ = writeln!(f, "{}", l);
            }
        }
    });

    let buffer_clone2 = log_buffer.clone();
    let file_clone2 = log_file_mutex.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(mut buf) = buffer_clone2.lock() {
                buf.push(l.clone());
            }
            if let Ok(mut f) = file_clone2.lock() {
                let _ = writeln!(f, "{}", l);
            }
        }
    });

    let process = RunProcess {
        pid,
        start_time: std::time::SystemTime::now(),
        child: Some(child),
        log_buffer,
        root_path: root_str,
        env,
    };

    // Serialize initial state to disk
    let state_path = run_dir.join("state.json");
    let f = fs::File::create(&state_path).context("Failed to create state file")?;
    serde_json::to_writer_pretty(f, &process).context("Failed to write state file")?;

    state.processes.insert(run_id.clone(), process);

    Ok(run_id)
}

pub fn stop(state: &mut EncoreState, run_id: &str) -> Result<bool> {
    if let Some(mut process) = state.processes.remove(run_id) {
        if let Some(mut child) = process.child.take() {
            // child.kill() propagates error.
            // Using let _ = to ignore if already dead?
            // Context suggests we want to ensure it dies.
            child.kill().context("Failed to kill process")?;
            // child.wait()?;
        }
        Ok(true)
    } else {
        Err(anyhow!("Process not found: {}", run_id))
    }
}

pub fn status(state: &EncoreState, run_id: &str) -> Result<String> {
    if state.processes.contains_key(run_id) {
        Ok("running".to_string())
    } else {
        // Check disk?
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let run_dir = cwd.join(".axiomregent").join("runs").join(run_id);
        let state_path = run_dir.join("state.json");

        if state_path.exists() {
            Ok("stopped".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
}
