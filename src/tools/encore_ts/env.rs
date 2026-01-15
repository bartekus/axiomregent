// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvInfo {
    pub deployed: bool,
    pub version: String,
}

pub fn check() -> Result<EnvInfo> {
    let output = Command::new("encore").arg("version").output().context(
        "Failed to execute 'encore version'. Ensure encore CLI is installed and in PATH.",
    )?;

    if !output.status.success() {
        return Ok(EnvInfo {
            deployed: false,
            version: String::new(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // output format example: "encore v1.35.0 (2024-03-22T15:27:06Z) macos-arm64"
    // or just "v1.35.0"
    let version = stdout
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_string();

    Ok(EnvInfo {
        deployed: true,
        version,
    })
}
