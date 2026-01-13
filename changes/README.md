# Changes

This directory contains the immutable history of all authorized changesets applied to this repository by the Antigravity agent.

## Structure

Each subdirectory represents a distinct changeset, named by its deterministic `changeset_id`.

Inside each changeset directory:
- `00-meta.json`: Canonical identity and context.
- `01-implementation_plan.md`: Execution graph / Implementation Plan.
- `04-walkthrough.md`: Execution record and results.
- `05-status.json`: Current state of the changeset.

## Governance

- Do not manually edit files in this directory unless resolving a corruption issue.
- The `antigravity` agent manages this directory.

See [spec/antigravity/automation.md](../spec/antigravity/automation.md) for full protocol details.
