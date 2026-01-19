// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: MCP_ROUTER
// Spec: spec/core/router.md

use anyhow::{Result, anyhow};
use axiomregent::io::fs::RealFs;
use axiomregent::resolver::order::ResolveEngine;
use axiomregent::router::{JsonRpcRequest, Router};
use env_logger::Target;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

// POLICY: stdout is RESERVED for protocol messages.
// All logs, panics, and diagnostics MUST write to stderr.
#[tokio::main]
async fn main() -> Result<()> {
    // 0. Setup Logging & Panic Safety
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(Target::Stderr)
        .format_timestamp(None) // Stable tests
        .init();

    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic: {}", info);
    }));

    log::info!("mcp starting (stdio - MCP framed JSON-RPC)");

    // 1. Setup Resolver
    let dirs = match std::env::var("AXIOMREGENT_WORKSPACE_ROOTS") {
        Ok(val) => val.split(':').map(PathBuf::from).collect(),
        Err(_) => {
            // Default roots
            let mut roots = Vec::new();
            if let Some(home) = dirs::home_dir() {
                roots.push(home.join("Dev"));
                roots.push(home.join("src"));
            }
            roots
        }
    };

    let run_root = dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    let fs = RealFs;
    let resolver = Arc::new(ResolveEngine::new(fs, dirs));

    // 2. Setup MountRegistry
    let mounts = axiomregent::router::mounts::MountRegistry::new();

    // 3. Setup Stores & Tools
    let storage_config = axiomregent::config::StorageConfig::default();
    let store = Arc::new(axiomregent::snapshot::store::Store::new(storage_config)?);
    let lease_store = Arc::new(axiomregent::snapshot::lease::LeaseStore::new());

    let snapshot_tools = Arc::new(axiomregent::snapshot::tools::SnapshotTools::new(
        lease_store.clone(),
        store.clone(),
    ));
    let workspace_tools = Arc::new(axiomregent::workspace::WorkspaceTools::new(
        lease_store.clone(),
        store.clone(),
    ));
    let featuregraph_tools = Arc::new(axiomregent::featuregraph::tools::FeatureGraphTools::new());
    let feature_tools = Arc::new(axiomregent::feature_tools::FeatureTools::new());
    let xray_tools = Arc::new(axiomregent::xray::tools::XrayTools::new());
    let antigravity_tools = Arc::new(axiomregent::antigravity_tools::AntigravityTools::new(
        workspace_tools.clone(),
        snapshot_tools.clone(),
        feature_tools.clone(),
    ));
    let encore_tools = Arc::new(axiomregent::tools::encore_ts::tools::EncoreTools::new());
    let run_tools = Arc::new(axiomregent::run_tools::RunTools::new(&run_root));

    // 3.5 Setup Supervisor
    let log_buffer = Arc::new(axiomregent::supervisor::buffer::LogBuffer::new(1000));
    let (supervisor_handle, command_rx) =
        axiomregent::supervisor::SupervisorHandle::new(log_buffer.clone());

    let lock_path = run_root.join(".axiomregent/encore.lock");
    // SAFETY: If we can't lock, we assume another instance matches.
    // For now we just log and don't spawn the managed supervisor loop.
    let _lock = match axiomregent::supervisor::lock::FileLock::try_acquire(lock_path.clone()) {
        Ok(lock) => {
            log::info!("Acquired supervisor lock. Running in Managed mode.");

            let cwd = run_root.clone();
            let cmd = std::env::var("AXIOM_ENCORE_CMD").unwrap_or_else(|_| "encore".to_string());
            let mut args = vec![
                "run".to_string(),
                "--tag".to_string(),
                "axiomregent".to_string(),
            ];
            if let Ok(extra) = std::env::var("AXIOM_ENCORE_ARGS") {
                args.extend(extra.split_whitespace().map(String::from));
            }

            // Default probe: HTTP /
            let http_check = Arc::new(axiomregent::readiness::HttpCheck {
                path: "/".to_string(),
            });
            let probe = axiomregent::readiness::HealthProbe::new(vec![http_check]);

            let supervisor = axiomregent::supervisor::Supervisor {
                cmd,
                args,
                cwd,
                env: std::env::vars().collect(),
                health_probe: Some(probe),
                log_buffer: log_buffer.clone(),
                state: supervisor_handle.state.clone(),
                command_rx,
            };

            let token = tokio_util::sync::CancellationToken::new();
            tokio::spawn(async move {
                if let Err(e) = supervisor.run(token).await {
                    log::error!("Supervisor loop exited with error: {}", e);
                }
            });

            Some(lock)
        }
        Err(e) => {
            log::warn!(
                "Failed to acquire lock: {}. Running in Observer/External mode.",
                e
            );
            None
        }
    };

    let supervisor_tools = Arc::new(axiomregent::supervisor::tools::SupervisorTools::new(
        supervisor_handle,
    ));

    // 4. Setup Router
    let router = Router::new(
        resolver,
        mounts,
        snapshot_tools,
        workspace_tools,
        featuregraph_tools,
        xray_tools,
        antigravity_tools,
        encore_tools,
        run_tools,
        supervisor_tools,
    );

    // 4. Stdio Loop (MCP framing)
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        let maybe_payload = read_mcp_message(&mut input)?;
        let Some(payload) = maybe_payload else {
            break; // EOF
        };

        match serde_json::from_str::<JsonRpcRequest>(&payload) {
            Ok(req) => {
                let response = router.handle_request(&req);
                let resp_str = serde_json::to_string(&response)?;
                write_mcp_message(&mut stdout, resp_str.as_bytes())?;
            }
            Err(e) => {
                // IMPORTANT: Some clients will send other traffic; log but don't crash.
                log::error!("Failed to parse JSON-RPC payload: {}", e);
            }
        }
    }

    Ok(())
}

/// Reads a single MCP stdio framed message.
///
/// MCP clients typically speak:
///   Content-Length: <n>\r\n
///   \r\n
///   <n bytes of JSON>
///
/// For local diagnostics, we also accept a single-line JSON payload (line-delimited)
/// **IF AND ONLY IF** `MCP_ALLOW_LINE_JSON` is set.
fn read_mcp_message<R: BufRead + Read>(r: &mut R) -> Result<Option<String>> {
    let mut first_line = String::new();

    // Read until we find a non-empty line or EOF.
    loop {
        first_line.clear();
        let n = r.read_line(&mut first_line)?;
        if n == 0 {
            return Ok(None);
        }
        if !first_line.trim().is_empty() {
            break;
        }
    }

    let trimmed = first_line.trim_end_matches(['\r', '\n']);

    // Line-delimited JSON fallback for dev/testing.
    if trimmed.starts_with('{') && std::env::var("MCP_ALLOW_LINE_JSON").is_ok() {
        return Ok(Some(trimmed.to_string()));
    }

    // Otherwise, treat it as the start of headers.
    let mut content_length: Option<usize> = None;
    parse_header_line(trimmed, &mut content_length)?;

    // Read remaining headers until blank line.
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line)?;
        if n == 0 {
            return Err(anyhow!("EOF while reading MCP headers"));
        }

        let l = line.trim_end_matches(['\r', '\n']);
        if l.is_empty() {
            break;
        }

        parse_header_line(l, &mut content_length)?;
    }

    let len = content_length.ok_or_else(|| anyhow!("Missing Content-Length header"))?;

    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;

    let s = String::from_utf8(buf)?;
    Ok(Some(s))
}

fn parse_header_line(line: &str, content_length: &mut Option<usize>) -> Result<()> {
    // Keep this deterministic and strict.
    // We accept both `Content-Length:` and `content-length:`.
    let lower = line.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("content-length:") {
        let v = rest.trim();
        if let Ok(n) = v.parse::<usize>() {
            *content_length = Some(n);
        } else {
            return Err(anyhow!("Invalid Content-Length value: {}", v));
        }
    }
    Ok(())
}

fn write_mcp_message<W: Write>(w: &mut W, payload: &[u8]) -> Result<()> {
    // MCP stdio framing
    write!(w, "Content-Length: {}\r\n\r\n", payload.len())?;
    w.write_all(payload)?;
    w.flush()?;
    Ok(())
}
