# UI Specification

This document defines the UI requirements for `axiomregent-view`.

## Route Map (React Router v7)

```
/                   -> Connection / Dashboard
/tools              -> Tool Explorer
/tools/:name        -> Tool Runner / Docs
/activity           -> Activity Log (Runs, Changesets)
/activity/:id       -> Run Detail / Changeset Detail
/changesets         -> Changeset List
/changesets/:id     -> Changeset Detail (includes Verification substate)
```

## Core UI Sections

### 1. Connection / MCP Inspector
- **Responsibility**: Establish connection to AxiomRegent server.
- **Components**:
  - `ConnectionStatus`: Shows green/red status, latency, and protocol version.
  - `ServerCapabilities`: Displays capabilities returned by `initialize`.
  - `MountList`: Renders response from `list_mounts`.

### 2. Tools Explorer
- **Responsibility**: Browsable catalog of all discovered tools.
- **Components**:
  - `ToolGrid`: Grid of `ToolCard`s.
  - `ToolCard`: Shows name, description, and "Run" button.
  - **Sorting**: Alphabetical by name.
  - **Filtering**: By name or category (inferred from prefix `gov.`, `antigravity.`, etc).

### 3. Tool Runner
- **Responsibility**: Execute tools with manual input.
- **Components**:
  - `SchemaForm`: Dynamic form generated from `inputSchema`.
  - `JSONPreview`: Real-time preview of the request JSON.
  - `ConsoleOutput`: ANSI-capable terminal for logs/output.
  - `History`: Local history of recent calls.

#### Safety Gates
For tools marked as **Tier 3 (Execution/Destructive)** in the contract or determined by name heuristics (e.g. `*.execute`, `*.delete`):
- **Confirmation Step**: The UI **must** show a confirmation modal or interstitial before execution.
- **Warning Banner**: A distinct warning (e.g., yellow/red border or banner) must be visible.
- **Explicit Action**: The user must click a secondary "Confirm Execution" button.

### 4. Changesets (Antigravity)
- **Responsibility**: View and manage Antigravity changesets.
- **Components**:
  - `ChangesetList`: Table of active changesets (Status, ID, Subject).
  - `ChangesetDetail`:
    - **Header**: Status, ID, Base State.
    - **PlanView**: Renders `ImplementationPlanV1`.
    - **ExecutionLog**: Real-time log of execution steps.
    - **VerificationPanel**: **Substate view**. Only visible if `validation` passed.

#### Verification Substate
The `VerificationPanel` renders `ChangesetStatus.verification`.
- If `verification` is null: Show "Verification not run".
- If `verification` exists:
  - Show `last_run.outcome` (Success/Failure).
  - Show `last_run.timestamp` (if present).
  - Show `last_run.profile`.

### 5. Activity / Runs
- **Responsibility**: Global log of all tool executions.
- **Components**:
  - `ActivityFeed`: Chronological list of `Action` events.

## State Model

### Global State (`MCPContext`)
- `connectionStatus`: Connected | Connecting | Disconnected
- `tools`: `ToolDescriptor[]` (Result of `tools/list`)
- `capabilities`: `ServerCapabilities`

### Route-Local State
- **Tool Runner**: Form state, ephemeral execution results.
- **Changeset Detail**: Auto-refreshing `ChangesetStatus` (polling every 2s when active).

### Substates
- **Verification** is NOT a top-level route. It is a nested view within `ChangesetDetail`.

## UX Rules

1. **Sorting**: All lists (Tools, Files, Changesets) must be sorted alphabetically by ID/Name unless a timestamp is present (then reverse chron).
2. **Empty States**: Every list must have a distinct "No items found" state.
3. **Loading States**: Skeletons for initial loads; spinners for actions.
4. **Optimistic Updates**: STRICTLY FORBIDDEN. The UI must always reflect the server truth.
5. **Canonization**: All user input for `repo_key`, `changeset_id` must be trimmed and validated against regex before submission.
