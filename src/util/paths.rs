// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: MCP_ROUTER
// Spec: spec/core/router.md

use std::path::{Component, Path};

pub fn normalize_path(path: &Path) -> String {
    let mut s = path.to_string_lossy().replace("\\", "/");
    if s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    s
}

pub fn path_depth(path: &Path) -> usize {
    path.components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count()
}
