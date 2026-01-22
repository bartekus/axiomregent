## Encore Go → Encore.ts → Firecracker Diff Trace Map

### 1. Purpose

- Map the evolution of Encore’s local dev behavior across:
  - **Go daemon implementation** (original Encore runtime).
  - **Encore.ts / Rust+Node supervisor**.
  - **Pronghorn Firecracker wrapper**.
- For each behavior, identify:
  - Where it was implemented in the Go codebase.
  - Where it lives in the Encore.ts stack (conceptually, with known entrypoints).
  - Where Pronghorn’s Firecracker integration will plug in.

> Note: Source-level details for Encore.ts internals are limited in public docs; this trace uses concrete Go code references where available and docs-based references for TS/Rust. All new Firecracker behavior is described as Pronghorn-side components.

---

### 2. Watcher and Rebuild Pipeline

- **Go daemon (historical behavior)**
  - **Responsibility**
    - Monitor source files for changes.
    - Rebuild the application and restart processes on change.
  - **Implementation (Go)**
    - File: `cli/daemon/run/run.go`
    - Key symbols (per DeepWiki summary of Encore Go repo):
      - `buildAndStart(...)` – builds the app, then starts processes in a `ProcGroup`.
      - `ProcGroup` – encapsulates running processes; on reload, a new group is created and the old one is shut down.
      - Live reload logic monitors file system changes and calls `buildAndStart` again.
    - Evidence:
      - DeepWiki “Application Execution / API Gateway” section referencing `cli/daemon/run/run.go` and `buildAndStart`.

- **Encore.ts / Rust+Node supervisor**
  - **Responsibility**
    - Watch TS project files (`*.ts`, `encore.service.ts`, etc.).
    - Re-run TS parser and rebuild runtime artifacts.
    - Restart Node+Rust runtime on change.
  - **Implementation (TS/Rust) – conceptual**
    - TS parser invoked via an external binary or library referenced by `ENCORE_TSPARSER_PATH`.
    - Rust runtime binaries referenced by `ENCORE_RUNTIME_PATH`.
    - CLI orchestrates watchers and rebuilds using these tools.
  - **Evidence**
    - Encore.ts docs on hot reload and daemon-driven recompilation.  
      - Path: `docs/ts/quick-start` (Encore docs)
    - Environment variables for parser/runtime paths.  
      - Path: `docs/ts/develop/env-vars` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Ensure the watcher and rebuild pipeline run **inside** the microVM unchanged.
    - Provide:
      - Workspace mount at `/workspace` with correct file change semantics.
      - Sufficient inotify/fanotify capabilities in guest kernel for Encore watchers.
    - No direct modifications to Encore watcher code; only environment the watcher sees.
  - **Key interfaces**
    - VM lifecycle APIs (`createWorkspace`, `startVM`, `syncFiles`).
    - Host does not interpret Encore’s file watcher; it only manages storage and VM state.

---

### 3. Build Pipeline and Metadata Generation

- **Go daemon**
  - **Responsibility**
    - Parse Go services and infra declarations.
    - Build binaries and runtime configuration.
  - **Implementation**
    - File: `cli/daemon/run/run.go` and supporting packages.
    - `buildAndStart` coordinates:
      - Code compilation.
      - Config generation.
      - Infra provisioning.
  - **Evidence**
    - DeepWiki: “buildAndStart compiles the application and starts processes in a ProcGroup.”

- **Encore.ts**
  - **Responsibility**
    - Run TS parser to:
      - Discover services.
      - Emit schemas and metadata (for API explorer, infra provisioning).
    - Build JS artifacts from TS.
  - **Implementation**
    - TS parser (external binary) invoked by CLI:
      - Controlled by `ENCORE_TSPARSER_PATH`.
    - Rust runtime consumes metadata to configure API gateway and infra.
  - **Evidence**
    - Encore.ts benefits and architecture docs: static analysis and type-safe API generation.  
      - Path: `docs/ts/concepts/benefits` (Encore docs)
    - Environment variables for TS parser path.  
      - Path: `docs/ts/develop/env-vars` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Provide a stable filesystem (`/workspace`) and environment for Encore CLI to:
      - Locate TS parser and runtime binaries.
      - Read/write metadata and build artifacts.
    - Ensure build caches persist across VM idle/restore where possible (through snapshots).
  - **Key integration points**
    - VM image includes TS parser and Rust runtime.
    - Optional: host awareness of build completion via:
      - Health checks.
      - `buildStarted` / `buildFinished` events emitted by a small guest agent when Encore logs indicate start/finish.

---

### 4. Port Assignment and HTTP Gateway

- **Go daemon**
  - **Responsibility**
    - Start HTTP server(s) for API gateway.
    - Bind to configurable listen address/port.
  - **Implementation**
    - Files under `runtime` / `gateway` and `cli/daemon/run`.
    - Exposes a single public port for API (e.g. `localhost:4000`).
  - **Evidence**
    - DeepWiki documentation of “API Gateway” and Go runtime modules.

- **Encore.ts**
  - **Responsibility**
    - Expose app HTTP server(s) via gateway inside Node+Rust runtime.
    - Bind to port specified by `encore run --port` or defaults.
  - **Implementation**
    - Rust runtime and Node bindings constitute the gateway.
  - **Evidence**
    - Encore.ts quickstart examples showing running on `localhost:4000`.  
      - Path: `docs/ts/quick-start` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Map VM internal ports to host-exposed ports per workspace:
      - `VM_IP:4000` → `HOST_IP:HOST_APP_PORT`.
      - `VM_IP:9400` → `HOST_IP:HOST_DASHBOARD_PORT`.
    - Integrate with Pronghorn routing (e.g., per-project subdomain).
  - **Implementation**
    - Host-side port proxy:
      - Could be iptables DNAT or sidecar proxy (nginx/envoy).
    - Control plane:
      - Records port mappings in database (e.g., `project_deployments` in Supabase).
  - **Evidence (Pronghorn)**
    - Existing deployment data model handling per-deployment URLs and runtimes.  
      - Path: `supabase/migrations/20251208042729_a63ac5ad-4cf0-42bf-a279-8954a8bbfb5d.sql` (deployment tables)

---

### 5. Infra Emulation (DB, Pub/Sub, Buckets)

- **Go daemon**
  - **Responsibility**
    - Provision and manage local infra services.
  - **Implementation**
    - `ResourceManager` and related types/functions in `cli/daemon/run/run.go` start:
      - Postgres clusters.
      - Pub/Sub worker(s).
      - Object storage mocks.
  - **Evidence**
    - DeepWiki: mentions `ResourceManager` managing database cluster and services.

- **Encore.ts**
  - **Responsibility**
    - Provide local infra for TS projects with the same semantics.
    - Use Docker for Postgres where possible.
  - **Implementation**
    - CLI calls into internal infra provisioners.
    - Uses host Docker or equivalent inside the dev environment.
  - **Evidence**
    - Docker requirement for Encore.ts local databases.  
      - Path: `docs/ts/install` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Provide infra **inside the VM** so Encore’s expectations remain intact:
      - Install Postgres.
      - Optionally run Docker-in-VM if Encore hardcodes Docker usage.
    - Ensure infra endpoints are reachable only within the VM, not exposed externally.
  - **Implementation**
    - Guest image includes infra daemons.
    - Pronghorn may:
      - Preconfigure infra on boot (systemd units).
      - Allow Encore to manage lifecycle via Docker within VM if needed.

---

### 6. Auth, Secrets, and Environment

- **Go daemon**
  - **Responsibility**
    - Load configuration and secrets.
    - Inject env vars into running processes.
  - **Implementation**
    - Config modules and secret managers in Encore Go.
  - **Evidence**
    - Encore Go docs on config and secret handling.  
      - Path: `docs/go/develop/config`, `docs/go/develop/secrets` (Encore docs)

- **Encore.ts**
  - **Responsibility**
    - Use Encore-managed secrets and environment variables.
    - Provide consistent interface via TS SDK.
  - **Implementation**
    - CLI reads local secrets store and passes them to TS runtime.
  - **Evidence**
    - Encore.ts config and secrets docs.  
      - Path: `docs/ts/develop/secrets` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Bridge Pronghorn secrets (stored in Supabase) into VM environment:
      - On `startVM`, orchestrator writes:
        - `.env` file into workspace volume.
        - Or exports env vars in VM’s `encore run` process environment.
    - Never expose secrets in host environment beyond orchestrator service.
  - **Implementation**
    - Host queries Supabase for project secrets (e.g., from `project_deployments.env_vars`).
    - VM agent uses metadata channel (vsock) to fetch secrets at runtime and set env.

---

### 7. Logs and Observability

- **Go daemon**
  - **Responsibility**
    - Aggregate logs from runtime processes and infra.
    - Feed them to the Dev Dashboard.
  - **Implementation**
    - Daemon collects stdout/stderr and telemetry from processes.
  - **Evidence**
    - Go docs and DeepWiki: logs and dashboard integration.

- **Encore.ts**
  - **Responsibility**
    - Emit logs and traces from TS app via Rust runtime to dashboard.
  - **Implementation**
    - Logging pipeline integrated with dashboard server.
  - **Evidence**
    - Local Dev Dashboard docs showing logs and traces for TS apps.  
      - Path: `docs/ts/quick-start` (Encore docs)

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Provide an external “logs tap” without modifying Encore:
      - Capture guest console (serial/stdout).
      - Optionally tail Encore log files.
    - Expose logs via `attachLogs` streaming API to Pronghorn UI.
  - **Implementation**
    - Firecracker exposes serial console; host reads from it.
    - Alternatively, guest agent forwards logs over vsock.

---

### 8. Gateway Proxy and Readiness

- **Go daemon**
  - **Responsibility**
    - Determine when runtime is ready to serve.
    - Manage process lifecycle on failure/reload.
  - **Implementation**
    - `ProcGroup` health and readiness checks.
  - **Evidence**
    - DeepWiki mention of readiness and process groups in `cli/daemon/run/run.go`.

- **Encore.ts**
  - **Responsibility**
    - Similar readiness semantics: once TS and Rust runtime are up, app and dashboard are available.
  - **Implementation**
    - Internally, CLI probably monitors log output or health endpoints.

- **Pronghorn Firecracker wrapper**
  - **Responsibility**
    - Surface readiness to Pronghorn:
      - Poll Dev Dashboard health endpoint via forwarded port.
      - Or rely on a guest agent that reports ready via vsock.
    - Emit `vmReady` and `portOpen` events when readiness detected.
  - **Implementation**
    - Host orchestrator monitors forwarded dashboard URL or an internal health RPC.

---

### 9. Summary: Where Firecracker Wrapper Intercepts

- **Go daemon → TS supervisor continuity**
  - Core loop (build, start, reload) now handled by Encore.ts CLI and Rust runtime.
  - Firecracker wrapper **does not** replace this logic; it only:
    - Provides an isolated Linux environment.
    - Manages lifecycle and networking.

- **Firecracker wrapper responsibility matrix**

| Concern            | Legacy (Go)       | Encore.ts / Rust Supervisor     | Firecracker Wrapper (Pronghorn)           |
|--------------------|-------------------|----------------------------------|-------------------------------------------|
| Build & reload     | `buildAndStart`, `ProcGroup` | TS parser + CLI daemon            | None – provide FS and CPU                 |
| Infra provisioning | `ResourceManager` | TS infra provisioning logic      | Provide Postgres & local services in VM   |
| Ports & routing    | Daemon + gateway  | Gateway inside Rust/Node runtime | Host-to-VM port forwarding, workspace URLs|
| Logs & traces      | Daemon & dashboard| TS dashboard integration         | Capture console/logs, expose `attachLogs` |
| Secrets/env        | Go config modules | TS secrets/config                | Inject env/secrets into VM environment    |
| Readiness          | ProcGroup health  | TS runtime readiness             | Expose `vmReady` events                   |

---

### 10. Recommendation & 2‑Week Plan (Trace-Focused)

- **Recommendation**
  - Treat Encore.ts + Rust supervisor as a black box from Pronghorn’s perspective.
  - Rely on:
    - Documented CLI behavior and env vars.
    - Stable filesystem interface at `/workspace`.
    - VM-level lifecycle and port management.
  - Avoid patching internal watcher/build logic; any necessary hooks should be added via:
    - Small guest-side agents.
    - Env vars (e.g., binding dashboard to `0.0.0.0:9400`).

- **2‑Week Trace-Oriented Plan**
  - **Week 1**
    - Clone Encore repo and confirm:
      - Go daemon references (`cli/daemon/run/run.go`, `buildAndStart`, `ProcGroup`, `ResourceManager`).
      - TS parser integration points (`ENCORE_TSPARSER_PATH` usage).
      - Rust runtime loader (function names and modules).
    - Document actual subprocess trees on a real Encore.ts project using `ps`/`strace` inside a dev VM.
  - **Week 2**
    - Align Firecracker wrapper design with confirmed paths:
      - Validate which env vars must be set (ports, dashboard listen address, log paths).
      - Confirm how to detect readiness (dashboard health or other).
    - Update this diff trace map with precise TS/Rust file paths and function symbols.

