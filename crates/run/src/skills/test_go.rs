// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use crate::runner::{RunConfig, Skill};
use crate::state::{SkillResult, SkillStatus};
use anyhow::{Context, Result};
use std::process::Command;

pub struct TestGo;

impl Skill for TestGo {
    fn id(&self) -> &str {
        "test:go"
    }

    fn run(&self, _config: &RunConfig) -> Result<SkillResult> {
        // Mirrors Go's ExecSkill behavior for "test:go"
        // args: []string{"go", "test", "./..."}

        let output = Command::new("go")
            .arg("test")
            .arg("./...")
            .output()
            .context("running go test")?;

        if output.status.success() {
            return Ok(SkillResult {
                skill: self.id().to_string(),
                status: SkillStatus::Pass,
                exit_code: 0,
                note: None,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);
        let lines: Vec<&str> = combined.lines().collect();

        // Capture last 20 lines for note
        let note = if lines.len() > 20 {
            let truncated = &lines[lines.len() - 20..];
            format!("...(truncated)...\n{}", truncated.join("\n"))
        } else {
            combined
        };

        Ok(SkillResult {
            skill: self.id().to_string(),
            status: SkillStatus::Fail,
            exit_code: output.status.code().unwrap_or(1),
            note: Some(note.trim().to_string()),
        })
    }
}
