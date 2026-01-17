// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug, Default)]
pub struct EncoreState {
    pub processes: HashMap<String, RunProcess>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunProcess {
    pub pid: u32,
    pub start_time: SystemTime,
    #[serde(skip)]
    pub child: Option<std::process::Child>, // Child is not serializable
    #[serde(skip)]
    pub log_buffer: Arc<Mutex<Vec<String>>>, // Not serializable, re-create or ignore on load
    pub root_path: String, // Path to the app root
    pub env: Option<HashMap<String, String>>,
}

impl EncoreState {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }
}
