# axiomregent
axiomregent is a temporal, authoritative execution environment for a codebase.

It treats a repository as a deterministic state machine evolving over time. All reads are snapshot-grounded; all writes are lease-guarded; and every mutation is validated against explicit governance rules before being committed.

## Feature Status & Specifications

axiomregent is composed of 6 authoritative features. The source of truth for these capabilities is defined in the following specifications:

| Feature ID | Description | Specification | Status |
| :--- | :--- | :--- | :--- |
| **MCP_ROUTER** | Core MCP Router & Dispatch | [spec/core/router.md](spec/core/router.md) | Stable |
| **MCP_SNAPSHOT_WORKSPACE** | Snapshot & Workspace Tools | [spec/core/snapshot-workspace.md](spec/core/snapshot-workspace.md) | Stable |
| **FEATUREGRAPH_REGISTRY** | Feature Graph Registry | [spec/core/featuregraph.md](spec/core/featuregraph.md) | Stable |
| **GOVERNANCE_ENGINE** | Preflight & Drift Governance | [spec/core/governance.md](spec/core/governance.md) | Stable |
| **XRAY_ANALYSIS** | Repository Scanning Engine | [spec/xray/analysis.md](spec/xray/analysis.md) | Stable |
| **ANTIGRAVITY_AUTOMATION** | Antigravity Agent Protocol | [spec/antigravity/automation.md](spec/antigravity/automation.md) | Beta |
| **AXIOMREGENT_RUN_SKILLS** | AxiomRegent Run CLI Skills | [spec/run/skills.md](spec/run/skills.md) | Stable |

## Antigravity Automation

Antigravity is the repo-native agent integration layer. It allows autonomous agents to safely propose and execute changes using a deterministic "Changeset" protocol.

- **Changesets**: All agent actions are reified as artifacts in `changes/<change_set_id>/`.
- **Determinism**: Plans are hashed and immutable.
- **Safety**: Changes are gated by Safety Tiers (Autonomous, Gated, Forbidden).

See [Antigravity Spec](spec/antigravity/automation.md) for details.

## Quick Start

```bash
# Build the project
make rust-build

# Run tests
make rust-test

# Run lint checks
make rust-lint

# everything
make check
```

## Core Responsibilities

axiomregent provides the following guarantees:

1. **Temporal Ground Truth**
   •	Every repository state is identified by a snapshot_id
   •	Snapshots are immutable and replayable
   •	Historical states can be queried, diffed, and validated

2. **Deterministic Mutation**
   All write operations are:
   •	Explicit
   •	Context-aware
   •	Applied atomically
   •	No hidden side effects
   •	No implicit filesystem access

3. **Governance Enforcement**
   •	Structural rules (e.g. feature graphs, specs, invariants) are enforced before mutation
   •	Invalid changes are rejected with machine-readable violations
   •	Drift between declared intent and actual state is detectable

4. **Agent Isolation**
   •	Agents are treated as untrusted actors
   •	They cannot bypass validation
   •	They cannot mutate state without a valid lease and snapshot context

## Interaction Model (How LLMs Should Use It)

LLMs and agents must follow this pattern:

1. **Observe**
   •	Request a snapshot-grounded view of the repository
   •	Treat the returned data as the only valid source of truth

2. **Reason**
   •	Plan changes externally
   •	Determine intent, not file edits

3. **Propose**
   •	Submit explicit mutation operations
   •	Include the snapshot context the plan was based on

4. **Validate**
   •	Handle governance errors and violations deterministically
   •	Revise intent if necessary

5. **Commit**
   •	Accept the new snapshot_id as the new reality

## Temporal Programming Model (Key Insight for LLMs)

axiomregent enables temporal programming:
•	Time is a first-class dimension
•	State transitions are explicit
•	Bugs can be debugged by replaying history
•	Fixes can be applied from known-good snapshots

LLMs should reason in terms of:
•	“What snapshot am I operating on?”
•	“What invariant must remain true?”
•	“What is the minimal valid transition?”
