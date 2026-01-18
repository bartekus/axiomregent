# AxiomRegent View Specification

## Purpose
This directory defines the UI Contract for `axiomregent-view`, a client interface for the AxiomRegent MCP server. This spec allows engineering interfaces (like Cursor or custom web apps) to interact with AxiomRegent deterministically, without inventing new protocols or assuming hidden behavior.

## Scope
- Defines the **contract** between the UI and the MCP server.
- Defines the **canonical data models** used for exchange.
- Defines the **user interface requirements** based on server capabilities.
- Defines **testing and verification** strategies.

## Non-Goals
- Implementing the UI itself (this is a spec, not an app).
- Defining the business logic of AxiomRegent (that lives in the server spec).
- Changing the MCP protocol (we use standard MCP).

## Glossary
- **MCP**: Model Context Protocol. The standard used for communication.
- **Tool**: An executable function exposed by the server (e.g., `antigravity.propose`).
- **Changeset**: A proposed set of changes to the repository, managed by Antigravity.
- **Verification**: A substate of execution where changes are validated against requirements.
- **Tier**: A safety classification for operations (Tier 1: Safe/Read-only, Tier 2: Gated/Mutation, Tier 3: Forbidden/Hard).

## Consumption Rules
1. **Discovery First**: Do not assume tools exist. Use `tools/list` to discover capabilities.
2. **Schema Compliance**: Interact only using the canonical schemas defined in `data_models.json` and `mcp_contract.md`.
3. **Deterministic Rendering**: The UI must be a pure function of the MCP state. No hidden client-side state is allowed unless explicitly derived from server data.
4. **Error Handling**: All errors must be displayed using the canonical error schema.

## Missing Tools
If a tool referenced in this spec is not returned by `tools/list`, the UI **must not** render the corresponding feature. Graceful degradation is required.
