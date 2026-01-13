// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::io::{self, Read};

mod registry;
mod runner;
mod scanner;
mod skills;
mod state;

use runner::{RunConfig, Runner};
use state::StateStore;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output results in JSON
    #[arg(long)]
    json: bool,

    /// Directory to store run state
    #[arg(long, default_value = ".axiomregent/run")]
    state_dir: String,

    /// Fail if warnings occur
    #[arg(long)]
    fail_on_warning: bool,

    /// Read NULL-delimited file list from stdin
    #[arg(long)]
    files0: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Skill ID to run (if no subcommand)
    #[arg(index = 1)]
    skill_id: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List,
    All,
    Resume,
    Reset,
    Report,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve bin path (executable of current process, or assume "axiomregent" is in PATH?)
    // We want to call the Main Go Binary.
    // If we are shipped INSIDE the distribution, we are `bin/run`.
    // The main binary is `bin/axiomregent`.
    // We should try to find `axiomregent` relative to us, or in PATH.
    // For now, let's assume `axiomregent` is in PATH or we can find it.
    // Spec says: "invoked by the Go axiomregent CLI".
    // If invoked by `axiomregent`, `axiomregent` is in PATH? Not necessarily.
    // But `Os::args()[0]` tells us our path.
    // If we are at `.../bin/run`, `axiomregent` is likely `.../bin/axiomregent`.

    let current_exe = env::current_exe().unwrap_or_else(|_| "run".into());
    let bin_path = if let Some(parent) = current_exe.parent() {
        let sibling = parent.join("axiomregent");
        if sibling.exists() {
            sibling.to_string_lossy().into_owned()
        } else {
            "axiomregent".to_string()
        }
    } else {
        "axiomregent".to_string()
    };

    // Stdin buffer
    let mut stdin_buffer = None;
    if cli.files0 {
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .context("reading stdin")?;
        stdin_buffer = Some(buffer);
    }

    let config = RunConfig {
        json: cli.json,
        state_dir: cli.state_dir.clone(),
        fail_on_warning: cli.fail_on_warning,
        files0: cli.files0,
        bin_path,
        stdin_buffer,
    };

    let store = StateStore::new(&config.state_dir);
    let registry = registry::get_registry();
    let runner = Runner::new(registry, store, config);

    let success = match &cli.command {
        Some(Commands::List) => {
            runner.list();
            true
        }
        Some(Commands::Reset) => {
            // Access store directly or via runner?
            // Runner has private store. Add reset method or expose store?
            // Runner has store.
            // Actually `store` ownership was moved to `runner`.
            // I should add `reset` to Runner.
            // Or create store separately.
            // I'll add `reset` to Runner.
            // "runner.reset()".
            // runner.store is private.
            // Let's add reset method to Runner.
            // Actually I didn't add it in runner.rs.
            // I'll cheat and create a temporary store for reset since it's just filesystem op.
            StateStore::new(&cli.state_dir)
                .reset()
                .context("resetting state")?;
            println!("State cleared.");
            true
        }
        Some(Commands::Report) => {
            // Same, report logic should be in Runner or Store.
            // I didn't add report to Runner.
            // Let's add report logic here using temporary store.
            let store = StateStore::new(&cli.state_dir);
            let last = store.read_last_run()?;
            if cli.json {
                if let Some(l) = last {
                    println!("{}", serde_json::to_string_pretty(&l)?);
                } else {
                    // Empty JSON? null?
                    println!("null");
                }
            } else {
                if let Some(l) = last {
                    println!("Status: {}", l.status);
                    if !l.failed.is_empty() {
                        println!("Failed:");
                        for f in l.failed {
                            println!("  - {}", f);
                        }
                    } else {
                        println!("All passed.");
                    }
                } else {
                    println!("No run state found.");
                }
            }
            true
        }
        Some(Commands::All) => runner.run_all()?,
        Some(Commands::Resume) => runner.resume()?,
        None => {
            if let Some(id) = &cli.skill_id {
                runner.run_specific(&[id.clone()])?
            } else {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                true
            }
        }
    };

    if cli.json {
        let store = StateStore::new(&cli.state_dir);
        if let Ok(Some(last)) = store.read_last_run() {
            println!("{}", serde_json::to_string_pretty(&last)?);
        }
    }

    if !success {
        std::process::exit(1);
    }

    Ok(())
}
