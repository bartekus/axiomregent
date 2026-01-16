// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use crate::tools::encore_ts::state::{EncoreState, RunProcess};
use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
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
    let run_id = Uuid::new_v4().to_string();

    // Determine run directory and ensure it exists
    // Default to .axiomregent/runs/<run_id>/
    // We need to find the workspace root or just use relative if we are in a consistent CWD?
    // Using CWD + .axiomregent for now as per config defaults.
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let run_dir = cwd.join(".axiomregent").join("runs").join(&run_id);
    fs::create_dir_all(&run_dir).context("Failed to create run directory")?;

    let mut cmd = Command::new("encore");
    cmd.arg("run");
    cmd.current_dir(root);

    if let Some(env_vars) = env {
        cmd.envs(env_vars);
    }

    // Capture stdout/stderr for logs
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to start 'encore run'")?;
    let pid = child.id();

    // Create log buffer
    let log_buffer = Arc::new(Mutex::new(Vec::new()));

    // Spawn threads to capture output
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let buffer_clone1 = log_buffer.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(mut buf) = buffer_clone1.lock() {
                buf.push(l);
            }
        }
    });

    let buffer_clone2 = log_buffer.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(mut buf) = buffer_clone2.lock() {
                buf.push(l);
            }
        }
    });

    let process = RunProcess {
        pid,
        start_time: std::time::SystemTime::now(),
        child: Some(child),
        log_buffer,
        root_path: root.to_string_lossy().to_string(),
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
            child.kill().context("Failed to kill process")?;
            // child.wait()?; // Optional, prevent zombies?
        }

        // Update state on disk to reflect stopped status (remove file or update?)
        // Requirement was to persist state. If stopped, maybe we just leave it as is or update a status field?
        // RunProcess struct doesn't have a status field yet.
        // For now, let's keep the file but maybe add a "stopped" marker if we had one.
        // Or strictly strictly, we just leave it.
        // Better: ensure we don't leak the process handle.

        // Clean up or update the state file?
        // Let's check where the file is.
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let run_dir = cwd.join(".axiomregent").join("runs").join(run_id);
        let _state_path = run_dir.join("state.json");

        // We could delete the state file to indicate it's not running?
        // Or keep it for history.
        // Let's keep it but ideally we'd mark it as finished.
        // Since RunProcess has `pid`, checking if it exists is one way.
        // But for now, just succeeding killing is enough.

        Ok(true)
    } else {
        // Fallback: Check if there is a state file and if the process is actually running?
        // For now, if it's not in memory, we assume it's not managed by us or already stopped.
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
            // It existed at some point. Is it running?
            // Without a pidfile check or similar, hard to say if we crashed and lost memory state.
            // Assumption: if not in memory, it's "stopped" or "unknown" for this session.
            Ok("stopped".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
}
