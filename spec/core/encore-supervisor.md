# Feature: Encore Daemon Supervision

> [!IMPORTANT]
> This feature puts the Encore daemon under strict supervision of the AxiomRegent MCP server, ensuring deterministic lifecycle management, single-instance guarantees, and robust health monitoring.

## Goals
- **Determinism**: The daemon's lifecycle is strictly coupled to the MCP server.
- **Zero-Config**: Users get a working environment automatically.
- **Observability**: Clear visibility into daemon state, logs, and health.

## Architecture

The `encore::supervisor` module manages the daemon process using a strict **State Machine** and **OS-level Locking**.

### State Machine

| State | Description | Transition To |
| :--- | :--- | :--- |
| **Stopped** | Initial state. No process running. | `Starting` |
| **Starting** | Process spawned. Waiting for readiness checks. | `Healthy`, `Unhealthy`, `Fatal` |
| **Healthy** | Daemon is serving traffic (Layered checks pass). | `Unhealthy` |
| **Unhealthy** | Healthy check failed or crash detected. | `Backoff`, `Starting`, `Fatal` |
| **Backoff** | Waiting to restart after failure (rate-limited). | `Starting` |
| **Fatal** | Max restarts exceeded or unrecoverable error. | *Terminal* (until manual restart) |

### Ownership Model

The supervisor explicitly tracks its relationship to the daemon:

- **Managed**: The supervisor spawned the process and owns its lifecycle (signals, cleanup).
- **External**: The supervisor is monitoring an externally managed instance (e.g. `encore run` in a terminal).
- **Off**: Supervision is disabled by config.

## lifecycle Contract

### 1. Startup
On MCP Server start:
1.  **Lock Acquisition**: Attempt to acquire exclusive OS lock on `<repo_root>/.axiomregent/encore.lock`.
    *   **Success**: Set Ownership=`Managed`. Spawn daemon. Transition `Stopped` -> `Starting`.
    *   **Locked**: Check `AXIOM_ENCORE_MODE`.
        *   `auto`: Attempt fallback to External mode (check `encore.json` or env var).
        *   `external`: Set Ownership=`External`. Transition `Stopped` -> `Starting` (monitor only).

### 2. Process Management (Managed)
- **Spawn**: `encore run` with `ENCORE_TAG=axiomregent`. New process group.
- **Info File**: Write `.axiomregent/run/encore.json` with `{"endpoint": "...", "pid": ...}`.
- **Output**: Capture stdout/stderr to ring buffer.

### 3. Readiness Gate
Layered checks must pass for `Healthy` state:
1.  **L1 (TCP)**: Connect to port.
2.  **L2 (HTTP)**: `GET /` returns any response.
3.  **L3 (App)**: `GET /_encore/health` (or configured path) returns 200 OK.

> [!NOTE]
> `encore.status` tool works in ALL states and NEVER blocks. Tools requiring the daemon must explicitly await readiness.

### 4. Shutdown
On MCP Server shutdown (or `encore.stop`):
1.  **Signal**: Send `SIGTERM` to process group.
2.  **Wait**: 5 seconds.
3.  **Force**: Send `SIGKILL` if still running.
4.  **Cleanup**: Release lock. Delete `encore.json`.

### 5. Failure & Recovery
- **Crash**: Process exit (non-zero or signal).
- **Policy**: `AXIOM_ENCORE_RESTART_POLICY` (default: `on-failure`).
- **Guardrails**: Max X restarts in Y seconds. If exceeded -> `Fatal`.

## Configuration

| Env Variable | Default | Description |
| :--- | :--- | :--- |
| `AXIOM_ENCORE_MODE` | `auto` | `auto`, `external`, `off` |
| `AXIOM_ENCORE_ENDPOINT` | - | Explicit URL for external mode / fallback. |
| `AXIOM_ENCORE_HEALTH_PATH` | `/_encore/health` | Path for L3 readiness check. |
| `AXIOM_ENCORE_MAX_RESTARTS` | `5` | Max restart attempts before Fatal. |
| `AXIOM_ENCORE_RESTART_WINDOW`| `60s` | Time window for restart counting. |

## MCP Tools

### `encore.status`
Returns the current supervisor state. Non-blocking.
```json
{
  "ownership": "Managed",
  "state": "Healthy",
  "pid": 12345,
  "endpoint": "http://127.0.0.1:4000",
  "uptime_seconds": 300,
  "restart_count": 0,
  "last_error": null
}
```

### `encore.restart`
Triggers a restart sequence (Stop -> Start).
Input: `{"force": boolean}`. Output: `{"accepted": true}`.

### `encore.logs`
Retrieves recent logs from the ring buffer.
Input: `{"limit": 100, "offset": 0}`.
