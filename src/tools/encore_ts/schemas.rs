// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaSnapshotV1 {
    pub services: Vec<ServiceInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub description: Option<String>,
    pub apis: Vec<ApiInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiInfo {
    pub name: String,
    pub path: String,
    pub method: String,
    pub access: String, // "public", "private", "auth"
}
