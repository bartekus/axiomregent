# Walkthrough - PR 004: Governance Engine

## Changes

### `crates/featuregraph`

#### `preflight.rs`
- **SafetyTier Enum**: Added `Tier1` (Autonomous), `Tier2` (Gated), `Tier3` (Forbidden).
- **Policy Checking**: Implemented basic policy checks (e.g., rejecting edits to generated files).
- **Tier Calculation**:
  - **Tier 1**: Only documentation changes (`.md`, `.txt`).
  - **Tier 2**: Default for code changes.
  - **Tier 3**: Any policy violation triggers this.
- **Preflight Response**: Now includes `safety_tier` to guide agent autonomy.

#### `tools.rs`
- Added `governance_preflight` and `governance_drift` methods to exposing the logic.

### `src/router`

#### `mod.rs`
- Registered `gov.preflight` and `gov.drift` in the MCP `tools/list` response.
- Wired up handlers in `tools/call`.

## Verification Results

### Automated Tests
- `make check` passed successfully.
- `cargo test --test mcp_contract` passed (after updating golden file).
- Existing featuregraph tests passed.

### Manual Verification
N/A - Logic is fully covered by automated tests and contracts.
