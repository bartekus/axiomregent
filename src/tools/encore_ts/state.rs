// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct EncoreState {
    pub processes: HashMap<String, RunProcess>,
}

pub struct RunProcess {
    pub pid: u32,
    pub start_time: SystemTime,
    pub child: Child,
    pub log_buffer: Arc<Mutex<Vec<String>>>, // Simplified log buffer
}

impl Default for EncoreState {
    fn default() -> Self {
        Self::new()
    }
}

impl EncoreState {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }
}
