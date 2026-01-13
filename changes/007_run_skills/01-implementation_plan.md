# PR 007: AxiomRegent Run Skills

## Goal Description
Implement the native Rust "Run" skills for AxiomRegent, specifically `test:go`, `test:build`, and `lint:gofumpt`. This enables the `axiomregent run` CLI to execute these common development tasks.

## Proposed Changes
### [run]
#### [MODIFY] [skills/mod.rs](file:///Users/bart/Dev/axiomregent/crates/run/src/skills/mod.rs)
- Add unit tests for skill IDs.

#### [VERIFY] [skills/test_go.rs](file:///Users/bart/Dev/axiomregent/crates/run/src/skills/test_go.rs)
- Implements `go test ./...`

#### [VERIFY] [skills/test_build.rs](file:///Users/bart/Dev/axiomregent/crates/run/src/skills/test_build.rs)
- Implements `go build ./...`

#### [VERIFY] [skills/lint_gofumpt.rs](file:///Users/bart/Dev/axiomregent/crates/run/src/skills/lint_gofumpt.rs)
- Implements `gofumpt` linting.

## Verification Plan
### Automated Tests
- Run `cargo test -p run` to execute the new unit tests.
- Run `cargo run -p run -- list` to verify registration.
