# Encore Daemon Boot Sequence and Runtime Behavior Analysis

**Source-grounded analysis for Rust porting to AxiomRegent**

This document provides a detailed, source-cited analysis of how `encore daemon -f` boots, binds ports, serves components, tracks apps, runs build/run pipelines, and shuts down. Every claim is backed by specific file paths, function names, and code references.

---

## 1. Boot Sequence Narrative

### Entrypoint and Initialization

**CLI Entrypoint**: `cli/cmd/encore/daemon.go:18-36`
- Cobra command `daemon` with `-f` (foreground) flag
- When `-f` is set, calls `daemonpkg.Main()` directly
- Without `-f`, spawns daemon in background via `cmdutil.StartDaemonInBackground()`

**Main Function**: `cli/cmd/encore/daemon/daemon.go:60-94`
```go
func Main() {
    watcher.BumpRLimitSoftToHardLimit()
    if err := redirectLogOutput(); err != nil {
        log.Error().Err(err).Msg("could not setup daemon log file, skipping")
    }
    if err := runMain(); err != nil {
        log.Fatal().Err(err).Msg("daemon failed")
    }
}
```

**Boot Timeline**:

1. **Log Setup** (`cli/cmd/encore/daemon/daemon.go:475-497`)
   - Log path: `env.EncoreDaemonLogPath()` → `~/Library/Caches/encore/daemon.log` (macOS)
   - Logs: `"writing output to %s"` at line 485
   - Dual output: stderr (console) + file (daemon.log)
   - Uses zerolog with RFC3339Nano timestamps

2. **Signal Handling** (`cli/cmd/encore/daemon/daemon.go:72-94`)
   - Creates `signal.NotifyContext(context.Background(), syscall.SIGINT)`
   - Exit channel: `exit := make(chan error)` for subsystem failures
   - Deferred cleanup: `defer d.closeAll()`

3. **Daemon Initialization** (`cli/cmd/encore/daemon/daemon.go:127-187`)
   - **Unix Socket**: `d.listenDaemonSocket()` → `~/Library/Caches/encore/encored.sock`
   - **TCP Listeners** (with retry): Dashboard (9400), DBProxy (9500), Runtime (9600), Debug (9700), ObjectStorage (9800), MCP (9900)
   - **SQLite DB**: Opens `conf.Dir()/encore.db`, runs migrations
   - **Managers**: Apps, Secret, RunMgr, NS, ClusterMgr, ObjectsMgr, MCPMgr
   - **Trace Store**: SQLite-backed, starts cleanup goroutine (60s interval)

4. **Component Serving** (`cli/cmd/encore/daemon/daemon.go:189-197`)
   - Spawns 7 goroutines: `serveDaemon()`, `serveRuntime()`, `serveDBProxy()`, `serveDash()`, `serveDebug()`, `serveObjects()`, `serveMCP()`
   - Each sends errors to `d.exit` channel

5. **App Watching** (`cli/daemon/daemon.go:82-83`)
   - Background goroutine: `go srv.watchApps()`
   - Registers app listener for code generation
   - Sets up file watchers via `apps.WatchAll()`

### Critical Boot Order

- **Socket creation happens BEFORE serving**: `listenDaemonSocket()` creates socket synchronously, then `serveDaemon()` starts gRPC server
- **TCP listeners start retrying in background**: `listenTCPRetry()` spawns goroutine that retries binding
- **"serving X" log can appear BEFORE "listening on port X"**: See retry logic below
- **DB migrations run synchronously**: Blocks until complete before continuing

---

## 2. Component Map (Ports, Protocols, Handlers)

| Component | Port | Protocol | Server Type | Implementation | Key Endpoints |
|-----------|------|----------|-------------|---------------|---------------|
| **daemon** | Unix socket | gRPC | `grpc.Server` | `cli/cmd/encore/daemon/daemon.go:255-260` | `daemonpb.DaemonServer` RPCs |
| **dashboard** | 9400 | HTTP | `http.Server` | `cli/cmd/encore/daemon/daemon.go:284-288` | `/__encore` (WebSocket), `/__graphql` (proxy) |
| **dbproxy** | 9500 | HTTP | `http.Server` | `cli/cmd/encore/daemon/daemon.go:269-272` | `ClusterMgr.ServeProxy()` |
| **runtime** | 9600 | HTTP | `http.Server` | `cli/cmd/encore/daemon/daemon.go:262-267` | Trace ingestion, engine endpoints |
| **debug** | 9700 | HTTP | `http.Server` | `cli/cmd/encore/daemon/daemon.go:290-300` | `/debug/pprof/*` (Go pprof) |
| **objectstorage** | 9800 | HTTP | `http.Server` | `cli/cmd/encore/daemon/daemon.go:279-282` | `PublicBuckets.Serve()` |
| **mcp** | 9900 | HTTP/SSE | `http.Server` | `cli/cmd/encore/daemon/daemon.go:274-277` | `MCPMgr.Serve()` (SSE endpoint) |

### Daemon gRPC Server

**Implementation**: `cli/daemon/daemon.go:42-86`
- **Key RPCs**:
  - `Version()`: Returns daemon version + config hash
  - `Run()`: Streaming RPC, starts app, streams `CommandMessage` (Output/Exit/Error)
  - `GenClient()`: Generates API client code
  - `SecretsRefresh()`: Updates secret values
  - `GenWrappers()`: Generates user-facing wrappers

**Streaming Protocol**: `cli/daemon/daemon.go:244-344`
- `CommandMessage` types: `Output` (stdout/stderr), `Exit` (code), `Error` (errlist)
- Buffering: `streamLog` buffers output until `FlushBuffers()` called
- Ordering: Mutex-protected writes, sequential `Send()` calls

### Dashboard WebSocket

**Path**: `/__encore` (`cli/daemon/dash/server.go:79-120`)
- Protocol: JSON-RPC 2.0 over WebSocket
- Connection log: `"dash: websocket connection established"` (line 98)
- Handles: app queries, run status, trace subscriptions, notifications

---

## 3. Retry/Backoff Logic for Listeners

### Implementation: `retryingTCPListener`

**Location**: `cli/cmd/encore/daemon/daemon.go:499-600`

**Key Struct**:
```go
type retryingTCPListener struct {
    component     string
    addr          netip.AddrPort
    ctx           context.Context
    cancel        func()
    doneListening chan struct{}  // closed when listener ready or failed
    underlying    net.Listener
    listenErr     error
}
```

**Retry Strategy** (`cli/cmd/encore/daemon/daemon.go:573-600`):
- **Backoff**: Exponential (`backoff.NewExponentialBackOff()`)
  - Initial: 50ms
  - Max: 500ms
  - MaxElapsed: 5 seconds
- **Retry Loop**: `backoff.Retry()` with `net.Listen("tcp", addr)`
- **Logging**: 
  - Error: `"unable to listen, retrying"` (line 590) - logged on each retry
  - Success: `"listening on port"` (line 598)
  - Failure: `"unable to listen, giving up"` (line 596)

**Critical Behavior**:
- **Accept() blocks until listener ready**: `cli/cmd/encore/daemon/daemon.go:529-539`
  - Waits on `doneListening` channel
  - Returns `listenErr` if binding failed
  - Returns `net.ErrClosed` if context canceled
- **"serving X" can log before "listening on port X"**: 
  - `serveDash()` logs at line 285, but listener may still be retrying
  - `http.Serve()` blocks on `Accept()`, so actual serving waits for listener
- **Process continues serving other components**: Each listener retries independently in background goroutine

**Edge Cases**:
- Context cancellation: Returns `backoff.Permanent(err)` to stop retrying
- Port conflicts: Retries up to 5 seconds, then gives up (non-fatal for daemon)
- Socket removal: Unix socket removed if exists before binding (line 212-214)

---

## 4. Readiness and Health Semantics

### What "Ready" Means

**Daemon Ready**:
- Unix socket exists and accepts connections
- `Version()` RPC responds successfully
- **Check**: `cli/cmd/encore/cmdutil/daemon.go:23-38`
  - Socket exists: `xos.SocketStat(socketPath)`
  - Socket responsive: `dialDaemon()` with 500ms timeout
  - Version check: `Version()` RPC call

**Component Ready**:
- **TCP listeners**: `doneListening` channel closed AND `listenErr == nil`
- **HTTP servers**: Accepting connections (implicit, no explicit health check)
- **Database**: Migrations complete, SQLite WAL mode enabled

**App Tracking Ready**:
- App registered in SQLite `app` table
- Log: `"tracking app app_id=..."` (`cli/daemon/apps/apps.go:85`)

**Run Ready**:
- `Run.started` channel closed (`cli/daemon/run/run.go:298`)
- Process group started: `ProcGroup.Start()` completed
- Gateway listening: `pollUntilProcessIsListening()` returns true (for reloads)

### Health Probes

**No explicit health endpoint**: Components don't expose `/health` endpoints
- **Implicit health**: Ability to accept connections
- **Version check**: `Version()` RPC is the primary health probe
- **Socket staleness**: Detected via `detectSocketClose()` polling (200ms interval)

---

## 5. Shutdown Semantics

### Signal Handling

**Entrypoint**: `cli/cmd/encore/daemon/daemon.go:72-94`
- **Signal**: `syscall.SIGINT` only (SIGTERM not explicitly handled)
- **Context cancellation**: `signal.NotifyContext()` cancels on SIGINT
- **Exit channel**: Subsystems send errors to `exit` channel

### Shutdown Sequence

1. **Context Cancellation**: All subsystems receive `ctx.Done()`
2. **Close All Resources**: `d.closeAll()` (`cli/cmd/encore/daemon/daemon.go:447-451`)
   - Closes: Unix listener, TCP listeners, SQLite DB, Apps manager
3. **Listener Close**: `retryingTCPListener.Close()` (`cli/cmd/encore/daemon/daemon.go:541-551`)
   - Cancels context
   - Closes underlying listener if ready
4. **Socket Cleanup**: Unix socket removed on close (default behavior)
   - Override: `SetUnlinkOnClose(false)` if socket changed externally

### Run Shutdown

**Process Group**: `cli/daemon/run/proc_groups.go:148-163`
- **Close()**: Sends `os.Interrupt` to all processes
- **Graceful timeout**: 10 seconds (`gracefulShutdownTime`)
- **Kill on timeout**: `Kill()` if not exited
- **WaitGroup**: Waits for all processes to exit

**Run Teardown**: `cli/daemon/run/run.go:217-228`
- Closes builder, removes temp dir, closes SvcProxy, stops ResourceManager
- **ResourceManager.StopAll()**: Stops SQL, PubSub, Redis, Objects servers

**HTTP Server Shutdown**: `cli/daemon/run/run.go:307-316`
- `srv.Close()` on context cancellation
- Error handling: Ignores `http.ErrServerClosed`

### Socket Close Detection

**Implementation**: `cli/cmd/encore/daemon/daemon.go:409-441`
- **Polling**: 200ms interval, checks socket inode
- **Detection**: Socket removed OR inode changed
- **Exit**: Sends nil error to `exit` channel (graceful)

### Goroutine Lifecycles

- **Background watchers**: Canceled via context
- **Trace cleanup**: `CleanEvery()` exits on `ctx.Done()` (`cli/daemon/engine/trace2/sqlite/write.go:50-61`)
- **App watchers**: Closed via `Instance.Close()` → `watcher.Close()`
- **Run monitors**: Exit when `Run.Done()` channel closes

### Known Shutdown Issues

- **"use of closed network connection"**: Handled in `cli/internal/jsonrpc2/serve.go:135` (ignored)
- **Race on socket removal**: External removal detected via polling, but daemon may not exit immediately
- **Process group races**: Multiple `Close()` calls safe (idempotent via `sync.Once`-like patterns)

---

## 6. App Watch + Codegen Pipeline

### App Discovery

**Tracking**: `cli/daemon/apps/apps.go:66-87`
- **Entrypoint**: `Manager.Track(appRoot)`
- **Database**: Inserts/updates `app` table (root, local_id, platform_id, updated_at)
- **Log**: `"tracking app app_id=..."` (line 85)
- **Resolution**: Reads `encore.app` file, parses platform ID

### File Watching

**Setup**: `cli/daemon/apps/apps.go:434-478`
- **Watcher**: `watcher.New()` per app instance
- **Watched paths**: App root (`i.root`) + runtime path (if dev mode)
- **Events**: `watcher.WaitForEvents()` returns `[]watcher.Event`
- **Distribution**: Events sent to manager watchers + instance watchers

**Debouncing**: `cli/daemon/watch.go:40-95`
- **Strategy**: `debounce.New(100 * time.Millisecond)`
- **Implementation**: `regenerateCodeDebouncer`
  - Tracks `running` state to prevent concurrent runs
  - `runAfter` flag ensures at least one run after events stop
  - Re-runs while events continue

**Event Filtering**: `cli/daemon/watch.go:40-43`
- **Ignore**: `run.IgnoreEvents()` filters out irrelevant changes
- **Examples**: Build artifacts, generated files

### Code Generation

**Trigger**: `cli/daemon/watch.go:97-103`
- **Function**: `regenerateUserCode()` → `genUserFacing()`
- **Success log**: `"successfully generated user code"` (line 101)
- **Failure log**: `"failed to regenerate app"` (line 99)

**Generation Process**: `cli/daemon/userfacing.go:30-78`
1. **Parse**: `bld.Parse()` with build info
2. **Cache metadata**: `app.CacheMetadata(parse.Meta)`
3. **Generate**: `bld.GenUserFacing()` (language-specific)
   - **Go**: Generates `encore.gen.go` files
   - **TypeScript**: Generates `encore.gen.ts` files
   - **CUE**: Generates `encore.gen.cue` files

**Node/NPM Usage**: 
- **TypeScript apps**: Builder invokes `node`/`npm` during `GenUserFacing()`
- **Working dir**: App root
- **Env**: Inherits daemon environment, may set `ENCORE_RUNTIME_LIB`

**Gitignore Update**: `cli/daemon/watch.go:105-143`
- **Directives added**: `encore.gen.go`, `encore.gen.cue`, `/.encore`, `/encore.gen`
- **Idempotent**: Only adds missing directives

### Watcher Lifecycle

**Start**: `apps.Instance.beginWatch()` (`cli/daemon/apps/apps.go:434-478`)
- **Once**: `syncutil.Once` ensures single watcher per instance
- **Goroutine**: Watches events, distributes to listeners
- **Close**: `Instance.Close()` → `watcher.Close()`

**Manager Watchers**: `cli/daemon/apps/apps.go:196-210`
- **Registration**: `Manager.WatchAll(fn)` adds to watchers list
- **Invocation**: `onWatchEvent()` calls all registered watchers
- **App tracking**: `List()` resolves all apps, triggers watcher setup

---

## 7. Run Pipeline (Tracking → Build → Run → Teardown)

### Pipeline Overview

**Entrypoint**: `cli/daemon/run.go:25-268` (gRPC `Run()` method)

**Sequence**:
1. **Track App**: `s.apps.Track(req.AppRoot)`
2. **Resolve Namespace**: `s.namespaceOrActive()`
3. **Create Listener**: `net.Listen("tcp", listenAddr)`
4. **Start Run**: `s.mgr.Start()` → `Run.start()`
5. **Build & Start**: `buildAndStart()` → `StartProcGroup()`
6. **Monitor**: Wait on `runInstance.Done()`
7. **Teardown**: `runInstance.Close()`

### Build Jobs (optracker)

**Creation**: `cli/daemon/run/run.go:352`
- **Type**: `optracker.NewAsyncBuildJobs(ctx, appID, tracker)`
- **Implementation**: `internal/optracker/async.go:11-87`

**Job Lifecycle**:
- **Start**: `jobs.Go(description, track, minDuration, f)`
  - Log: `"starting build job"` (line 49)
  - Spawns goroutine with `wait.Add(1)`
- **Completion**: 
  - Success: `tracker.Done(trackerID, minDuration)` (line 65)
  - Failure: `tracker.Fail(trackerID, err)` (line 59)
  - Log: `"build job finished"` or `"build job failed"` (lines 67, 57)
- **Error handling**: First error cancels context, stored in `firstError`
- **Wait**: `jobs.Wait()` blocks until all jobs complete

**Job Types** (`cli/daemon/run/run.go:415-458`):
1. **Parse**: "Building Encore application graph"
2. **Topology**: "Analyzing service topology"
3. **Compile**: "Compiling application source code"
4. **Secrets**: "Fetching application secrets" (150ms min duration)
5. **Infra**: "Creating PostgreSQL database cluster" (300ms min)
6. **Migrations**: "Running database migrations" (250ms min)

### Infrastructure Setup

**ResourceManager**: `cli/daemon/run/infra/infra.go:94-112`
- **StartRequiredServices()**: Checks metadata, starts needed services
- **SQL Cluster**: `StartSQLCluster()` → `ClusterManager.Create()` → `Cluster.Start()`
- **PubSub**: `StartPubSub()` → NSQ daemon
- **Redis**: `StartRedis()` → Redis server
- **Objects**: `StartObjects()` → Object storage server

**SQL Cluster Lifecycle**: `cli/daemon/sqldb/db.go:207-259`
- **Create**: `ClusterManager.Create()` with cluster ID
- **Start**: `Cluster.Start()` → Docker container or external connection
- **Setup**: `SetupAndMigrate()` → Creates DBs, runs migrations
- **Namespace**: Type `sqldb.Run` for normal runs, `sqldb.Test` for tests

**Migrations**: `cli/daemon/sqldb/db.go:256-314`
- **Logs**: 
  - `"database already up to date"` (line 307) - `migrate.ErrNoChange`
  - `"migration completed"` (line 312) - success
- **Dirty migration handling**: Resets dirty flag, re-applies (lines 362-388)
- **Non-sequential**: Supports `AllowNonSequentialMigrations` flag

### Process Groups

**Creation**: `cli/daemon/run/run.go:465-717`
- **All-in-one**: Single process for all services (`isSingleProc()` check)
- **Per-service**: Multiple processes, one per service/gateway
- **Config generation**: `RuntimeConfigGenerator` creates runtime config files

**Start**: `cli/daemon/run/proc_groups.go:131-145`
- **Sequential start**: Locks `procMu`, starts all processes
- **Error handling**: Kills started processes on error
- **Noop gateway**: Created if no gateway processes

**Process Lifecycle**: `cli/daemon/run/proc_groups.go:388-447`
- **Start**: `cmd.Start()`, logs `"process started"` (line 409)
- **Monitor**: Goroutine waits on `cmd.Wait()`
- **Exit**: 
  - Success: `"process exited successfully"` (line 424)
  - Error: `"process exited with error"` (line 422)
- **Cleanup**: Decrements `runningProcs`, broadcasts `procCond`

**Reload Behavior**: `cli/daemon/run/run.go:702-714`
- **Wait for readiness**: `pollUntilProcessIsListening()` for all gateways
- **Backoff**: 50ms initial, 250ms max, 5s timeout
- **Check**: TCP dial to `listenAddr`

### Serving App Ports

**HTTP Server**: `cli/daemon/run/run.go:301-316`
- **Handler**: `h2c.NewHandler(r, &http2.Server{})` (HTTP/2 cleartext)
- **Serve**: `http.Serve(ln, handler)` in goroutine
- **Shutdown**: `srv.Close()` on context cancellation
- **Error**: `http.ErrServerClosed` ignored

**Proxy Setup**: `cli/daemon/run/proc_groups.go:203-239`
- **Reverse proxy**: `httputil.ReverseProxy` per process
- **Transport**: `transport.NewH2CTransport()` for HTTP/2
- **Auth key**: Added to requests (unless test header set)

### Teardown on Failure

**Build Failure**: `cli/daemon/run/run.go:185-190`
- **Deferred cleanup**: `defer func() { if err != nil { r.Close() } }`
- **ResourceManager.StopAll()**: Stops all infra services
- **Process kill**: `p.Kill()` if proc started but build failed (line 688)

**Run Exit**: `cli/daemon/run/run.go:318-336`
- **Monitor**: Goroutine watches `ProcGroup.Done()`
- **Verification**: Checks `proc.Load()` to ensure proc still active (reload race)
- **Listeners**: Calls `OnStop(r)` on all listeners
- **Channel close**: `close(r.exited)`

**Error Propagation**: `cli/daemon/run.go:153-166`
- **Error list**: `run.AsErrorList(err)` sends to stream
- **Exit code**: `sendExit(1)` on failure
- **Stream cleanup**: Removes from `s.streams` map

---

## 8. Minimum Portable Contract for Rust

### Must Replicate

1. **Unix Socket Management**
   - Path: `~/Library/Caches/encore/encored.sock` (macOS)
   - Behavior: Remove if exists, bind, detect external removal
   - Protocol: gRPC `daemonpb.DaemonServer`

2. **TCP Listener Retry Logic**
   - Exponential backoff: 50ms → 500ms, 5s max
   - Non-blocking: Accept() waits on readiness channel
   - Logging: Retry errors, success, failure

3. **Component Serving**
   - Dashboard (9400): HTTP + WebSocket `/__encore`
   - DBProxy (9500): HTTP proxy to SQL clusters
   - Runtime (9600): HTTP trace ingestion
   - Debug (9700): HTTP pprof endpoints
   - ObjectStorage (9800): HTTP object storage API
   - MCP (9900): HTTP/SSE MCP endpoint

4. **App Tracking**
   - SQLite database: `app` table (root, local_id, platform_id, updated_at)
   - File watching: Watch app root + runtime (dev mode)
   - Code generation: Trigger on file changes (debounced 100ms)

5. **Run Pipeline**
   - Build jobs: Async execution with optracker
   - Infrastructure: SQL, PubSub, Redis, Objects
   - Process groups: All-in-one or per-service
   - HTTP serving: H2C support, reverse proxy

6. **Shutdown**
   - Signal handling: SIGINT → context cancellation
   - Graceful: 10s timeout for processes
   - Resource cleanup: Close all listeners, stop infra

### Can Delegate

1. **Code Generation**: Delegate to existing Go builder via gRPC or subprocess
2. **SQL Migrations**: Use existing `golang-migrate` via FFI or subprocess
3. **Process Execution**: Use Rust `std::process::Command` (equivalent to Go `exec.Command`)
4. **File Watching**: Use `notify` crate (equivalent to Go `watcher`)
5. **gRPC Server**: Use `tonic` (Rust gRPC) with same proto definitions
6. **HTTP Servers**: Use `axum` or `hyper` (equivalent to Go `http.Server`)

### Interface Requirements

**gRPC Client to daemonpb**:
- `Version()`: Health check
- `Run()`: Streaming RPC for app execution
- `GenClient()`: Client code generation
- `SecretsRefresh()`: Secret updates
- `GenWrappers()`: Wrapper generation

**MCP Interface**:
- SSE endpoint: `/sse?appID=...`
- Event streaming: App status, run events, traces

**Dashboard Interface**:
- WebSocket: `/__encore` (JSON-RPC 2.0)
- GraphQL proxy: `/__graphql`

---

## 9. Rust-Facing Interfaces

### gRPC Client

**Proto**: `proto/encore/daemon/daemon.proto`
- **Client**: `tonic` with `daemonpb` generated code
- **Connection**: Unix socket dialer
- **Streaming**: `Streaming<CommandMessage>` for `Run()` RPC

### MCP Server

**Protocol**: Server-Sent Events (SSE)
- **Endpoint**: `http://localhost:9900/sse?appID=...`
- **Events**: App status, run lifecycle, trace events
- **Implementation**: `cli/daemon/mcp/` (existing Go code can be referenced)

### Dashboard

**WebSocket**: JSON-RPC 2.0
- **Path**: `/__encore`
- **Methods**: App queries, run control, trace subscriptions
- **Implementation**: `cli/daemon/dash/` (reference for method signatures)

### File System

**Paths**:
- Cache dir: `os::UserCacheDir() + "/encore"`
- Socket: `cache_dir + "/encored.sock"`
- Log: `cache_dir + "/daemon.log"`
- DB: `conf.Dir() + "/encore.db"` (SQLite)

**Environment Variables**:
- `ENCORE_RUNTIMES_PATH`: Runtime library path
- `ENCORE_DAEMON_LOG_PATH`: Override log path
- `ENCORE_DEVDASH_LISTEN_ADDR`: Dashboard address override
- `ENCORE_MCPSSE_LISTEN_ADDR`: MCP address override
- `ENCORE_OBJECTSTORAGE_LISTEN_ADDR`: Object storage address override

---

## 10. Risk List: Footguns, Races, Known Bugs

### Log Ordering

**Issue**: "serving X" can log before "listening on port X"
- **Cause**: `serveDash()` logs immediately, but `retryingTCPListener` may still be retrying
- **Impact**: Misleading logs, but `http.Serve()` blocks on `Accept()`, so actual serving waits
- **Location**: `cli/cmd/encore/daemon/daemon.go:284-288` vs `573-600`

### Port Conflicts

**Issue**: Multiple daemon instances can race on port binding
- **Mitigation**: Retry logic (5s max), but daemon continues if one component fails
- **Risk**: Partial startup if some ports unavailable
- **Location**: `cli/cmd/encore/daemon/daemon.go:584-593`

### Socket Staleness

**Issue**: Stale socket file if daemon crashes
- **Detection**: `xos.SocketStat()` + dial test
- **Cleanup**: `os.Remove(socketPath)` if not responsive
- **Location**: `cli/cmd/encore/cmdutil/daemon.go:23-38`

### Atomic Nil Hazards

**Issue**: `ProcGroup` may be nil during reload
- **Mitigation**: `atomic.Value` for proc storage, nil checks
- **Location**: `cli/daemon/run/run.go:320-326` (proc.Load() check)

### Context Cancellation Races

**Issue**: Context canceled while operations in flight
- **Mitigation**: `ctx.Err()` checks before operations
- **Risk**: Partial cleanup if context canceled mid-operation
- **Location**: Multiple (e.g., `cli/daemon/run/run.go:343-346`)

### Process Group Races

**Issue**: Multiple `Close()` calls or reload during shutdown
- **Mitigation**: `sync.Mutex` on `procMu`, `atomic.Bool` for `Started`
- **Location**: `cli/daemon/run/proc_groups.go:148-163`, `398-447`

### Migration Dirty Flag

**Issue**: Dirty migration state can block startup
- **Mitigation**: Auto-reset dirty flag, re-apply migration
- **Risk**: Data loss if migration partially applied
- **Location**: `cli/daemon/sqldb/db.go:360-388`, `cli/cmd/encore/daemon/daemon.go:384-404`

### "Use of Closed Network Connection"

**Issue**: HTTP server closed while handling request
- **Mitigation**: Ignore `http.ErrServerClosed`, handle in JSON-RPC layer
- **Location**: `cli/daemon/run/run.go:309`, `cli/internal/jsonrpc2/serve.go:135`

### Watcher Resource Limits

**Issue**: Too many file watchers (macOS limit)
- **Mitigation**: `watcher.BumpRLimitSoftToHardLimit()` at startup
- **Location**: `cli/cmd/encore/daemon/daemon.go:62`

### Secret Loading Race

**Issue**: Secrets loaded asynchronously, may not be ready
- **Mitigation**: `secrets.Get(ctx, expSet)` with context timeout
- **Risk**: Build fails if secrets unavailable
- **Location**: `cli/daemon/run/run.go:447-454`

### Trace Cleanup Locking

**Issue**: Concurrent cleanup and writes
- **Mitigation**: SQLite WAL mode, but no explicit locking
- **Risk**: Potential data races (low, SQLite handles most)
- **Location**: `cli/daemon/engine/trace2/sqlite/write.go:50-122`

### Reload Readiness Race

**Issue**: New proc may not be listening when old proc killed
- **Mitigation**: `pollUntilProcessIsListening()` waits up to 5s
- **Risk**: Brief downtime if proc slow to start
- **Location**: `cli/daemon/run/run.go:704-713`

---

## Call Graphs

### Boot Sequence

```
cli/cmd/encore/daemon.go:Run()
  → daemonpkg.Main()
    → redirectLogOutput()
      → env.EncoreDaemonLogPath()
    → runMain()
      → signal.NotifyContext()
      → d.init()
        → d.listenDaemonSocket()
        → d.listenTCPRetry() [×6]
        → d.openDB()
        → apps.NewManager()
        → sqldb.NewClusterManager()
        → run.NewManager()
        → mcp.NewManager()
        → daemon.New()
      → d.serve()
        → go d.serveDaemon()
        → go d.serveRuntime()
        → go d.serveDBProxy()
        → go d.serveDash()
        → go d.serveDebug()
        → go d.serveObjects()
        → go d.serveMCP()
      → select { exit, ctx.Done() }
```

### Run Pipeline

```
cli/daemon/run.go:Run()
  → s.apps.Track()
  → s.mgr.Start()
    → run.start()
      → run.buildAndStart()
        → optracker.NewAsyncBuildJobs()
        → r.Builder.Parse()
        → r.ResourceManager.StartRequiredServices()
        → jobs.Go("Compiling...")
        → jobs.Go("Fetching secrets...")
        → jobs.Wait()
        → r.StartProcGroup()
          → newProcGroup()
          → p.NewAllInOneProc() OR p.NewProcForService()
          → p.Start()
            → cmd.Start()
            → go cmd.Wait()
      → http.Serve(ln, handler)
      → go monitor proc.Done()
```

### App Watching

```
cli/daemon/daemon.go:New()
  → go srv.watchApps()
    → s.apps.RegisterAppListener()
    → s.apps.WatchAll()
      → apps.Instance.beginWatch()
        → watcher.New()
        → watcher.RecursivelyWatch()
        → go watcher.WaitForEvents()
          → s.onWatchEvent()
            → regenerateCodeDebouncer.ChangeEvent()
              → s.regenerateUserCode()
                → s.genUserFacing()
                  → bld.Parse()
                  → bld.GenUserFacing()
```

---

## Configuration Sources

### Environment Variables

- `ENCORE_RUNTIMES_PATH`: `internal/env/env.go:57-67`
- `ENCORE_DAEMON_LOG_PATH`: `internal/env/env.go:80-91`
- `ENCORE_DEVDASH_LISTEN_ADDR`: `internal/env/env.go:93-101`
- `ENCORE_MCPSSE_LISTEN_ADDR`: `internal/env/env.go:103-111`
- `ENCORE_OBJECTSTORAGE_LISTEN_ADDR`: `internal/env/env.go:113-118`
- `ENCORE_SQLDB_HOST`: External SQL override (`cli/cmd/encore/daemon/daemon.go:143-151`)
- `ENCORE_DAEMON_WATCH`: Disable watching if `"0"` (`cli/daemon/watch.go:24-26`)

### File Paths

- **Cache dir**: `os.UserCacheDir() + "/encore"` (macOS: `~/Library/Caches/encore`)
- **Socket**: `cache_dir + "/encored.sock"`
- **Log**: `cache_dir + "/daemon.log"` (or `ENCORE_DAEMON_LOG_PATH`)
- **DB**: `conf.Dir() + "/encore.db"` (typically `cache_dir + "/encore.db"`)

### Default Ports

- Dashboard: 9400
- DBProxy: 9500
- Runtime: 9600
- Debug: 9700
- ObjectStorage: 9800
- MCP: 9900

---

## Conclusion

This analysis provides a complete, source-grounded understanding of the Encore daemon's boot sequence, component serving, app tracking, build/run pipeline, and shutdown semantics. Every behavior is mapped to specific source locations, enabling faithful replication in Rust within AxiomRegent.

**Key Takeaways**:
1. Boot is mostly synchronous except for TCP listener retries
2. Retry logic is critical for port conflicts during handoff
3. App watching triggers codegen with 100ms debounce
4. Run pipeline is highly concurrent with async build jobs
5. Shutdown is graceful with 10s timeout for processes
6. Many behaviors can be delegated to existing Go components via gRPC/subprocess

**Next Steps for Rust Port**:
1. Implement Unix socket + gRPC server (tonic)
2. Implement retrying TCP listeners with same backoff
3. Delegate codegen to Go builder (keep existing)
4. Implement process groups with Rust `std::process`
5. Port file watching to `notify` crate
6. Keep SQLite for app tracking (use `rusqlite`)
