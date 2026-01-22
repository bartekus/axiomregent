// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use crate::tools::encore_ts::client::{
    CompileInput, DebugMode, NodeJSRuntime, ParseInput, PrepareInput, TsParserClient,
};
use crate::tools::encore_ts::state::EncoreState;
use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct EncoreTools {
    _state: Arc<Mutex<EncoreState>>,
    parsers: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<TsParserClient>>>>>,
}

impl Default for EncoreTools {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoreTools {
    pub fn new() -> Self {
        Self {
            _state: Arc::new(Mutex::new(EncoreState::new())),
            parsers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn env_check(&self) -> Result<Value> {
        let info = crate::tools::encore_ts::env::check()?;
        Ok(serde_json::to_value(info)?)
    }

    fn ensure_client(&self, root: &Path) -> Result<Arc<Mutex<TsParserClient>>> {
        let root = root.canonicalize().context("Failed to canonicalize root")?;

        {
            let parsers = self.parsers.lock().unwrap();
            if let Some(client) = parsers.get(&root) {
                return Ok(client.clone());
            }
        }

        // Find binary
        let binary_path = if let Ok(p) = which::which("tsparser-encore") {
            p
        } else {
            // Heuristic for dev
            let dev = std::path::Path::new("target/debug/tsparser-encore");
            if dev.exists() {
                dev.to_path_buf()
            } else {
                return Err(anyhow!("tsparser-encore not found"));
            }
        };

        // Create client
        let mut client = TsParserClient::new(&binary_path, &root)?;

        // Prepare
        let prepare_input = PrepareInput {
            app_root: root.clone(),
            runtime_version: "v1.0.0".to_string(), // TODO: Detect from encore.app or similar?
            local_runtime_override: None,
        };
        // We ignore prepare output for now as we just need it to be ready
        let _ = client.prepare(prepare_input)?;

        let client_arc = Arc::new(Mutex::new(client));

        {
            let mut parsers = self.parsers.lock().unwrap();
            parsers.insert(root.clone(), client_arc.clone());
        }

        Ok(client_arc)
    }

    pub fn parse(&self, root: &Path) -> Result<Value> {
        let client_arc = self.ensure_client(root)?;
        let mut client = client_arc.lock().unwrap();

        let input = ParseInput {
            app_root: root.canonicalize()?,
            platform_id: None,
            local_id: "local".to_string(),
            parse_tests: false,
        };

        let meta_bytes = client.parse(input)?;
        // We should decode this using prost if we want to return structured JSON?
        // Or return base64 string?
        // The tool "encore.ts.parse" description says "Parse Encore TS application".
        // `meta.proto` defines the structure.
        // For MCP, returning JSON is better.
        // I need the protobuf definition to decode.
        // `encore-tsparser` crate should expose the protobuf structs?
        // It exposes `encore_tsparser::builder::AppDesc`? No.
        // It returns `result.meta` which is `meta::v1::Data`.
        // I'll check if I can decode it.
        // For now, let's return it as base64-encoded bytes in a JSON wrapper.
        // The client (router caller) can decode if needed, OR we decode here.
        // Given I might not have easy access to the proto struct unless I import `encore-tsparser` generated `meta.rs`, I'll return base64 for now.
        // Wait, `changes/011_parse_meta` implemented mapping.

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&meta_bytes);

        // Cache meta.pb
        let run_dir = Path::new(".axiomregent/run");
        std::fs::create_dir_all(run_dir)?;
        std::fs::write(run_dir.join("encore_meta.pb"), &meta_bytes)?;

        Ok(serde_json::json!({ "meta_pb_base64": encoded }))
    }

    pub fn codegen(&self, root: &Path) -> Result<Value> {
        let client_arc = self.ensure_client(root)?;
        let mut client = client_arc.lock().unwrap();
        client.gen_user_facing()?;
        Ok(serde_json::json!({ "status": "ok" }))
    }

    pub fn compile(&self, root: &Path) -> Result<Value> {
        let client_arc = self.ensure_client(root)?;
        let mut client = client_arc.lock().unwrap();

        let input = CompileInput {
            debug: DebugMode::Full,
            nodejs_runtime: NodeJSRuntime::Node,
        };

        let result = client.compile(input)?;

        // Cache compile result
        let run_dir = Path::new(".axiomregent/run");
        std::fs::create_dir_all(run_dir)?;
        std::fs::write(
            run_dir.join("encore_compile.json"),
            serde_json::to_string_pretty(&result)?,
        )?;

        Ok(result)
    }

    // Proxy for existing meta/run tools if needed, or remove them.
    // Spec says PR-3 adds these. Run logic is PR-4.
    // I will comment out old run methods or keep them if they don't conflict using legacy logic.
    // I'll remove old logic for clarity as we are replacing it.

    // Old run methods (placeholders or legacy)
    pub fn run_start(
        &self,
        root: &Path,
        _env: Option<HashMap<String, String>>,
        _profile: Option<String>,
    ) -> Result<Value> {
        // 1. Compile
        let compile_output_val = self.compile(root)?;
        let compile_res: encore_tsparser::builder::CompileResult =
            serde_json::from_value(compile_output_val)?;

        // 2. Build Supervisor Config
        // Assume first output and combined entrypoint for now
        let output = compile_res.outputs.first().context("No compile outputs")?;
        let entrypoint = output.entrypoints.first().context("No entrypoints")?;

        use encore_supervisor::config::{BinaryConfig, InfraConfig, Proc, SupervisorConfig};

        let run_id = uuid::Uuid::new_v4().to_string();
        let run_dir = PathBuf::from(".axiomregent/runs").join(&run_id);
        std::fs::create_dir_all(&run_dir)?;

        let proc = Proc {
            id: "app".to_string(),
            command: entrypoint.cmd.command.clone(),
            env: entrypoint.cmd.env.clone(),
            services: entrypoint.services.clone(),
            gateways: entrypoint.gateways.clone(),
        };

        let binary_config = BinaryConfig { procs: vec![proc] };

        let supervisor_config = SupervisorConfig {
            binary_config,
            hosted_services: entrypoint.services.clone(),
            hosted_gateways: entrypoint.gateways.clone(),
        };

        let config_path = run_dir.join("supervisor.config.json");
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&supervisor_config)?,
        )?;

        // 3. Infra Config
        let infra_config = InfraConfig {
            hosted_services: entrypoint.services.clone(),
            hosted_gateways: entrypoint.gateways.clone(),
        };
        let infra_path = run_dir.join("infra.config.json");
        std::fs::write(&infra_path, serde_json::to_string_pretty(&infra_config)?)?;

        // 4. Find Supervisor Binary
        let supervisor_bin = if let Ok(p) = which::which("supervisor-encore") {
            p
        } else {
            let dev = Path::new("target/debug/supervisor-encore");
            if dev.exists() {
                dev.to_path_buf()
            } else {
                return Err(anyhow!("supervisor-encore not found"));
            }
        };

        // 5. Spawn
        use std::process::{Command, Stdio};
        let mut child = Command::new(&supervisor_bin)
            .arg("-c")
            .arg(&config_path)
            .env("ENCORE_INFRA_CONFIG_PATH", &infra_path)
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 6. Capture Logs
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let log_buffer = Arc::new(Mutex::new(Vec::new()));
        let lb1 = log_buffer.clone();
        let lb2 = log_buffer.clone();

        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for l in reader.lines().map_while(Result::ok) {
                let _ = lb1.lock().map(|mut b| b.push(l));
            }
        });
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for l in reader.lines().map_while(Result::ok) {
                let _ = lb2.lock().map(|mut b| b.push(l));
            }
        });

        // 7. Store State
        {
            let mut state = self._state.lock().unwrap();
            state.processes.insert(
                run_id.clone(),
                crate::tools::encore_ts::state::RunProcess {
                    pid: child.id(),
                    start_time: std::time::SystemTime::now(),
                    child: Some(child),
                    log_buffer,
                    root_path: root.to_string_lossy().to_string(),
                    env: _env,
                },
            );
        }

        Ok(serde_json::json!({ "run_id": run_id }))
    }

    pub fn run_stop(&self, run_id: &str) -> Result<Value> {
        let mut state = self._state.lock().unwrap();
        if let Some(mut proc) = state.processes.remove(run_id) {
            if let Some(mut child) = proc.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            Ok(serde_json::json!({ "status": "stopped" }))
        } else {
            Err(anyhow!("Run ID not found"))
        }
    }

    pub fn logs_stream(&self, run_id: &str, from_seq: Option<u64>) -> Result<Value> {
        let state = self._state.lock().unwrap();
        if let Some(proc) = state.processes.get(run_id) {
            let buffer = proc.log_buffer.lock().unwrap();
            let start = from_seq.unwrap_or(0) as usize;
            let logs = if start < buffer.len() {
                buffer[start..].to_vec()
            } else {
                Vec::new()
            };
            Ok(serde_json::json!({ "logs": logs }))
        } else {
            Err(anyhow!("Run ID not found"))
        }
    }
}
