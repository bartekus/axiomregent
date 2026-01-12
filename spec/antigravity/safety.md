# Antigravity Safety Policy

## 1. Safety Tiers

We define three **Safety Tiers** to categorize the risk level of automated changes.

### Tier 1: Autonomous (Auto-Approve)
**Conditions**:
-   `gov.preflight` = PASS.
-   `gov.drift` = NONE.
-   `features.impact`: Only touches "Low Impact" scope (e.g., docs, tests, non-core features).
-   Operation: Non-destructive (adds files, safe edits).
-   Verification: Compilation/Tests pass in `snapshot` mode first.

### Tier 2: Governance Gated (Review Required)
**Conditions**:
-   touches `High Impact` features (core business logic).
-   `gov.drift` found existng issues (cleanup required).
-   Operation: Deletes files or large refactors.

### Tier 3: Forbidden (Hard Block)
**Conditions**:
-   `gov.preflight` = FAIL (Violates repo policy).
-   Attempting to modify `.git`, `.mcp`.
-   Modifying files **outside** of `changes/<current_change_set_id>/` (e.g. other change sets, or root `changes/` config).
-   Invalid MCP tool usage.

## 2. Human-Review Rules

### Bypass Mechanism
-   **Approved Marker**: A human can force-execute a Tier 2 plan by creating a file named `APPROVED` in the `changes/<id>/` directory.
-   **Validation**: The Executor sees the `APPROVED` file and (if Tier 3 checks pass) updates `status.json` to `decision: approved` and proceeds.
-   **Constraint**: Tier 3 Violations cannot be bypassed even with an `APPROVED` marker.

## 3. Tool Allowlist

The Executor enforces a strict allowlist of tools that can be called during execution:

| Tool | Allowed Tiers | Notes |
| :--- | :--- | :--- |
| `gov.preflight` | All | Mandatory check. |
| `gov.drift` | All | Mandatory check. |
| `features.impact` | All | Mandatory check. |
| `workspace.apply_patch` | Tier 1, 2 | Main mechanism for code change. |
| `snapshot.create` | Tier 1, 2 | |
| `snapshot.info` | All | |
| `write_file` | Tier 1, 2 |  |
| `run_command` | **Tier 3 (Forbidden)** | Shell commands are not allowed in the automated plan to preserve determinism and safety. |

## 4. Error Handling & Safety
-   `STALE_LEASE` / `REPO_CHANGED`:
    -   **Action**: Abort immediate execution.
    -   **Agent**: Re-analyze. If strategy is still valid, update `base_state` and derive NEW Change Set ID.
-   `GOVERNANCE_FAIL`:
    -   **Action**: Hard stop. Write failure report to change artifacts.
