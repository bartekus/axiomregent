// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: ANTIGRAVITY_AUTOMATION
// Spec: spec/antigravity/automation.md

use crate::feature_tools::{FeatureTools, PreflightMode, PreflightRequest};
use crate::snapshot::tools::SnapshotTools;
use crate::workspace::WorkspaceTools;
use antigravity::validator::McpClient;
use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::sync::Arc;

pub struct InternalClient {
    pub repo_root: PathBuf,
    pub workspace: Arc<WorkspaceTools>,
    pub snapshot: Arc<SnapshotTools>,
    pub features: Arc<FeatureTools>,
}

impl McpClient for InternalClient {
    fn preflight(&self, mode: &str) -> Result<bool> {
        let mode_enum = match mode {
            "standard" => PreflightMode::Worktree,
            _ => PreflightMode::Worktree,
        };
        let req = PreflightRequest {
            mode: mode_enum,
            snapshot_id: None,
            intent: crate::feature_tools::PreflightIntent::Edit,
            changed_paths: vec![],
        };

        let response = self.features.preflight(&self.repo_root, req)?;
        Ok(response.allowed)
    }

    fn drift(&self, _mode: &str) -> Result<bool> {
        // mode "check" usually
        let violations = self.features.drift(&self.repo_root, None)?;
        Ok(!violations.is_empty())
    }

    fn impact(&self, _mode: &str, changed_paths: Vec<String>) -> Result<String> {
        let impacted = self.features.impact(&self.repo_root, changed_paths, None)?;
        if impacted.is_empty() {
            Ok("none".to_string())
        } else {
            Ok("high".to_string())
        }
    }

    fn call_tool(&self, name: &str, args: &serde_json::Value) -> Result<serde_json::Value> {
        match name {
            "write_file" | "workspace.write_file" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing path"))?;
                let content_base64 = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing content"))?;

                let lease_id = args
                    .get("lease_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let create_dirs = args
                    .get("create_dirs")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let dry_run = args
                    .get("dry_run")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                self.workspace.write_file(
                    &self.repo_root,
                    path,
                    content_base64,
                    lease_id,
                    create_dirs,
                    dry_run,
                )?;
                Ok(serde_json::json!({"status": "success"}))
            }
            "snapshot.create" => {
                let lease_id = args
                    .get("lease_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let paths = args.get("paths").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect()
                });

                self.snapshot
                    .snapshot_create(&self.repo_root, lease_id, paths)
            }
            "workspace.apply_patch" => {
                let patch = args
                    .get("patch")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing patch"))?;
                let mode = args
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("worktree");
                let lease_id = args
                    .get("lease_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                self.workspace.apply_patch(
                    &self.repo_root,
                    patch,
                    mode,
                    lease_id,
                    None,
                    None,
                    false,
                    false,
                )
            }
            "gov.preflight" | "gov.drift" | "features.impact" => {
                // Return success stub for now if called directly?
                // Or implement using self methods?
                // Executor calls client.drift() explicitly for checks, so call_tool("gov.drift") is rare but possible.
                Ok(serde_json::json!({"status": "skipped_meta_tool"}))
            }
            _ => Err(anyhow!("Tool {} not found", name)),
        }
    }
}
