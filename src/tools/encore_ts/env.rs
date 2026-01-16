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
    let output = match Command::new("encore").arg("version").output() {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(EnvInfo {
                deployed: false,
                version: String::new(),
            });
        }
        Err(e) => return Err(e).context("Failed to execute 'encore version'")?,
    };

    if !output.status.success() {
        return Ok(EnvInfo {
            deployed: false,
            version: String::new(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // output format example: "encore v1.35.0 (2024-03-22T15:27:06Z) macos-arm64"
    // or just "v1.35.0"
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    let version = if parts.first() == Some(&"encore") && parts.len() > 1 {
        parts[1].to_string()
    } else {
        parts.first().unwrap_or(&"unknown").to_string()
    };

    // Also check for node
    if Command::new("node").arg("--version").output().is_err() {
        return Ok(EnvInfo {
            deployed: false,
            version: "Missing Node.js".to_string(), // Or handle differently
        });
    }

    Ok(EnvInfo {
        deployed: true,
        version,
    })
}
