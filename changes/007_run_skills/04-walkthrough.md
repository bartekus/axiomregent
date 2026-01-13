# Walkthrough - PR 007: Associated Run Skills

## Changes
### Run Crate
- Implemented `test:go` in `crates/run/src/skills/test_go.rs`.
- Implemented `test:build` in `crates/run/src/skills/test_build.rs`.
- Implemented `lint:gofumpt` in `crates/run/src/skills/lint_gofumpt.rs`.
- Added unit tests for skill registration in `crates/run/src/skills/mod.rs`.

## Verification Results
### Unit Tests
`cargo test -p run` passed:
```
running 1 test
test skills::tests::test_skill_ids ... ok
```

### Manual Verification
`cargo run -p run -- list` successfully lists the new skills:
```
...
lint:gofumpt
...
test:build
...
test:go
...
```
