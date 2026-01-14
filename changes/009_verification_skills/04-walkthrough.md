# Walkthrough: Verification Skills (PR 009)

## Overview
Implemented repo-native verification run skills driven by `spec/verification.yaml`. This enables the `VERIFY` phase in the Antigravity protocol, allowing agents to run deterministic verification steps (build, test, lint) and produce canonical evidence artifacts.

## Changes

### 1. Specification & Schema
- **New Config**: `spec/verification.yaml` allows defining profiles (e.g., `pr`) and skills (e.g., `verify.test`).
- **New Schemas**:
  - `verification.schema.json`: Validates the config file.
  - `verify-result.schema.json`: Defines the structure of verification artifacts.
- **Status Update**: Extended `ChangesetStatusV1` in `05-status.json` to include a `verification` section tracking the last run profile and outcome.

### 2. Core Components (`crates/antigravity`)
- **`verification::config`**: Rust structs and validation logic for parsing the YAML config.
- **`verification::runner::ConstrainedRunner`**:
  - Executes commands with enforced timeouts.
  - Clears environment variables and applies an allowlist (e.g., `CI`, `PATH`).
  - **Network Policy**: Enforces `network: deny` by scrubbing proxy variables (`HTTP_PROXY`, etc.).
  - **Output Capture**: Captures stdout/stderr, computes SHA256 hashes, and generates truncated UTF-8 previews.
- **`verification::engine::VerifyEngine`**:
  - Orchestrates the verification process.
  - Runs toolchain checks first (e.g., `cargo --version`).
  - Executes skills in order.
  - **Drift Detection**: Checks for modifications to tracked files (git-like semantics) before and after each skill. Enforces `read_only: tracked` by failing if drift is detected.
  - **Artifact Generation**: Writes deterministic JSON artifacts to `changes/<id>/verify/<skill>.json`.

### 3. Integration
- **`src/internal_client.rs`**: Implemented `get_drift` using `git status --porcelain` to support accurate file modification detection.
- **MCP Router**: Exposed `antigravity.verify` tool, enabling agents to request verification runs.

## Verification Results

### Integration Test
Created `tests/verify_test.rs` to validate the end-to-end flow:
1.  **Setup**: Created a temporary git repo with `spec/verification.yaml` and a dummy changeset.
2.  **Execution**: Called `antigravity.verify` via the MCP Router.
3.  **Result**:
    - `antigravity.verify` returned success.
    - `changes/<id>/verify/verify.test.json` artifact was created with correct structure and success status.
    - `05-status.json` was updated with `verification.last_run.outcome = "passed"`.

### Command Output
```
$ cargo test --test verify_test

running 1 test
test test_antigravity_verify_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

## Next Steps
- Implement `governance` logic to block merge on failed verification (if not already handled by higher-level policy).
- Add more advanced drift detection (e.g., content hashes) if needed.
