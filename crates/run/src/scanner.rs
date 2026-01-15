// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct Scanner {
    root: PathBuf,
}

impl Scanner {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn scan_extensions(&self, exts: &[&str]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(&self.root)
            .hidden(false) // Don't ignore hidden files (like .github) unless gitignored
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file()) {
                        let path = entry.path();
                        let is_match = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|ext| exts.contains(&ext));

                        if is_match {
                            // Return relative path if possible, or absolute?
                            // Go scanner returns relative to root usually.
                            // Let's return relative to root for consistency with Go.
                            if let Ok(rel) = path.strip_prefix(&self.root) {
                                files.push(rel.to_path_buf());
                            } else {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
                Err(err) => eprintln!("Scanner warning: {}", err),
            }
        }
        Ok(files)
    }
}
