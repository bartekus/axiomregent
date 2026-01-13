# PR 004: Governance Engine Implementation Plan

## Goal Description
Implement the **Governance Engine** (`GOVERNANCE_ENGINE`) as defined in `spec/core/governance.md`. This engine provides "physics" and policy enforcement for the repository, ensuring changes conform to architectural rules and preventing drift. It introduces two key MCP tools: `gov.preflight` and `gov.drift`.

## User Review Required
> [!IMPORTANT]
> **Safety Tier Logic**: I am implementing a baseline heuristic for Safety Tiers:
> - **Tier 1 (Autonomous)**: Changes limited to documentation (`*.md`).
> - **Tier 2 (Gated)**: Default for code changes.
> - **Tier 3 (Forbidden)**: If any critical violations are found (e.g. editing generated files).
>
> Please confirm if this heuristic aligns with expectations or if stricter rules are needed immediately.

## Proposed Changes

### `crates/featuregraph`

#### [MODIFY] [preflight.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/preflight.rs)
- Update `PreflightResponse` to include `safety_tier` field.
- Add `SafetyTier` enum.
- Implement `check_policy_violations`:
  - Detect edits into generated files.
- Implement `calculate_safety_tier`:
  - Analyze `changed_paths` and violations to determine the tier.
- Update `check` method to call these new logic components.

#### [MODIFY] [tools.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/tools.rs)
- Add `governance_preflight` method to `FeatureGraphTools`.
- Add `governance_drift` method to `FeatureGraphTools`.

#### [MODIFY] [lib.rs](file:///Users/bart/Dev/axiomregent/crates/featuregraph/src/lib.rs)
- Ensure `preflight` module is public (already is).

### `src` (MCP Server)

#### [MODIFY] [router/mod.rs](file:///Users/bart/Dev/axiomregent/src/router/mod.rs)
- Register `gov.preflight` and `gov.drift` tools in the MCP router.

## Verification Plan

### Automated Tests
- **Unit Tests**: Add tests in `preflight.rs` covering:
  - Allowed vs Disallowed changes (policy violations).
  - Safety Tier calculation (Doc-only -> Tier 1, Code -> Tier 2).
  - Dangling feature checks.
- **Integration Check**: Run `make check` to ensure no regressions.

### Manual Verification
- None required for this PR as it is a backend logic implementation. The automated tests should cover the requirements.
