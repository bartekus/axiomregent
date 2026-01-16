use crate::tools::encore_ts::state::{EncoreState, RunProcess};
use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
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

    // Kill existing process for this root if strict 1-per-workspace policy?
    // For now, let's just spawn.

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

    state.processes.insert(
        run_id.clone(),
        RunProcess {
            pid,
            start_time: std::time::SystemTime::now(),
            child,
            log_buffer,
        },
    );

    Ok(run_id)
}

pub fn stop(state: &mut EncoreState, run_id: &str) -> Result<bool> {
    if let Some(mut process) = state.processes.remove(run_id) {
        process.child.kill().context("Failed to kill process")?;
        // process.child.wait()?; // Optional, prevent zombies?
        Ok(true)
    } else {
        Err(anyhow!("Process not found: {}", run_id))
    }
}

pub fn status(_state: &EncoreState, _run_id: &str) -> Result<String> {
    // Implementation needed if we want status check
    Ok("unknown".to_string())
}
