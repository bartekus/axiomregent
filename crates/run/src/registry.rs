// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

use crate::runner::{LegacySkill, Skill};
use crate::skills::lint_gofumpt::LintGofumpt;
use crate::skills::test_build::TestBuild;
use crate::skills::test_go::TestGo;

pub fn get_registry() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(LegacySkill::new("format:gofumpt")),
        Box::new(LintGofumpt),
        Box::new(LegacySkill::new("lint:golangci")),
        Box::new(TestBuild),
        Box::new(LegacySkill::new("test:binary")),
        Box::new(TestGo),
        Box::new(LegacySkill::new("test:coverage")),
        Box::new(LegacySkill::new("docs:yaml")),
        Box::new(LegacySkill::new("docs:validate-spec")),
        Box::new(LegacySkill::new("docs:spec-reference-check")),
        Box::new(LegacySkill::new("docs:orphan-specs")),
        Box::new(LegacySkill::new("docs:orphan-docs")),
        Box::new(LegacySkill::new("docs:doc-patterns")),
        Box::new(LegacySkill::new("docs:required-tests")),
        Box::new(LegacySkill::new("docs:header-comments")),
        Box::new(LegacySkill::new("purity")),
        Box::new(LegacySkill::new("docs:policy")),
        Box::new(LegacySkill::new("docs:provider-governance")),
    ]
}
