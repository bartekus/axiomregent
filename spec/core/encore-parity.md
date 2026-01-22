A lot of the “stuff you don’t want to reimplement in Rust” is exactly the Go daemon’s local-dev substrate. So yes: carving it out, forking it, and treating it as a “local emulator daemon” that your Rust supervisor spawns is a pretty clean strategy.

Here’s how I’d think about it, mapped to what’s actually hard/expensive vs. what’s already “yours” in Rust for TS.

What’s worth keeping in Go (and can be “lifted” into a local emulator daemon)

These are the parts that are mostly orchestration-heavy, stateful, and already integrated with Encore’s internal expectations:

1) App discovery + tracking + state DB (Go-owned today)
	•	Workspace/app detection, app identity, version tracking
	•	SQLite-backed tracking and any internal caching of “what’s running”
	•	This tends to be glue-code + persistence + edge cases (lots of yak shaving in Rust)

Liftability: high.
Reason to lift: it’s not your product differentiator, but it’s brittle to rewrite.

2) File watching + debounce + “what changed?” logic (Go-owned today)
	•	Cross-platform file watching, ignore rules, debouncing rebuilds
	•	Change classification (what triggers codegen vs rebuild vs restart)

Liftability: high.
Reason to lift: file-watch correctness is a time sink; Go version is battle-tested.

3) Infra resource bring-up for local dev (Go-owned today)
	•	Starting local Postgres/Redis/PubSub/Object emulators (or whatever Encore’s local stack is)
	•	Wiring credentials, ports, volumes, lifecycle, readiness

Liftability: very high.
Reason to lift: this is the most “emulator daemon” shaped subsystem.

4) Log streaming + event surfaces (Go-owned today)
	•	Aggregating logs from components, providing structured events
	•	Multiplexing to CLI/UI consumers

Liftability: high.
Reason to lift: it’s integration-heavy and changes tend to ripple.

What you already “have” in Rust (so keep Rust as the truth source)

These are the TS-specific parts where Rust is already real value and/or already exists:

1) TS parsing / metadata graph
	•	Rust tsparser-encore is already the right locus of truth.

2) TS codegen outputs (encore.gen/**)
	•	Again, Rust is already doing the heavy lifting.

3) Process lifecycle supervision (your Rust supervisor)
	•	Start/stop/restart, backoff, port leasing, readiness checks, proxying. This is your “axiomregent substrate” value.

So the natural split is:
	•	Go “local emulator daemon”: watches workspace, manages local infra emulators, maintains local runtime state/event streaming, and invokes the build pipeline orchestration it already knows.
	•	Rust supervisor: owns lifecycle of the Go daemon + the actual app runtime processes you care about, and presents the MCP-friendly control plane.

How much of the “don’t reimplement” bucket is realistically Go-liftable?

If you define “don’t reimplement” as the stuff that’s mostly orchestration + local infra + file watching + state + log/event multiplexing, then most of it is in that Go-owned set above.

The remaining “don’t reimplement” items tend to be:
	•	“Encore-internal contracts that are easier to call than to recreate”
	•	“Odd edge-case logic spread across CLI/daemon packages”

Those also tend to be liftable as-is if you keep the Go daemon intact but “rebrand” it as the local emulator daemon.

A concrete shape for the fork

You get the cleanest boundary if you treat the lifted Go code as a daemon with a small, stable API:

Go local emulator daemon responsibilities
	•	EnsureInfra(app_id) -> ports/creds/paths
	•	Watch(app_root) -> change events
	•	BuildPlan(change_set) -> steps
	•	StreamLogs(app_id) -> stream
	•	Health()

Rust supervisor responsibilities
	•	spawn/kill the Go daemon
	•	own port leasing policy for your world (or delegate fully to Go, but pick one)
	•	translate Go events into MCP tool responses
	•	keep Encore Rust crates untouched (no endpoint/readiness layers added inside them)

This also matches your concern: keep Encore’s Rust code “pure”, and do adaptation in the wrapper layer.

The main decision: who is the “orchestrator of record”?

You need to pick one place where “what is running” is decided:

Option A (usually best): Rust is the boss
	•	Rust supervisor is the source of truth.
	•	Go daemon provides infra + events + helpers.
	•	Rust decides what processes exist and when they restart.

Option B: Go remains the boss (Rust becomes a thin launcher)
	•	Fastest in the short term.
	•	But you’ll constantly fight “two control planes” once MCP tooling grows.

Given AxiomRegent’s goals, Option A aligns better.

The big risks to watch
	•	Double port ownership: if both Go and Rust assign ports, you will get heisenbugs. Make one authoritative.
	•	Duplicated readiness/proxy layers: same story - pick one.
	•	Tight coupling to Go daemon internals: keep a narrow API boundary so you can later peel pieces off into Rust incrementally.

My recommendation

Yes: lift the Go-owned local substrate into a renamed “local emulator daemon” that you treat as an internal dependency, spawned and supervised by Rust. Keep Rust as the control plane and keep Encore Rust crates untouched.

Minimal API contract: Rust supervisor ↔ Go “local emulator daemon”

Transport + framing
	•	JSON-RPC 2.0 over stdio (best for “spawned child process”).
	•	Rust spawns Go process, wires:
	•	stdin for requests
	•	stdout for JSON-RPC responses + notifications
	•	stderr reserved for logs (never protocol)
	•	Add a simple “hello” handshake so Rust can fail fast if versions drift.

Versioning + compatibility rules
	•	Every request includes client + protocol_version.
	•	Go daemon advertises capabilities and daemon_version.
	•	Backward compat rule: only add fields (optional), never remove/rename.
	•	Deprecations: keep accepting old fields for N releases; return a warning in meta.

Core concepts
	•	Workspace: root path + optional app selector
	•	AppId: stable string (hash or module path)
	•	LeaseId: opaque token for ports/resources lifetimes
	•	RunId: identifies a running dev session
	•	ResourceGraph: local infra instances + creds + endpoints
	•	ChangeSet: file changes + classification (codegen vs rebuild vs restart)

⸻

JSON-RPC surface (minimal but future-proof)

1) Handshake / introspection

Request
	•	daemon.hello

{
  "jsonrpc":"2.0",
  "id":1,
  "method":"daemon.hello",
  "params":{
    "protocol_version":"1.0",
    "client":{"name":"axiomregent","version":"0.1.0"},
    "workspace_root":"/path/to/ws"
  }
}

Response

{
  "jsonrpc":"2.0",
  "id":1,
  "result":{
    "protocol_version":"1.0",
    "daemon":{"name":"encore-local-emulator-daemon","version":"X.Y.Z"},
    "capabilities":{
      "watch":true,
      "infra":true,
      "build_orchestrate":true,
      "logs":true
    }
  }
}

2) Workspace/app discovery (optional but useful)
	•	workspace.scan → list apps and metadata
	•	app.resolve → turns a path/module into an app_id

app.resolve

{"jsonrpc":"2.0","id":2,"method":"app.resolve","params":{"path":"/path/to/app"}}

Returns:

{"jsonrpc":"2.0","id":2,"result":{"app_id":"app_123","module_path":"...","root":"/path/to/app"}}

3) Infra: ensure local emulators are up

This is the “big win” service Go keeps initially.
	•	infra.ensure (idempotent)
	•	inputs: app_id, optional run_id, optional requested resources
	•	outputs: infra graph (endpoints/ports/creds), plus a lease_id

{
  "jsonrpc":"2.0","id":3,"method":"infra.ensure",
  "params":{"app_id":"app_123","requested":{"sql":true,"redis":true,"pubsub":true,"objects":true}}
}

Response:

{
  "jsonrpc":"2.0","id":3,
  "result":{
    "lease_id":"lease_abc",
    "infra":{
      "gateways":[{"name":"runtime","host":"127.0.0.1","port":9600}],
      "sql":[{"rid":"sql1","host":"127.0.0.1","port":54321,"database":"encore","user":"...","password":"..."}],
      "redis":[{"rid":"redis1","host":"127.0.0.1","port":63790}],
      "objects":[{"rid":"obj1","endpoint":"http://127.0.0.1:9800"}]
    }
  }
}

	•	infra.release to drop a lease (optional; Rust can also just kill daemon).

4) Watch: stream changes as notifications
	•	watch.start → returns watch_id
	•	daemon sends notifications: watch.changed

Request:

{"jsonrpc":"2.0","id":4,"method":"watch.start","params":{"app_id":"app_123","root":"/path/to/app"}}

Notification:

{
  "jsonrpc":"2.0",
  "method":"watch.changed",
  "params":{
    "watch_id":"watch_1",
    "app_id":"app_123",
    "changes":[{"path":"svc/foo.ts","kind":"modified"}],
    "classification":{"needs_codegen":true,"needs_rebuild":true,"needs_restart":false}
  }
}

5) Build orchestration (initially Go-owned, later replaceable)

Even if Rust is “boss”, you can let Go propose the plan.
	•	build.plan → returns ordered steps + artifacts to produce
	•	build.run → executes (or you keep build.plan only and Rust executes steps)

Minimal:

{"jsonrpc":"2.0","id":5,"method":"build.plan","params":{"app_id":"app_123","changeset_id":"cs_77"}}

Response:

{
  "jsonrpc":"2.0","id":5,
  "result":{
    "plan_id":"plan_88",
    "steps":[
      {"kind":"tsparse","cmd":"tsparser-encore","args":["parse", "..."],"inputs":["..."],"outputs":["meta.pb"]},
      {"kind":"codegen","cmd":"tsparser-encore","args":["gen","..."],"outputs":["encore.gen/**"]},
      {"kind":"bundle","cmd":"node","args":["..."],"outputs":["dist/**"]}
    ]
  }
}

6) Logs/events (optional but very useful)
	•	logs.subscribe → daemon emits notifications logs.line
	•	Or daemon can expose “event stream” for infra state too

Request:

{"jsonrpc":"2.0","id":6,"method":"logs.subscribe","params":{"app_id":"app_123"}}

Notification:

{"jsonrpc":"2.0","method":"logs.line","params":{"app_id":"app_123","source":"infra/sql1","line":"ready on 54321"}}

7) Health and shutdown
	•	daemon.health
	•	daemon.shutdown (graceful)

⸻

Protobuf option (when you outgrow JSON)

If you later need strict typing + faster streaming:
	•	Keep JSON-RPC as the control plane
	•	For large payloads (infra graphs, plans), add:
	•	...getBytes methods that return a blob_id
	•	then a separate “blob fetch” method that returns base64 or length-delimited protobuf
This preserves the same logical API while changing payload encoding.

⸻

Step-down plan: replace Go subsystems one-by-one without breaking the interface

The trick is: the interface stays stable; only the implementation behind methods changes.

Phase 0: Establish the boundary (now)

Goal: Rust supervisor spawns Go daemon; Rust is the boss.
	•	Implement: daemon.hello, infra.ensure, watch.start, logs.subscribe, daemon.shutdown
	•	Rust owns:
	•	process lifecycle (restart/backoff)
	•	protocol wiring
	•	canonical “RunId” concept
	•	Go owns:
	•	infra bring-up + tracking
	•	file watching
	•	log/event multiplexing (at least for infra)

Deliverable: you can run apps reliably with minimal Rust surface area.

Phase 1: Replace “watch” first (lowest risk, high payoff)

Swap implementation of watch.start:
	•	Keep the method name and notification schema identical.
	•	Rust starts its own watcher.
	•	Go daemon either:
	•	becomes a no-op for watch.start but still returns a watch_id, or
	•	advertises capability watch=false, and Rust uses its internal watcher.
Best practice: prefer capability flagging so you don’t need fake implementations.

Outcome: file-watching correctness becomes yours, but interface unchanged.

Phase 2: Replace “build.plan”/“build.run” next (TS pipeline control)

You likely want Rust to own build determinism.
	•	Keep build.plan shape the same.
	•	Initially, Go returns plans; later Rust returns plans.
	•	If consumers depend on plan step kinds, keep them stable:
	•	tsparse, codegen, bundle, restart, etc.

Outcome: build orchestration moves into Rust while leaving Go for infra.

Phase 3: Replace infra piece-by-piece (the big one)

Break infra.ensure into sub-resources without changing the public call.

Internally, have infra.ensure accept requested flags (already in schema), so Rust can gradually take over:
	•	Start with easiest:
	•	Object storage emulator
	•	Then:
	•	Redis
	•	Then:
	•	PubSub
	•	Last:
	•	SQL (usually the gnarliest)

Mechanism:
	•	Go daemon starts advertising per-resource capabilities:
	•	capabilities.infra.sql = true/false
	•	capabilities.infra.redis = ...
	•	Rust infra.ensure aggregator:
	•	calls internal Rust providers for what it owns
	•	calls Go daemon for what it doesn’t
	•	merges into the same infra response format

Outcome: to callers, infra.ensure still returns the same infra graph + lease_id.

Phase 4: Replace logs/event multiplexing (optional)

Once Rust owns most components, consolidate eventing:
	•	Keep logs.subscribe and logs.line notifications identical.
	•	Either:
	•	Rust becomes the source of truth and forwards Go logs when needed, or
	•	Go becomes purely legacy, and Rust emits everything.

Outcome: single event bus.

Phase 5: Retire Go daemon (or freeze it)

At this point, Go daemon can be:
	•	removed entirely, or
	•	left as a “compat mode” implementation behind the same interface (useful for bisecting issues).

⸻

Two design rules that make the step-down painless
	1.	Capabilities, not forks

	•	Always rely on daemon.hello.capabilities to decide who does what.
	•	Don’t hardcode “Go always watches” or “Rust always builds”.

	2.	Stable IDs + stable shapes

	•	Keep app_id, run_id, lease_id, watch_id, plan_id as opaque strings.
	•	Keep response object shapes stable; only add fields.

⸻

If you want, I can also give you:
	•	a tiny .proto that mirrors the JSON shapes (so Rust/Go can share types cleanly), and
	•	a concrete Rust trait layout like InfraProvider, WatchProvider, BuildPlanner, where Go is just one implementation behind each provider.