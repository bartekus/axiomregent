# Implementation Plan - PR 009: Verification Skills

# Goal
Implement repo-native **Verification Run Skills** to introduce a distinct `VERIFY` phase to the Antigravity protocol. This enables deterministic, policy-driven verification (linting, testing, formatting) of changesets using a constrained command runner and explicit configuration in `spec/verification.yaml`.

## User Review Required
> [!IMPORTANT]
> **Protocol Update**: Verification is a substate, not a new high-level state. `ChangesetStatus.state` will remaining in `executed`. A new `verification` field will track results.
> **Configuration**: The repository will require `spec/verification.yaml` to define verification profiles.
> **Breaking Change**: `ChangesetStatus` schema updated.

## Workflow Design (Walkthrough)

The introduction of the `VERIFY` phase extends the life of a changeset:

1.  **PROPOSE** (`antigravity.propose`)
    - Agent generates a plan.
    - Status: `pending_review` or `validated`.

2.  **EXECUTE** (`antigravity.execute`)
    - Status: `executed` (Terminal state for mutation).

3.  **VERIFY** (`antigravity.verify` - **NEW**)
    - **Trigger**: Manual or auto-triggered.
    - **Input**: `changeset_id`, `profile` (default: "pr").
    - **Resolution**:
        - Reads `spec/verification.yaml`.
        - Validates profile exists.
        - Runs **Toolchain Checks** first (e.g., `cargo --version`).
    - **Runner**:
        - internal `ConstrainedRunner` executes steps.
        - **Isolation**: Clears env, allows only allowlisted vars.
        - **Network**: Records `deny` vs `allow`. `deny` enforced via env scrubbing (best effort).
        - **Constraints**: Enforces timeouts.
    - **Determinism**:
        - Captures stdout/stderr SHA256.
        - Stores deterministic trace/preview (truncated to 4KB, no timestamps).
        - Records duration_ms.
    - **Drift Check**:
        - **Definition**: Changes to tracked files *or* snapshot hash mismatch.
        - **Exclusion**: `changes/<id>/verify/*` and `05-status.json` are IGNORED.
        - If drift detected + `read_only=tracked` -> **Fail** (Tier 3 violation).
    - **Artifact Generation**:
        - Writes `changes/<id>/verify/<skill_sanitized>.json` (Overwrites on rerun).
        - Writes `changes/<id>/verify/_toolchain.json` (Toolchain results).
    - **State Update**: Updates `05-status.json` with `verification` metadata.

4.  **FINALIZE**
    - PR generation logic checks `verification` status.

## Proposed Changes

### 1. Specification & Schemas

#### [NEW] `spec/verification.schema.json`
- **Location**: `crates/antigravity/src/schemas/verification.schema.json`
- **Content**: Draft 2020-12 schema for `spec/verification.yaml`.

#### [NEW] `spec/verify-result.schema.json`
- **Location**: `crates/antigravity/src/schemas/verify-result.schema.json`
- **Content**: Validates verification artifacts.

### 2. Antigravity Crate Updates (`crates/antigravity`)

#### [NEW] `src/verification/mod.rs` & `config.rs`
- Rust structs for `VerificationConfig`.
- **Validation**:
    - `tier` in {1, 2}.
    - Unique `step.name` per skill.
    - Profile references exist.
    - Env regex `^[A-Z0-9_]+$`.

#### [NEW] `src/verification/runner.rs`
- `ConstrainedRunner` struct (internal).
- **Env**: Clears `env::vars()`, applies allowlist.
- **Output**:
    - `stdout_preview`: first 4KB, valid UTF-8 (replace invalid chars).
    - `stdout_sha256`: Hash of full output.
- **No general MCP tool**: This runner is private to `antigravity`.

#### [NEW] `src/verification/engine.rs`
- `VerifyEngine::run(repo_root, changeset_id, profile)`.
- Logic:
    1.  **Toolchains**: Run required commands, write `_toolchain.json`.
    2.  **Skills**: Iterate skills in profile.
        - Check Tier eligibility.
        - Run steps.
        - **Post-Step**: Check drift (`git status` analogue + snapshot diff).
        - Calculate effective tier (escalate if drift/network/timeout).
        - Write `verify/<skill>.json`.
    3.  **Update Status**: With `last_run` summary.

#### [MODIFY] `src/schemas.rs`
- Add to `ChangesetStatusV1`:
  ```rust
  pub verification: Option<VerificationHistory>
  ```
- `VerificationHistory` contains `last_run` struct.

### 3. Integration (`src/`)

#### [MODIFY] `src/router/mod.rs`
- New tool: `antigravity.verify`.
- Schema: `{ changeset_id, profile? }` (profile defaults to "pr").

#### [MODIFY] `src/antigravity_tools.rs`
- Wire to `VerifyEngine`.

### 4. Config
- Create `spec/verification.yaml` in repo root.
- Add `verify.test`, `verify.lint`, `verify.format`.

## Tests

1.  **Config Validation**:
    - Fail on duplicate step names.
    - Fail on unknown skills in profile.
    - Fail on invalid Env vars.
2.  **Runner Constraints**:
    - Verify Env is scrubbed.
    - Verify timeout kills process.
3.  **Drift Detection**:
    - Step that `touch`es a file in `src/` triggers drift violation.
    - Step that `touch`es `changes/<id>/verify/log.txt` does NOT trigger drift.
4.  **Determinism**:
    - Verify JSON output is canonical (sorted keys).
    - Verify preview truncation works.

## Acceptance Criteria

- [ ] `antigravity.verify` overwrites existing artifacts on rerun.
- [ ] Toolchain checks run and record to `verify/_toolchain.json`.
- [ ] `status.verify` reflects the outcome.
- [ ] Network `deny` effectively scrubs proxy env vars.
- [ ] Drift check ignores artifacts directory.
