// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use crate::runner::{RunConfig, Skill};
use crate::scanner::Scanner;
use crate::state::{SkillResult, SkillStatus};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use which::which;

pub struct LintGofumpt;

impl Skill for LintGofumpt {
    fn id(&self) -> &str {
        "lint:gofumpt"
    }

    fn run(&self, config: &RunConfig) -> Result<SkillResult> {
        // 1. Determine files
        // If config.files0 is set, we use stdin_buffer (passed via RunConfig? We need to expose it or parse it in Runner)
        // Runner handles files0 for Legacy, but for Native?
        // We need `config.target_files` in RunConfig.
        // Let's modify RunConfig to include target_files if parsed.
        // For now, assume full scan if files0 is false.

        let root = Path::new("."); // Cwd is repo root usually? 
        // We should validte Cwd. `main.rs` doesn't enforce Cwd.
        // Go implementation does `projectroot.Find(wd)`.
        // We should probably do the same or assume Cwd.
        // Let's assume Cwd for now.

        let files = if config.files0 {
            // We need to parse stdin_buffer from config
            // But config.stdin_buffer is raw bytes.
            if let Some(buf) = &config.stdin_buffer {
                let s = String::from_utf8_lossy(buf);
                s.split('\0')
                    .filter(|s| !s.is_empty() && s.ends_with(".go"))
                    .map(|s| s.to_string())
                    .collect()
            } else {
                return Ok(SkillResult {
                    skill: self.id().to_string(),
                    status: SkillStatus::Fail,
                    exit_code: 1,
                    note: Some("files0 requested but no input".to_string()),
                });
            }
        } else {
            let scanner = Scanner::new(root);
            scanner
                .scan_extensions(&["go"])
                .context("scanning go files")?
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };

        if files.is_empty() {
            return Ok(SkillResult {
                skill: self.id().to_string(),
                status: SkillStatus::Pass,
                exit_code: 0,
                note: Some("No Go files to check".to_string()),
            });
        }

        // 2. Check gofumpt existence
        if which("gofumpt").is_err() {
            return Ok(SkillResult {
                skill: self.id().to_string(),
                status: SkillStatus::Fail,
                exit_code: 2,
                note: Some(
                    "gofumpt not found. Run: go install mvdan.cc/gofumpt@v0.6.0".to_string(),
                ),
            });
        }

        // 3. Run gofumpt -l
        // Chunking? Go impl chunks by 200.
        // We can check ARG_MAX but 200 is safe.
        let chunk_size = 200;
        let mut unformatted = Vec::new();

        for chunk in files.chunks(chunk_size) {
            let output = Command::new("gofumpt")
                .arg("-l")
                .args(chunk)
                .output()
                .context("running gofumpt")?;

            if !output.status.success() {
                return Ok(SkillResult {
                    skill: self.id().to_string(),
                    status: SkillStatus::Fail,
                    exit_code: output.status.code().unwrap_or(1),
                    note: Some(format!(
                        "gofumpt failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )),
                });
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    unformatted.push(trimmed.to_string());
                }
            }
        }

        if !unformatted.is_empty() {
            unformatted.sort();
            unformatted.dedup();

            let mut note = String::from("Unformatted files:\n");
            for f in &unformatted {
                note.push_str(f);
                note.push('\n');
            }
            note.push_str("\nTo fix, run:\n  gofumpt -w");
            // Add file list to fix cmd?
            // "gofumpt -w " + list
            // Just hint.
            if unformatted.len() < 20 {
                note.push(' ');
                note.push_str(&unformatted.join(" "));
            } else {
                note.push_str(" .");
            }

            return Ok(SkillResult {
                skill: self.id().to_string(),
                status: SkillStatus::Fail,
                exit_code: 3,
                note: Some(note),
            });
        }

        Ok(SkillResult {
            skill: self.id().to_string(),
            status: SkillStatus::Pass,
            exit_code: 0,
            note: None,
        })
    }
}
