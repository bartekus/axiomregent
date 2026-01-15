// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: AXIOMREGENT_RUN_SKILLS
// Spec: spec/run/skills.md

pub mod lint_gofumpt;
pub mod test_build;
pub mod test_go;

#[cfg(test)]
mod tests {
    use crate::runner::Skill;
    use crate::skills::lint_gofumpt::LintGofumpt;
    use crate::skills::test_build::TestBuild;
    use crate::skills::test_go::TestGo;

    #[test]
    fn test_skill_ids() {
        assert_eq!(LintGofumpt.id(), "lint:gofumpt");
        assert_eq!(TestBuild.id(), "test:build");
        assert_eq!(TestGo.id(), "test:go");
    }
}
