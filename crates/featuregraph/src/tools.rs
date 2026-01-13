// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: FEATUREGRAPH_REGISTRY
// Spec: spec/core/featuregraph.md

use crate::scanner::Scanner;
use anyhow::{anyhow, Result};
use std::path::Path;

pub struct FeatureGraphTools {
    // Cache placeholder
}

impl Default for FeatureGraphTools {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureGraphTools {
    pub fn new() -> Self {
        Self {}
    }

    pub fn features_overview(
        &self,
        repo_root: &Path,
        _snapshot_id: Option<String>,
    ) -> Result<serde_json::Value> {
        // Only Worktree supported for now
        let scanner = Scanner::new(repo_root);
        let graph = scanner.scan()?;
        let json = serde_json::to_value(graph)?;
        Ok(json)
    }

    pub fn features_locate(
        &self,
        repo_root: &Path,
        feature_id: Option<String>,
        spec_path: Option<String>,
        file_path: Option<String>,
    ) -> Result<serde_json::Value> {
        let scanner = Scanner::new(repo_root);
        let graph = scanner.scan()?;

        if let Some(fid) = feature_id {
            if let Some(node) = graph.features.iter().find(|f| f.feature_id == fid) {
                return Ok(serde_json::to_value(node)?);
            }
            return Err(anyhow!("Feature ID not found: {}", fid));
        }

        if let Some(spath) = spec_path {
            if let Some(node) = graph.features.iter().find(|f| f.spec_path == spath) {
                return Ok(serde_json::to_value(node)?);
            }
            return Err(anyhow!("Spec path not found: {}", spath));
        }

        if let Some(fpath) = file_path {
            // Check impl files
            if let Some(node) = graph
                .features
                .iter()
                .find(|f| f.impl_files.contains(&fpath))
            {
                return Ok(serde_json::to_value(node)?);
            }
            // Check test files
            if let Some(node) = graph
                .features
                .iter()
                .find(|f| f.test_files.contains(&fpath))
            {
                return Ok(serde_json::to_value(node)?);
            }
            return Err(anyhow!("File not owned by any feature: {}", fpath));
        }

        Err(anyhow!("Must provide feature_id, spec_path, or file_path"))
    }

    pub fn governance_preflight(
        &self,
        repo_root: &Path,
        request: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let req: crate::preflight::PreflightRequest = serde_json::from_value(request)?;
        let scanner = Scanner::new(repo_root);
        let graph = scanner.scan()?;

        let checker = crate::preflight::PreflightChecker::new(repo_root);
        let response = checker.check(&graph, &req)?;

        let json = serde_json::to_value(response)?;
        Ok(json)
    }

    pub fn governance_drift(&self, repo_root: &Path) -> Result<serde_json::Value> {
        // For now, checks violation list from a fresh scan
        let scanner = Scanner::new(repo_root);
        let graph = scanner.scan()?;
        let json = serde_json::to_value(graph.violations)?;
        Ok(json)
    }
}
