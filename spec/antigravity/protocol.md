# Antigravity Automation: Protocol & Architecture

## 1. Goals and Non-Goals

### Goals
- **Determinism**: Ensure every action and artifact derivation is reproducible and independent of ephemeral state (time, temp paths, etc.).
- **Repo-Native Authority**: Use the Git repository itself (specifically the `changes/` directory) as the durable medium for agent-human-machine collaboration.
- **Strict MCP Enforcement**: All semantic understanding and modification of the repository must go through the core MCP server (`axiomregent`). The agent cannot invent its own parsing or modification logic.
- **Explicit Safety**: Enforce a "verify-then-trust" model where every change set is analyzed for risk, impact, and governance compliance before execution.
- **Reviewable Granularity**: Discrete, atomic change sets that can be inspected, approved, or rejected individually.

### Non-Goals
- **New Tooling**: We will not build new MCP tools. We rely exclusively on the existing, verified toolset (`features.*`, `gov.*`, `xray.scan`, etc.).
- **External State**: No external databases, artifact stores, or cloud dependencies. If it's not in the repo, it doesn't exist.
- **Implicit Automation**: No "magic" background fixers. All actions must be reified into a Change Set first.
- **Real-time Collaboration**: This is an async, artifact-driven protocol, not a real-time validaton loop.

---

## 2. Repo-Native Change Set Convention

The authoritative handshake between the Antigravity Agent and the execution environment is the **Change Set**.

**Root Directory**: `changes/`

### Folder Layout
Each change set resides in `changes/<change_set_id>/`:

```text
changes/<change_set_id>/
├── 00-meta.json              # Canonical identity & immutable context
├── 01-architecture.md        # Reasoning, context, and high-level design
├── 02-implementation-plan.json # Machine-readable, deterministic execution graph
├── 03-task-list.md           # Human-readable checklist (derived from plan)
├── 04-walkthrough.md         # (Optional) Proof-of-work, validation results
└── 05-status.json            # (Optional) Executor-managed state and decision record
```

### Canonical JSON Definition
To guarantee determinism, "Canonical JSON" is defined as:
1.  **UTF-8** encoding.
2.  **Lexicographical Sort**: Object keys sorted by byte value.
3.  **No Whitespace**: No spaces, tabs, or newlines between tokens.
4.  **Preserved Order**: Arrays retain authored order.
5.  **Minimal Numbers**: No trailing zeros (e.g., `1` not `1.0`).
6.  **No Trailing Newline**.

### Deterministic Change Set ID Derivation (Protocol: `changeset_id_v1`)

To ensure idempotency and prevent duplicate work, the `change_set_id` is derived deterministically.

**Inputs**:
1.  **Subject**: `subject` (string) - Brief description of intent.
2.  **Repo Key**: `repo_key` (string) - Normalized identity of the repository.
3.  **Base State**: `base_state` (object) - The precise state of the world this change starts from.

**Normalization Rules**:
-   **Subject Slug**: Lowercase, replace non-alphanumeric with `-`, trim `-`, max 48 chars.
-   **Repo Key**:
    -   If git remote `origin` exists: normalize to `host/owner/repo` (no scheme, no user, no port). Remove `.git` suffix.
    -   Fallback: Absolute path of `repo_root` (use with caution in shared envs).
-   **Base State**:
    -   If `snapshot`: `{"kind": "snapshot", "id": "<snapshot_id>"}`.
    -   If `worktree`: `{"kind": "worktree", "id": "<workspace_fingerprint>"}`. **Must** be derived from an MCP tool (e.g., `snapshot.info` or `gov.preflight` return value). If no worktree fingerprint tool exists, default to `snapshot` anchoring.

**Canonical Payload (`00-meta.json`)**:
MUST be serialized as Canonical JSON (sorted keys, no whitespace).

```json
{
  "base_state": {
    "id": "e93ac...",
    "kind": "snapshot"
  },
  "protocol": "changeset_id_v1",
  "repo_key": "github.com/bartekus/axiomregent",
  "subject": "refactor-login-flow"
}
```

**ID Generation**:
1.  `digest = sha256(canonical_payload_bytes)`
2.  `change_set_id = <subject_slug> + "--" + digest[0..12]`
3.  Collision handling: If `changes/<id>` exists AND `00-meta.json` content differs, extend digest to 16 chars.

**Integrity**:
`00-meta.json` **MUST** include:
- `plan_sha256`: SHA256 digest of the canonical bytes of `02-implementation-plan.json`.
- (Optional) `artifact_manifest`: Map of filename -> sha256 for all initial artifacts.

### Status Reuse Artifact (`05-status.json`)
Managed **exclusively** by the Local Executor. The Agent MUST NOT modify this file.

```json
{
  "schema_version": "changeset_status_v1",
  "change_set_id": "refactor-login-flow--...",
  "state": "draft|ready|validating|pending_review|executing|completed|failed|superseded",
  "decision": "auto|review_required|rejected|approved",
  "reason": "String explaining failure or rejection",
  "validated_against": {
    "base_state": { "kind": "...", "id": "..." },
    "plan_sha256": "..."
  },
  "execution": {
    "started_task_id": "task_01",
    "completed_task_ids": [],
    "last_error": "..."
  }
}
```
**Write Rules**:
1.  **Executor Only**: Only the trusted Local Executor may write to `05-status.json`.
2.  **Agent Prohibition**: The Agent may never modify `status.json`.
3.  **Immutability**: Any change to `02-implementation-plan.json` by the Agent requires a NEW `change_set_id` (superseding the old one).

---

## 3. System Architecture

The system consists of three distinct components separated by trust boundaries.

### 1. Antigravity Agent (Untrusted Proposer)
-   **Role**: Reasoning engine, architect, and plan author.
-   **Capabilities**:
    -   Reads repo state via MCP (`features.*`, `xray.scan`).
    -   Writes **Draft** artifacts to `changes/<change_set_id>/`.
    -   CANNOT directly apply patches or mutate code outside of `changes/`.
-   **Trust**: Low. Its output is treated as a "proposal" that must be verified.

### 2. MCP Server (`axiomregent`) (Authoritative Source of Truth)
-   **Role**: The "Physics Engine" of the repository.
-   **Capabilities**:
    -   Provides semantic understanding (`features.impact`, `gov.drift`).
    -   Manages snapshots and leases.
    -   Executes specific atomic operations (`workspace.apply_patch`).
-   **Trust**: Absolute. It is the only component allowed to assess impact or violation.

### 3. Local Executor / Client (Trusted Validator)
-   **Role**: The "Adult in the Room". A CLI watcher or GUI (AxiomRegent-View).
-   **Capabilities**:
    -   Watches `changes/` for new proposals.
    -   Validates proposals against the MCP Safe Contract.
    -   Decides whether to **Execute**, **Reject**, or **Request Review**.
    -   Calls MCP to apply plans to the worktree.
    -   Writes execution status to `changes/<id>/status.json` (or `changes/.executions/<id>.json`).
-   **Trust**: High. It enforces the safety config.

**Data Flow**:
1.  Agent **Analyze** -> MCP (`xray`, `features`)
2.  Agent **Plan** -> Write `changes/<id>/*`
3.  Client **Detect** -> Read `changes/<id>/00-meta.json`
4.  Client **Verify** -> Call MCP (`gov.preflight`, `features.impact`)
5.  Client **Execute/Gate** -> Call MCP (`workspace.apply_patch`) OR Wait for Human.

---

## 4. Implementation Plan Artifact Specification (`02-implementation-plan.json`)

This file contains the deterministic instructions for executing the change. It is strictly versioned.

```json
{
  "version": "plan_v1",
  "change_set_id": "refactor-login-flow--a1b2c3d4e5f6",
  "generated_from_digest": "sha256:e93ac...",
  "prerequisites": {
    "base_snapshot_id": "sha256:...",
    "required_leases": ["/src/auth"]
  },
  "tasks": [
    {
      "id": "task_01",
      "type": "mcp_call",
      "tool": "workspace.apply_patch",
      "params": {
        "repo_root": "...",
        "patch": "...",
        "mode": "snapshot"
      },
      "risk_profile": {
        "impact_score": "high",
        "touched_features": ["auth_core"],
        "destructive": false
      }
    }
  ]
}
```

**Rules**:
-   **Ordered**: Tasks are executed sequentially.
-   **Explicit Tools**: Only white-listed MCP tools (`workspace.*`, `snapshot.*`) are allowed for code mutation.
-   **No Hidden Shell**: `shell_command` tasks are FORBIDDEN in the automated plan to guarantee MCP authority. Verification uses `gov.preflight` or `features.impact` (or manual user steps in `04-walkthrough.md`).
-   **Risk Profile**: Each task must be annotated with its estimated risk (derived from `features.impact`).

---

## 5. Artifact Persistence Rules

### Immutability & Rewrites
-   **Approved Plans are Immutable**: Once a human or the automated system marks a plan as `APPROVED` (via a status file or git tag), the Agent MUST NOT overwrite it.
-   **Drafts are Mutable**: The Agent can overwrite files in `changes/<id>/` *only if* the Change Set is in `DRAFT` or `REQUESTED_CHANGES` state.
-   **Supersession**: If a plan needs to change after approval, a **NEW** Change Set (new ID) must be created. The old one is marked as `SUPERSEDED` in `00-meta.json` (or a sidecar `status.json`).

### Integrity
-   `00-meta.json` dictates the ID. If the content doesn't hash to the ID, the folder is invalid and ignored.
-   `02-implementation-plan.json` must checksum matches the one referenced in `00-meta.json` (if we add robust integrity checking later). For now, filesystem consistency is assumed within the single atomic write of the plan.

---

## 6. Ingestion & Validation Pipeline

The Client follows this loop:

1.  **Discovery**: Scan `changes/*/00-meta.json`.
2.  **Filter**: Check `changes/<id>/status.json`. If present, skip (already processed).
3.  **Validation (The "Safe Contract")**:
    -   **Schema Check**: Validate `02-implementation-plan.json` against `plan_v1` schema.
    -   **Integrity Check**: Validate `00-meta.json`'s `plan_sha256` matches the plan file.
    -   **Repo Alignment**: The Executor calls `gov.preflight` (and/or `snapshot.info`) in the plan's declared mode.
        -   If `snapshot`: Verifies snapshot exists and `base_state.id` matches.
        -   If `worktree`: Queries MCP for the deterministic worktree status/fingerprint. IF it does not match `base_state.id`, the plan is STALE.
    -   **Governance Check**: Call `gov.preflight` AND `gov.drift` **in the matching mode** (`snapshot` or `worktree`).
        -   If `gov.drift` returns violations -> **REJECT**.
    -   **Impact Analysis**: Call `features.impact`.
        -   If critical features touched -> **FLAG_FOR_REVIEW**.
4.  **Decision**:
    -   **Pass**: Schedule for execution.
    -   **Fail**: Write `status.json` with `{"state": "failed", "reason": "..."}`.
    -   **Review**: Notify user.

---

## 7. State & Concurrency Model

### Ecosystem State (`changes/`)
-   **Concurrent Drafts**: Multiple `changes/*` folders can exist.
-   **Serialized Execution**: The Client executes only ONE change set at a time per worktree.
    -   **Locking**: Executor creates `changes/.locks/worktree.lock` (or OS-level file lock).
    -   **Release**: Lock is released only when `status.json` reaches a terminal state (`completed`, `failed`, `rejected`).

### Change Set Lifecycle
1.  **DRAFT**: Being written by Agent.
2.  **READY**: Meta + Plan exist.
3.  **VALIDATING**: Client is checking preflight/impact.
4.  **PENDING_REVIEW**: Gated by Tier 2.
5.  **EXECUTING**: In-flight.
6.  **COMPLETED**: Successfully applied.
7.  **FAILED**: Execution error.
8.  **SUPERSEDED**: Replaced by a newer version (e.g., due to stale lease).

### Staleness
If `current_repo_hash` != `plan.base_state.id`:
-   Plan is **STALE**.
-   Agent must:
    1.  Read stale plan.
    2.  Check if patch still applies (dry-run).
    3.  If yes: Re-base (create new Change Set with new base).
    4.  If no: Re-plan.
