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
        _root: &Path,
        _env: Option<HashMap<String, String>>,
        _profile: Option<String>,
    ) -> Result<Value> {
        Err(anyhow!("Not implemented yet (PR-4)"))
    }

    pub fn run_stop(&self, _run_id: &str) -> Result<Value> {
        Err(anyhow!("Not implemented yet (PR-4)"))
    }

    pub fn logs_stream(&self, _run_id: &str, _from_seq: Option<u64>) -> Result<Value> {
        Err(anyhow!("Not implemented yet (PR-4)"))
    }
}
