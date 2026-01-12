# Governance Engine (Preflight & Drift)

**Feature ID**: `GOVERNANCE_ENGINE`
**Implementation**: `crates/featuregraph` (Preflight/Drift logic)

## Overview
The Governance Engine enforces "physics" and policy within the repository. It ensures that changes conform to architectural rules and that the repository does not drift from its specified state.

## Tools

### `gov.preflight`
- **Description**: A virtual check run *before* execution to determine if a proposed change is valid.
- **Checks**:
  - **Policy Violations**: e.g., "Do not edit generated files manually".
  - **Architectural Constraints**: e.g., "Core cannot depend on extensions".
  - **Safety Tiers**: Assigns a safety tier (1-3) based on impact and operations.

### `gov.drift`
- **Description**: Detects discrepancies between the "should-be" state and the "is" state.
- **Usage**:
  - Run after execution to verify no unintended side effects.
  - Run periodically to find "rot" or unmanaged manual changes.
- **Violations**: Returns a list of specific violations (e.g., `DANGLING_FEATURE`, `SPEC_MISMATCH`).

## Safety Tiers
1.  **Tier 1 (Autonomous)**: Safe, low-impact, non-destructive.
2.  **Tier 2 (Gated)**: High-impact or destructive. Requires human review.
3.  **Tier 3 (Forbidden)**: Violates hard constraints. Cannot be executed.
