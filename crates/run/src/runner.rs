// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use crate::state::{LastRun, SkillResult, SkillStatus, StateStore};
use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub trait Skill {
    fn id(&self) -> &str;
    fn run(&self, config: &RunConfig) -> Result<SkillResult>;
}

pub struct RunConfig {
    pub json: bool,
    pub state_dir: String,
    pub fail_on_warning: bool,
    pub files0: bool,
    pub bin_path: String,
    pub stdin_buffer: Option<Vec<u8>>,
}

pub struct Runner {
    registry: Vec<Box<dyn Skill>>,
    store: StateStore,
    config: RunConfig,
}

impl Runner {
    pub fn new(registry: Vec<Box<dyn Skill>>, store: StateStore, config: RunConfig) -> Self {
        Self {
            registry,
            store,
            config,
        }
    }

    pub fn list(&self) {
        if self.config.json {
            // TODO: Output JSON
            let ids: Vec<&str> = self.registry.iter().map(|s| s.id()).collect();
            println!("{{ \"skills\": {:?} }}", ids);
        } else {
            for skill in &self.registry {
                println!("{}", skill.id());
            }
        }
    }

    pub fn run_all(&self) -> Result<bool> {
        self.execute_sequence(self.registry.iter().map(|s| s.as_ref()).collect())
    }

    pub fn run_specific(&self, skill_ids: &[String]) -> Result<bool> {
        let mut to_run = Vec::new();
        for id in skill_ids {
            if let Some(s) = self.registry.iter().find(|s| s.id() == id) {
                to_run.push(s.as_ref());
            } else {
                eprintln!("Skill not found: {}", id);
                return Ok(false);
            }
        }
        self.execute_sequence(to_run)
    }

    pub fn resume(&self) -> Result<bool> {
        let last_run = self.store.read_last_run()?;
        if let Some(last) = last_run {
            if last.failed.is_empty() {
                println!("No failed skills to resume.");
                return Ok(true);
            }
            let mut to_run = Vec::new();
            for id in &last.failed {
                if let Some(s) = self.registry.iter().find(|s| s.id() == id) {
                    to_run.push(s.as_ref());
                }
            }
            self.execute_sequence(to_run)
        } else {
            println!("No run state found.");
            Ok(true)
        }
    }

    fn execute_sequence(&self, skills: Vec<&dyn Skill>) -> Result<bool> {
        let mut failed = Vec::new();
        let mut skill_names = Vec::new();
        let mut overall_success = true;

        for skill in skills {
            let id = skill.id();
            skill_names.push(id.to_string());

            if !self.config.json {
                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("SKILL: {}", id);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            }

            let res = skill.run(&self.config)?;

            // Persist
            self.store.write_skill_result(&res)?;

            if res.status == SkillStatus::Skip {
                if !self.config.json {
                    println!("SKIP: {}", id);
                    if let Some(note) = &res.note {
                        println!("{}", note);
                    }
                }
                continue;
            }

            if res.status != SkillStatus::Pass {
                failed.push(id.to_string());
                overall_success = false;
                if !self.config.json {
                    println!("FAIL: {} (exit {})", id, res.exit_code);
                    if let Some(note) = &res.note {
                        println!("{}", note);
                    }
                }
            } else {
                if !self.config.json {
                    println!("PASS: {}", id);
                    if let Some(note) = &res.note {
                        println!("{}", note);
                    }
                }
            }
        }

        let last_run = LastRun {
            status: if overall_success {
                "pass".to_string()
            } else {
                "fail".to_string()
            },
            skills: skill_names,
            failed: failed.clone(),
        };
        self.store.write_last_run(&last_run)?;

        if !overall_success {
            eprintln!("Run failed: {:?}", failed);
        }

        Ok(overall_success)
    }
}

pub struct LegacySkill {
    id: String,
}

impl LegacySkill {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl Skill for LegacySkill {
    fn id(&self) -> &str {
        &self.id
    }

    fn run(&self, config: &RunConfig) -> Result<SkillResult> {
        let mut cmd = Command::new(&config.bin_path);
        cmd.arg("internal")
            .arg("run-skill")
            .arg(&self.id)
            .arg("--json"); // Always use JSON for bridge communication

        // Configure command logic for the legacy bridge.
        if config.fail_on_warning {
            cmd.arg("--fail-on-warning");
        }

        if config.files0 {
            cmd.arg("--files0");
            cmd.stdin(Stdio::piped());
        }

        cmd.arg("--state-dir").arg(&config.state_dir);

        // Capture stdout to read the JSON result from the bridge.
        // Inherit stderr so that logs and progress updates from the legacy runner are visible to the user.
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = cmd.spawn().context("spawning legacy skill")?;

        if config.files0 {
            if let Some(stdin_buf) = &config.stdin_buffer {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin
                        .write_all(stdin_buf)
                        .context("writing to child stdin")?;
                }
            }
        }

        let output = child
            .wait_with_output()
            .context("waiting for legacy skill")?;

        // Parse stdout as SkillResult
        if output.status.success() || output.status.code().unwrap_or(1) != 0 {
            if let Ok(res) = serde_json::from_slice::<SkillResult>(&output.stdout) {
                return Ok(res);
            }
        }

        // If parsing failed or process crashed
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout); // Might contain non-json garbage if panic

        Ok(SkillResult {
            skill: self.id.clone(),
            status: SkillStatus::Fail,
            exit_code: output.status.code().unwrap_or(1),
            note: Some(format!(
                "Bridge error. Stdout: '{}'. Stderr: '{}'",
                stdout.trim(),
                stderr.trim()
            )),
        })
    }
}
