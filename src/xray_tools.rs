// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: XRAY_ANALYSIS
// Spec: spec/xray/analysis.md

use anyhow::Result;
// use serde_json::json; // Unused
use std::path::Path;
// use std::path::PathBuf; // Unused
use xray::XrayIndex;

pub struct XrayTools;

impl Default for XrayTools {
    fn default() -> Self {
        Self::new()
    }
}

impl XrayTools {
    pub fn new() -> Self {
        Self
    }

    pub fn scan(&self, target: &Path) -> Result<XrayIndex> {
        // We do NOT want to write the index to disk when called from MCP,
        // usually the agent just wants the data.
        // So we pass None for output path.
        xray::scan_target(target, None)
    }
}
