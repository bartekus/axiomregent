I’ll first map the TS parsing crate (`tsparser`), then follow its outputs into metadata/service graph, then locate TS codegen, then build/run paths and supervisor hooks, and finally extract env vars/files and synthesize the AxiomRegent wrapper contract. I’ll keep a running “Trail” as I hop between Rust, Node/TS, and supervisor code.

[5 tools called]

I’ve located the core TS parser crate and initial TypeScript-related references; next I’ll drill into `tsparser`’s entrypoints and outputs, then follow those into service discovery, codegen, and runtime/supervisor integration while building the requested trail and maps.

[5 tools called]

I’ve identified the core TS parser binary and its prepare/parse/compile/gen-user-facing commands; next I’ll trace how `Builder` and `ParseContext` use SWC and how metadata is encoded, then follow the compile/codegen paths into runtime and supervisor integration.

[42 tools called]

### Executive Trace Summary (TS-only)

- **TS parsing & metadata** are implemented by the Rust crate `encore-tsparser` (`tsparser/`), whose binary `tsparser-encore` is invoked by the Go `v2/tsbuilder` to parse TS apps, compute Encore metadata, and orchestrate TS codegen & bundling (`tsparser/src/bin/tsparser-encore.rs`, `v2/tsbuilder/tsbuilder.go`).  
- **SWC-based AST parsing** is encapsulated in `ModuleLoader` and `ParseContext` (`tsparser/src/parser/module_loader.rs`, `tsparser/src/parser/parser.rs`), using `Syntax::Typescript` and a `NodeModulesResolver` wrapped in `EncoreRuntimeResolver` to resolve `encore.dev/*` packages and TS path mappings (`tsparser/src/runtimeresolve/node.rs`).  
- **Service graph & metadata** are built entirely in Rust: `Parser::parse` walks all `.ts` files under the app root, discovers services/resources/usages, and `legacymeta::compute_meta` converts this into `encore.parser.meta.v1.Data` (prost message) used by the runtime (`tsparser/src/parser/parser.rs`, `tsparser/src/legacymeta/mod.rs`).  
- **Generated TS artifacts** live under `encore.gen/` in the app root and include internal entrypoints (per-service, per-gateway, combined) and client/auth catalogs; this is produced by `Builder::generate_code` using Handlebars templates (`tsparser/src/builder/mod.rs`, `tsparser/src/builder/codegen.rs`).  
- **encore.gen.ts (docs)** now corresponds to a **directory tree** `encore.gen/internal/...` and `encore.gen/clients/...` rather than a single file; generation is triggered via the `gen-user-facing` command over the `tsparser-encore` stdin protocol (`tsparser/src/bin/tsparser-encore.rs`, `v2/tsbuilder/tsbuilder.go`, `cli/daemon/userfacing.go` described in `docs/daemon_boot_analysis.md`).  
- **Node/TS build pipeline** uses a Rust-side `EsbuildCompiler` wrapper that shells out to `tsbundler-encore` (via `ENCORE_TSBUNDLER_PATH` or default) to bundle the generated entrypoints into `.mjs` artifacts under `.encore/build` (`tsparser/src/builder/compile.rs`, `tsparser/src/builder/transpiler.rs`).  
- **Runtime process specs** for TS are JSON-serialized `CompileResult` / `JSBuildOutput` / `CmdSpec` that describe how to invoke Node or Bun (with debugger flags) on the bundled `.mjs` entrypoints, including which Encore services/gateways each process hosts (`tsparser/src/builder/compile.rs`, `tsparser/src/builder/transpiler.rs`, `pkg/builder/builder.go`).  
- **Encore TS runtime** is a Node-native addon (`runtimes/js/`) that embeds the Rust `encore_runtime_core::Runtime`, initializes it from `ENCORE_RUNTIME_CONFIG` / `ENCORE_APP_META` env vars, and exposes JS APIs under `encore.dev/*` (`runtimes/js/src/runtime.rs`, `runtimes/core/src/lib.rs`).  
- **Runtime topology & routing** are computed entirely inside `encore_runtime_core` using the TS-generated `meta.v1.Data` and runtime config; this determines endpoints, gateways, auth handlers, pubsub, SQldb, objects, metrics, and which services/gateways are hosted by each process (`runtimes/core/src/lib.rs`, `runtimes/core/src/api/manager.rs`).  
- **Local dev watch → TS codegen** is driven by the Go daemon’s file watcher; on changes it runs the TS builder’s `GenUserFacing`, which in turn issues `gen-user-facing` to `tsparser-encore`, regenerating `encore.gen` and updating `.gitignore` (`cli/daemon/watch.go`, `docs/daemon_boot_analysis.md`, `v2/tsbuilder/tsbuilder.go`).  
- **Local dev build → run**: for TS apps, the daemon delegates `Parse`/`Compile`/`TestSpec`/`GenUserFacing` to `v2/tsbuilder.BuilderImpl`, which is a thin process bridge around `tsparser-encore`; the daemon then uses the resulting TS `JSBuildOutput` to start Node/Bun processes and proxy traffic to them (`v2/tsbuilder/tsbuilder.go`, `cli/daemon/run/run.go`, `pkg/builder/builder.go`).  
- **Rust supervisor (`supervisor-encore`)** is a separate, language-agnostic supervisor that reads `ENCORE_RUNTIME_CONFIG` / `ENCORE_INFRA_CONFIG_PATH` and a JSON `supervisor.config` to derive which services/gateways to run, then spawns processes with `ENCORE_PROCESS_CONFIG` env and supervises them with restart policies (`supervisor/src/config.rs`, `supervisor/src/bin/supervisor-encore.rs`, `supervisor/src/supervisor.rs`).  
- **TS vs Go**: TS uses the same meta/runtime protobufs and the same Rust runtime core as Go apps, but the parsing, bundling, and per-service/gateway entrypoints are TS-specific and live entirely in `tsparser` + JS runtime + tsbundler; Go code is mostly a transport/daemon wrapper around these.  

---

### TS Substrate Component Map

#### Rust crates involved in TS parsing / metadata / graph

- **`encore-tsparser` (crate: `encore-tsparser`)**  
  - **Location**: `tsparser/` (Rust + TS testdata).  
  - **Entry binary**: `tsparser/src/bin/tsparser-encore.rs` (binary `tsparser-encore` per `tsparser/Cargo.toml`).  
  - **Key APIs (Rust)**:  
    - `Builder` orchestrates prepare/parse/compile/test/gen-user-facing (`tsparser/src/builder/mod.rs`).  
    - `Builder::parse(&ParseParams) -> Option<AppDesc>` (`tsparser/src/builder/parse.rs`).  
    - `Builder::compile(&CompileParams) -> CompileResult` (`tsparser/src/builder/compile.rs`).  
    - `Builder::test(&TestParams) -> TestResult` (`tsparser/src/builder/test.rs`).  
    - `Builder::generate_code(&CodegenParams) -> CodegenResult` (`tsparser/src/builder/codegen.rs`).  
    - `ParseContext` + `Parser` + `ModuleLoader` drive SWC parsing and module resolution (`tsparser/src/parser/parser.rs`, `tsparser/src/parser/module_loader.rs`).  
    - `legacymeta::compute_meta(&ParseContext, &ParseResult) -> v1::Data` builds the Encore meta protobuf (`tsparser/src/legacymeta/mod.rs`).  
  - **Inputs**:  
    - Commands over stdin: `"prepare"`, `"parse"`, `"compile"`, `"test"`, `"gen-user-facing"` (`tsparser/src/bin/tsparser-encore.rs`).  
    - JSON payloads for each command (e.g. `ParseInput` with `app_root`, `platform_id`, `local_id`, `parse_tests`) (`tsparser/src/bin/tsparser-encore.rs`).  
    - Environment: `ENCORE_APP_REVISION`, `ENCORE_JS_RUNTIME_PATH`, `ENCORE_TSPARSER_PATH`, `ENCORE_TSBUNDLER_PATH` (indirect).  
    - Files: TS source under app root; `tsconfig.json` for path mapping; `package.json` for package manager (`tsparser/src/parser/parser.rs`, `tsparser/src/runtimeresolve/node.rs`, `tsparser/src/builder/package_mgmt.rs`).  
  - **Outputs**:  
    - For `prepare`: JSON describing installed runtime/version setup (`builder.prepare` result) written with framed length + status byte (`tsparser/src/bin/tsparser-encore.rs`).  
    - For `parse`: protobuf bytes `encore.parser.meta.v1.Data` from `AppDesc.meta.encode_to_vec()` (`tsparser/src/bin/tsparser-encore.rs`, `tsparser/src/legacymeta/mod.rs`).  
    - For `compile`: JSON of `CompileResult { outputs: Vec<JSBuildOutput> }` (`tsparser/src/builder/compile.rs`).  
    - For `test`: JSON of `TestResult { cmd: Option<CmdSpec> }` (`tsparser/src/builder/test.rs`).  
    - For `gen-user-facing`: empty success payload on completion after generating TS code (`tsparser/src/bin/tsparser-encore.rs`, `tsparser/src/builder/codegen.rs`).  
  - **Ownership**: fully **Rust-owned**, invoked as an external CLI (`tsparser-encore`).

- **`encore-runtime-core` (crate: `encore_runtime_core`)**  
  - **Location**: `runtimes/core/`.  
  - **Key APIs**:  
    - `RuntimeBuilder` and `Runtime` (`runtimes/core/src/lib.rs`).  
    - Generated prost modules for `encore.runtime.v1` and `encore.parser.meta.v1` (`runtimes/core/src/lib.rs` `encore::runtime::v1`, `encore::parser::meta::v1`).  
    - `Runtime::builder().with_meta_autodetect().with_runtime_config_from_env().build()` used from JS N-API glue (`runtimes/js/src/runtime.rs`).  
  - **Inputs**:  
    - Env vars: `ENCORE_RUNTIME_CONFIG` and/or `ENCORE_RUNTIME_CONFIG_PATH`, `ENCORE_INFRA_CONFIG_PATH`, `ENCORE_PROCESS_CONFIG`, `ENCORE_APP_META` and/or `ENCORE_APP_META_PATH` (`runtimes/core/src/lib.rs`).  
    - Binary `RuntimeConfig` (protobuf) and `Data` (meta) via env or `/encore/meta` file.  
  - **Outputs**:  
    - In-memory service graph (`metapb::Data`) and runtime config structures used for hosting API, sqldb, pubsub, objects, metrics (`runtimes/core/src/lib.rs`).  
    - HTTP server start via `api().start_serving()` and internal proxies.  
  - **Ownership**: **Rust-owned**, embedded into Node via N-API.

- **`encore-js runtime bridge` (Rust crate inside `runtimes/js/`)**  
  - **Location**: `runtimes/js/src/*.rs`.  
  - **Key APIs**: N-API exported `Runtime` type and methods (`runtimes/js/src/runtime.rs`).  
  - **Inputs/Outputs**: Provide JS-callable methods that delegate to `encore_runtime_core` (see Evidence).  
  - **Ownership**: **Rust-owned** Node addon, but invoked from **Node**.

#### Node / TypeScript tooling invoked

- **Package management & runtime library install**  
  - **Resolvers**: `resolve_package_manager` chooses npm / bun / yarn / pnpm based on `packageManager` field in `package.json` or workspace root (`tsparser/src/builder/package_mgmt.rs`).  
  - **Package managers**:  
    - `NpmPackageManager::setup_deps` runs `npm install` to install `encore.dev` if not installed or version mismatch (`tsparser/src/builder/package_mgmt.rs`).  
    - `YarnPackageManager`, `PnpmPackageManager`, `BunPackageManager` similarly call `yarn install`, `pnpm install`, or `bun install` and enforce `node-modules` linker for Yarn (`tsparser/src/builder/package_mgmt.rs`).  
  - **Ownership**: Node/npm/bun/yarn/pnpm are **external CLI tools**; usage orchestrated by **Rust** `encore-tsparser`.

- **TS bundling**  
  - **Tool**: `tsbundler-encore` CLI (`tsparser/src/builder/transpiler.rs`).  
  - **Invocation**:  
    - Path from `ENCORE_TSBUNDLER_PATH` or default `"tsbundler-encore"` (`tsparser/src/builder/transpiler.rs`).  
    - Arguments include `--bundle`, `--engine=node:21`, `--outdir=<artifact_dir/...>` and each generated TS entrypoint path.  
  - **Ownership**: external binary (Node-based bundler) invoked by Rust; not in this repo.

- **Node / Bun runtime**  
  - **Entry commands**: constructed as `["node", "--enable-source-maps", "$ARTIFACT_DIR/.../main.mjs"]` or `["bun", "run", "$ARTIFACT_DIR/.../main.mjs"]`, optionally with `--inspect` / `--inspect-brk` in debug mode (`tsparser/src/builder/transpiler.rs`).  
  - **Ownership**: Node/Bun CLIs are external; `CmdSpec` instructs the higher-level supervisor/daemon to spawn them.

- **JS runtime library**  
  - **Package**: `runtimes/js/encore.dev` (TS source, compiled to npm package).  
  - **Usage**: `EncoreRuntimeResolver` resolves imports like `encore.dev/api` via `js_runtime_path` to the local runtime installation (`tsparser/src/runtimeresolve/node.rs`).  
  - **Ownership**: TS code is **Node-owned**, but path resolution is **Rust-owned**.

#### Generated artifacts (encore.gen/, build outputs, runtime config)

- **Codegen directory**: `encore.gen` relative to app root (`tsparser/src/builder/codegen.rs`, `write_gen_encore_dir`).  
  - **Service entrypoints**:  
    - `encore.gen/internal/entrypoints/services/<service>/main.ts` (`tsparser/src/builder/codegen.rs`, `ENTRYPOINT_SERVICE_MAIN` template).  
  - **Gateway entrypoints**:  
    - `encore.gen/internal/entrypoints/gateways/<gateway>/main.ts` (`tsparser/src/builder/codegen.rs`).  
  - **Combined entrypoint** (all services + gateways):  
    - `encore.gen/internal/entrypoints/combined/main.ts` (`tsparser/src/builder/codegen.rs`).  
  - **Client catalog**:  
    - `encore.gen/clients/index.js`, `encore.gen/clients/index.d.ts` listing services (`tsparser/src/builder/codegen.rs`, `CATALOG_CLIENTS_INDEX_*` templates).  
    - `encore.gen/internal/clients/<service>/endpoints.{js,d.ts}`, `endpoints_testing.js` per service (`tsparser/src/builder/codegen.rs`).  
  - **Auth catalog**:  
    - `encore.gen/auth/index.ts` and `encore.gen/internal/auth/auth.ts` for auth handlers (`tsparser/src/builder/codegen.rs`, `CATALOG_AUTH_*` templates).  

- **Build artifacts**:  
  - Artifact dir: `.encore/build` under app root (`tsparser/src/builder/compile.rs`).  
  - Bundled entrypoints: `"$ARTIFACT_DIR/combined/<dir>/<name>.mjs"` or `services/...`, `gateways/...` depending on `InputKind` (`tsparser/src/builder/transpiler.rs`).  
  - Each `JSBuildOutput` stores `artifact_dir` (absolute path) and `entrypoints: Vec<Entrypoint>`; `Entrypoint` includes `CmdSpec { command, env, prioritized_files }` and lists `services` and `gateways` hosted (`tsparser/src/builder/compile.rs`, `tsparser/src/builder/transpiler.rs`, `pkg/builder/builder.go`).

- **Runtime configs & metadata**:  
  - **Meta**: `encore.parser.meta.v1.Data` protobuf encoded by `prost-build` (build script) from `proto/encore/parser/meta/v1/meta.proto` (`tsparser/build.rs`, `tsparser/src/legacymeta/mod.rs`).  
  - **Runtime config**: `encore.runtime.v1.RuntimeConfig` produced by deployment tooling (not TS-specific) and supplied to runtimes via `ENCORE_RUNTIME_CONFIG` or `ENCORE_RUNTIME_CONFIG_PATH` (`runtimes/core/src/lib.rs`).  
  - **On local dev**, Go daemon caches metadata and runtime config in SQLite and env; TS paths reuse those, as described in `docs/daemon_boot_analysis.md`.

#### Runtime boundary: Node vs Rust vs external

- **Rust-owned runtime**:  
  - `encore_runtime_core::Runtime` (Rust) hosts API, infra, metrics, sqldb, pubsub based on `RuntimeConfig` and `Data` (`runtimes/core/src/lib.rs`).  
  - The **canonical service graph & topology** live here (not in Node).  

- **Node-owned runtime process**:  
  - Node/Bun process executes bundled `.mjs` entrypoint(s).  
  - That code imports `encore.dev/*` and instantiates the N-API `Runtime` (Rust core) (`runtimes/js/src/runtime.rs` + `runtimes/js/encore.dev/internal/runtime/mod.ts`).  
  - Node handles HTTP server / framework glue (for TS APIs) and delegates service logic and Encore primitives to the Rust core.  

- **External tools**:  
  - `tsparser-encore` binary: Rust CLI invoked from Go or from AxiomRegent.  
  - `tsbundler-encore`: external TS bundler binary.  
  - Node / Bun / npm / yarn / pnpm: external CLIs for runtime and dependency management.  

---

### End-to-end Call Graphs

#### Parse → Metadata → Service Graph

**High-level text call graph**

- **Daemon / CLI side (Go)**  
  - `v2/tsbuilder.BuilderImpl.Parse` → spawns `tsparser-encore` process, sets env (`ENCORE_JS_RUNTIME_PATH`, `ENCORE_APP_REVISION`) and working dir (`p.App.Root() + p.WorkingDir`) (`v2/tsbuilder/tsbuilder.go`).  
  - Writes `"prepare\n" + PrepareInput JSON` → reads framed JSON success string (ignored by you except for logging) (`v2/tsbuilder/tsbuilder.go`, `tsparser/src/bin/tsparser-encore.rs`).  
  - Writes `"parse\n" + ParseInput JSON` → reads response bytes; on success, interprets payload as `meta.v1.Data` protobuf (`v2/tsbuilder/tsbuilder.go`).  

- **Parser binary (`tsparser-encore`)**  
  - `main` constructs `Builder` and SWC `SourceMap`/`Handler`, then loops on `parse_cmd()` which decodes line-based commands + JSON payloads (`tsparser/src/bin/tsparser-encore.rs`).  
  - For `prepare`: builds `PrepareParams` and calls `builder.prepare`, serializes response JSON → `write_result(Ok(bytes))` (`tsparser/src/bin/tsparser-encore.rs`).  
  - For `parse`:  
    - Constructs `builder::App { root, platform_id, local_id }` (`tsparser/src/bin/tsparser-encore.rs`).  
    - Constructs `ParseContext::new(app_root, js_runtime_path_from_env, SourceMap, Handler)` which wraps a `NodeModulesResolver` in `EncoreRuntimeResolver`, optionally with `TsConfigPathResolver` (to honor TS `paths` and `baseUrl`) (`tsparser/src/parser/parser.rs`, `tsparser/src/runtimeresolve/node.rs`).  
    - Calls `builder.parse(&ParseParams{ app, pc, working_dir, parse_tests })` (`tsparser/src/builder/parse.rs`).  

- **SWC AST & resource discovery**  
  - `Parser::parse` builds a `WalkDir` over `pc.app_root`, ignoring `node_modules`, `encore.gen`, `__tests__`, dotfiles, and TS/JS test/spec files; it prioritizes `encore.service.ts` files (`tsparser/src/parser/parser.rs`).  
  - For each `.ts` file, it calls `ModuleLoader::load_fs_file`, which:  
    - Uses swc’s `Resolver` to resolve the module path (including `~encore/clients` / `encore.gen` special cases) (`tsparser/src/parser/module_loader.rs`).  
    - Reads file into a `SourceFile` and runs `parse_file`, which:  
      - Builds `Syntax::Typescript(TsConfig { tsx, dts, decorators: true, no_early_errors: false, ... })`, `Lexer::new`, and `Parser::new_from(lexer)`, then `parser.parse_module()` (`tsparser/src/parser/module_loader.rs`).  
      - Applies `swc_ecma_transforms_base::resolver` to get a resolved AST (`tsparser/src/parser/module_loader.rs`).  
  - `PassOneParser::parse` walks each module AST, recognizing Encore primitives (e.g. `api(...)`, `service(...)`, `pubsub.topic(...)`, `cron.job(...)`, metrics, SQLDB, buckets) and producing `Resource` values and `Bind`s; it also identifies service roots and documents them (`tsparser/src/parser/parser.rs`, `tsparser/src/parser/resources/*`).  
  - `Parser::parse` then:  
    - Resolves bind references (`resolve_binds`) to actual resources (`tsparser/src/parser/parser.rs`).  
    - Discovers services using `discover_services(&file_set, &binds)` by scanning binds of type `Resource::Service`, `Resource::APIEndpoint`, `Resource::PubSubSubscription`, `Resource::Gateway`, `Resource::AuthHandler` and building `DiscoveredService { name, root }` (`tsparser/src/parser/service_discovery.rs`).  
    - Injects generated `ServiceClient` resources and binds (`inject_generated_service_clients`) so clients for each service appear as resources (`tsparser/src/parser/parser.rs`).  
    - Scans usage expressions in all modules via `UsageResolver` to discover: topic publishers, DB/bucket usage, metric usage, and endpoint calls (`tsparser/src/parser/parser.rs`).  
    - Produces `ParseResult { resources, binds, usages, services }` (`tsparser/src/parser/parser.rs`).  

- **Metadata & service graph**  
  - `validate_and_describe(pc, parse)` first runs `AppValidator` to check:  
    - API path conflicts, duplicate names, schema types, metric constraints, SQL DB uniqueness, etc. (`tsparser/src/legacymeta/mod.rs`).  
  - If no errors, it calls `compute_meta(pc, &parse)` which:  
    - Constructs packages and services from discovered services (service roots) → `v1::Service` and `v1::Package` entries with `rel_path`, `name`, `service_name`, docs, secrets, rpc calls, etc. (`tsparser/src/legacymeta/mod.rs`).  
    - Iterates over binds and resources to produce:  
      - `v1::Rpc` entries for API endpoints (name, service_name, path, HTTP methods, request/response schemas, streaming flags, tags, static assets, auth requirements) (`tsparser/src/legacymeta/mod.rs`).  
      - `v1::SqlDatabase`, `v1::Bucket`, `v1::PubSubTopic`, `v1::Metric` for infra resources, plus `v1::pub_sub_topic::Subscription` for subscriptions and publishers (`tsparser/src/legacymeta/mod.rs`).  
      - `v1::CronJob` for cron jobs mapped to endpoint `QualifiedName`s (`tsparser/src/legacymeta/mod.rs`).  
      - `v1::Gateway` entries with optional `v1::AuthHandler`, enforcing a single `api-gateway` and at most one auth handler (`tsparser/src/legacymeta/mod.rs`).  
      - `Data.decls` representing schema types (via `SchemaBuilder` and `encore.parser.schema.v1`) (`tsparser/src/legacymeta/mod.rs`).  
      - `Data.language = Lang::Typescript` (`tsparser/src/legacymeta/mod.rs`).  
      - `Data.app_revision` from `ENCORE_APP_REVISION` env var or empty (`tsparser/src/legacymeta/mod.rs`).  
    - Returns a fully-populated `v1::Data` service graph.  
  - `AppDesc { parse, meta }` bundles parse + metadata, serialized by `tsparser-encore`’s `Command::Parse` case (`tsparser/src/builder/parse.rs`, `tsparser/src/bin/tsparser-encore.rs`).  

**Key structs / types**

- `ParseContext` (TS AST environment: loader, type checker, file set, error handler) (`tsparser/src/parser/parser.rs`).  
- `ModuleLoader`, `Module` (SWC integration, AST storage, pseudo-modules for `encore.gen/clients` and `encore.gen/auth`) (`tsparser/src/parser/module_loader.rs`).  
- `ParseResult`, `Service`, `Resource`, `Bind`, `Usage` (semantic model of the TS app) (`tsparser/src/parser/parser.rs`, `tsparser/src/parser/resources/*`, `tsparser/src/parser/resourceparser/*`).  
- `v1::Data`, `v1::Service`, `v1::Rpc`, `v1::Gateway`, `v1::PubSubTopic`, `v1::SqlDatabase`, `v1::Bucket`, `v1::Metric`, `v1::CronJob`, etc. from `encore.parser.meta.v1` (`tsparser/build.rs`, `tsparser/src/legacymeta/mod.rs`, `runtimes/core/src/lib.rs`).  

#### Watch → Codegen

- `cli/daemon/watch.go` sets up a debounced watcher per app; on relevant events it calls `regenerateUserCode()` → `genUserFacing()` (TS + Go + CUE) as documented in `docs/daemon_boot_analysis.md`.  
- `genUserFacing()` uses the generic `builder.Impl` interface; for TS apps, the active impl is `v2/tsbuilder.BuilderImpl` (`pkg/builder/builder.go`, `v2/tsbuilder/tsbuilder.go`).  
- `BuilderImpl.GenUserFacing` marshals an empty `genUserFacingInput` JSON and writes `"gen-user-facing\n"+payload` to the same `tsparser-encore` process used for parse/compile (`v2/tsbuilder/tsbuilder.go`, `tsparser/src/bin/tsparser-encore.rs`).  
- In the `tsparser-encore` binary, the `Command::GenUserFacing` arm builds `CodegenParams { app, pc, working_dir, desc }` from the cached parse result and calls `builder.generate_code()` (`tsparser/src/bin/tsparser-encore.rs`).  
- `Builder::generate_code` computes `node_modules` path (for later build steps) and calls `codegen_data`, which walks the `AppDesc` service/bind graph to construct:  
  - Service/gateway entrypoint TS code from templates (including endpoint options, streaming flags, auth, tags).  
  - Client catalog JS/TS for each service and per-app index.  
  - Auth catalog TS files (`tsparser/src/builder/codegen.rs`).  
- `write_gen_encore_dir` writes all `CodegenFile`s under `app_root/encore.gen`, creating directories recursively (`tsparser/src/builder/codegen.rs`).  
- The watcher also updates `.gitignore` to ignore `encore.gen` (and some Go/CUE gen files) (`cli/daemon/watch.go`, as described in `docs/daemon_boot_analysis.md`).  

#### Build → Run

- **Build / compile**  
  - The daemon’s run pipeline uses `builder.Compile` (for TS: `v2/tsbuilder.BuilderImpl.Compile`) with a `CompileParams` containing `BuildInfo`, app instance, prior parse result, experiments, working dir, env, etc. (`pkg/builder/builder.go`, `v2/tsbuilder/tsbuilder.go`).  
  - `BuilderImpl.Compile` reuses the existing `tsparser-encore` process (`data` struct) and writes `"compile\n"+compileInput` with `UseLocalRuntime`, `Debug` (`DebugModeDisabled|Enabled|Break`), and `NodeJSRuntime` (`"nodejs"` or `"bun"`) (`v2/tsbuilder/tsbuilder.go`).  
  - `tsparser-encore` handles `Command::Compile`, constructing `CompileParams { app, pc, working_dir, desc, debug, nodejs_runtime }` and calling `builder.compile` (`tsparser/src/bin/tsparser-encore.rs`, `tsparser/src/builder/compile.rs`).  
  - `Builder::compile` first calls `generate_code` to ensure `encore.gen` is up to date, then:  
    - Creates `.encore/build` directory.  
    - Finds `node_modules` via `find_node_modules_dir`.  
    - Constructs a single `Input { kind: InputKind::Combined(gateway_names, service_names), entrypoint: app_root/encore.gen/internal/entrypoints/combined/main.ts }` (`tsparser/src/builder/compile.rs`).  
    - Uses `EsbuildCompiler` to call `tsbundler-encore` and output `.mjs` bundles + `Entrypoint` specs (`tsparser/src/builder/transpiler.rs`).  
    - Returns `CompileResult { outputs: vec![JSBuildOutput { artifact_dir: build_dir, entrypoints }] }` (`tsparser/src/builder/compile.rs`).  
  - `BuilderImpl.Compile` unmarshals this JSON back into `[]*builder.JSBuildOutput` for generic `CompileResult` (`v2/tsbuilder/tsbuilder.go`, `pkg/builder/builder.go`).  

- **Run / runtime launch**  
  - The Go daemon run pipeline (TS and Go) is described in `docs/daemon_boot_analysis.md` and implemented in `cli/daemon/run/run.go`. For TS builds, when iterating over `CompileResult.Outputs`, any `*builder.JSBuildOutput` is recognized as JS runtime output (`pkg/dockerbuild/spec.go`, `pkg/builder/builder.go`).  
  - For each `JSBuildOutput`, the daemon’s process group builder will:  
    - For each `Entrypoint` (TS combined / per-service / per-gateway), expand `CmdSpec` into `Cmd` by substituting `$ARTIFACT_DIR` with actual `artifact_dir` path (`pkg/builder/builder.go`).  
    - Build process definitions mapping services/gateways to command+env according to `Entrypoint.Services` and `Entrypoint.Gateways`.  
    - Start OS-level processes (`exec.CommandContext`) per entrypoint, often in an all-in-one mode for TS (combined entrypoint) (`cli/daemon/run/proc_groups.go`, `cli/daemon/run/run.go`).  
  - Each Node/Bun process then:  
    - Loads the compiled `.mjs` entrypoint (generated code).  
    - Imports the N-API `Runtime` (Rust) from the `encore.dev/internal/runtime` package, which internally calls `encore_runtime_core::Runtime::builder().with_meta_autodetect().with_runtime_config_from_env().build()` and starts `Runtime::run_blocking()` in a background thread (`runtimes/js/src/runtime.rs`, `runtimes/core/src/lib.rs`).  
    - Registers handlers for all TS endpoints, gateways, subscriptions, and metrics based on what the generated code passes in (`runtimes/js/src/runtime.rs`, `runtimes/js/encore.dev/internal/api/*`).  

- **Ports / listeners & readiness**  
  - The HTTP port the Node process listens on is indirectly controlled by the runtime configuration and `ENCORE_LISTEN_ADDR` env var; in Go runtime this is read via `encoreenv.Get("ENCORE_LISTEN_ADDR")` (`runtimes/go/appruntime/apisdk/app/setup.go`), and in Rust runtime core `api::manager` respects env `ENCORE_LISTEN_ADDR` if set (`runtimes/core/src/api/manager.rs`).  
  - The daemon sets up an external listener that proxies to the TS runtime processes; readiness is determined by attempts to connect to the gateway ports as in the Go path (see `pollUntilProcessIsListening` in `cli/daemon/run/run.go` described in `docs/daemon_boot_analysis.md`). For TS, this uses the same process group machinery as Go.  

#### Reload loop

- File watcher events → `regenerateUserCode` → `bld.Parse` + `bld.GenUserFacing` run again; on success, daemon may trigger `buildAndStart` with new JS build outputs (`cli/daemon/watch.go`, `cli/daemon/run/run.go`).  
- Reload logic waits for new gateway processes to be listening, then tears down old ones (`cli/daemon/run/run.go`, reload behavior in `docs/daemon_boot_analysis.md`).  
- For TS apps, this still flows through `v2/tsbuilder` + `tsparser-encore` for parse/compile and uses shared process-group reload logic; the **runtime core** does not have TS-specific reload but treats new TS processes as any other host.  

---

### Wrapper Contract for AxiomRegent (Rust supervisor wrapper mode)

This section focuses on **what AxiomRegent must call / provide** if you integrate Encore via the existing Rust supervisor (`supervisor-encore`) and TS toolchain, without rehosting the Go daemon.

#### What Rust APIs exist to “start supervisor”

- **Binary entry**: `supervisor/src/bin/supervisor-encore.rs` defines a `#[tokio::main]` that:  
  - Calls `config::load_supervisor_config()` to derive `SupervisorConfig { binary_config, hosted_services, hosted_gateways }`.  
  - Computes an exposed port from `PORT` (default `8080`).  
  - Builds `Process` structs for each hosted service/gateway via `config::create_process_config` (service/gateway assignment, per-service ports, etc.).  
  - Constructs `Supervisor::new(procs)` and calls `supervise(token)` plus an optional HTTP proxy if both services and gateways are present (`supervisor/src/bin/supervisor-encore.rs`).  
- **Library API**:  
  - `Supervisor::new(procs: Vec<Process>)` and `Supervisor::supervise(token: CancellationToken)` can be called directly from your Rust code without invoking the binary (`supervisor/src/supervisor.rs`).  
  - `config::load_supervisor_config()` and `config::create_process_config(...)` are public and can be orchestrated manually (`supervisor/src/config.rs`).  

**Wrapper Contract – starting supervisor from AxiomRegent**

- **Option 1: exec the `supervisor-encore` binary**  
  - Provide environment and config files (see below), set `PORT`, `ENCORE_RUNTIME_CONFIG`/`ENCORE_INFRA_CONFIG_PATH`, `ENCORE_PROCESS_CONFIG`, `ENCORE_APP_META`/`PATH`, and run the binary.  
  - Monitor stdout/stderr and process exit as your integration boundary.  

- **Option 2: embed the supervisor crate**  
  - Call `config::load_supervisor_config()` yourself (after setting env/env-files), then:  
    - Optionally alter `SupervisorConfig` (e.g., filter `hosted_services`/`hosted_gateways`).  
    - Construct `Process` entries via `config::create_process_config(...)`.  
    - Instantiate `Supervisor::new(procs)` and call `supervise(root_token.child_token())` inside your own async runtime (`supervisor/src/config.rs`, `supervisor/src/supervisor.rs`).  

#### Config that must be provided

- **BinaryConfig / supervisor.config.json**  
  - JSON file listing available binaries and which services/gateways they implement; corresponds to the deployment image layout, including TS binaries (Node entrypoints) generated by the TS build pipeline.  
  - Loaded from `-c <path>` CLI flag or default `/encore/supervisor.config.json` inside the container (`supervisor/src/config.rs`).  

- **RuntimeConfig / InfraConfig**  
  - For local TS runs, you can either:  
    - Set `ENCORE_INFRA_CONFIG_PATH` to a JSON file with an `InfraConfig` to be converted into a runtime config, or  
    - Set `ENCORE_RUNTIME_CONFIG` (base64 or `gzip:<base64>`) containing serialized `runtime.v1.RuntimeConfig` (`supervisor/src/config.rs`, `runtimes/core/src/lib.rs`).  
  - This config must define `deployment.hosted_services`, `deployment.hosted_gateways`, infra resources, observability, etc., compatible with TS meta.  

- **ProcessConfig (per process)**  
  - The supervisor encodes `ProcessConfig { local_service_ports, hosted_gateways, hosted_services }` into `ENCORE_PROCESS_CONFIG` (base64 JSON) for each started process (`supervisor/src/config.rs`).  
  - AxiomRegent does **not** have to construct `ENCORE_PROCESS_CONFIG` manually if it uses `config::create_process_config`, but must ensure it passes through to child processes’ env.  

- **App Metadata**  
  - TS meta must be supplied as `ENCORE_APP_META` (base64/gzip+base64 of `encore.parser.meta.v1.Data`) or as a file at `ENCORE_APP_META_PATH` (raw protobuf) (`runtimes/core/src/lib.rs`).  
  - This is exactly the `meta.v1.Data` produced by the TS parser (`tsparser`); your integration must either:  
    - Keep the existing tsbuilder/daemon path for local dev, or  
    - Call `tsparser-encore` yourself, capture the proto payload from `parse`, and set the env/file accordingly before launching processes.  

#### Artifacts / metadata required before supervisor start

- **TS build artifacts**: `.encore/build` directory containing `.mjs` entrypoints referenced in `BinaryConfig.command` and `Entrypoint.Cmd.Command` (via `$ARTIFACT_DIR` substitution).  
- **Meta protobuf**: `encore.parser.meta.v1.Data` (as above).  
- **Runtime config protobuf**: `encore.runtime.v1.RuntimeConfig`.  
- **Supervisor config JSON**: lists the TS binaries and their service/gateway coverage.  

#### Observability / logs / events

- **Process-level**:  
  - `Process` uses `tokio::process::Command` and does not inherit parent env by default (`.env_clear()`); it sets env explicitly (`supervisor/src/supervisor.rs`, `supervisor/src/config.rs`).  
  - Logging from child processes (Node + Rust runtime) goes to their stdout/stderr; AxiomRegent can capture those streams if embedding or by container logging if exec’ing binary.  
- **Runtime-level**:  
  - `encore_runtime_core` uses its own logging module (`runtimes/core/src/log/mod.rs`, `writers.rs`) and can be controlled via env like `ENCORE_RUNTIME_LOG`, `ENCORE_LOG`, `ENCORE_NOLOG`, `ENCORE_RUNTIME_TRACE`.  
  - Traces and metrics are exported to endpoints defined in `RuntimeConfig` (e.g., Encore SaaS) (`runtimes/core/src/lib.rs`).  

#### Minimal substitute for Go daemon behavior

- **Things Go daemon today provides that you must replace or reuse**:  
  - App tracking, file watching, and codegen triggers (`cli/daemon/apps`, `cli/daemon/watch.go`).  
  - TS parse/compile/test orchestration via `v2/tsbuilder` and `tsparser-encore` (`v2/tsbuilder/tsbuilder.go`).  
  - Runtime config generation and infra setup (SQL, pubsub, objects, metrics) described in `docs/daemon_boot_analysis.md`.  
- **TS-native alternative path in repo**:  
  - There is **no** fully TS-only path that bypasses the Go daemon for generating `RuntimeConfig` and infra; the Rust runtime core assumes it receives a valid `RuntimeConfig` + `Data`.  
  - However, once you **have** those artifacts, you can **fully bypass the Go daemon** and use only the Rust supervisor + Rust runtime core + TS runtime to run the app.  

**Wrapper Contract (concise list)**

- **Inputs AxiomRegent must provide before launching Encore TS runtime via Rust supervisor**:  
  - `ENCORE_APP_META` or `ENCORE_APP_META_PATH`: serialized `encore.parser.meta.v1.Data` from TS parser (`runtimes/core/src/lib.rs`, `tsparser/src/legacymeta/mod.rs`).  
  - `ENCORE_RUNTIME_CONFIG` or `ENCORE_RUNTIME_CONFIG_PATH` or `ENCORE_INFRA_CONFIG_PATH`: serialized `encore.runtime.v1.RuntimeConfig` or JSON `InfraConfig` convertible to it (`runtimes/core/src/lib.rs`, `supervisor/src/config.rs`).  
  - `supervisor.config.json` (or equivalent) describing TS binaries and the services/gateways they host (`supervisor/src/config.rs`).  
  - Built TS artifacts and Node binaries reachable by `BinaryConfig.command` and `CmdSpec.Command` (from the compile step).  

- **Runtime environment per process (set via `create_process_config`)**:  
  - `PORT`: port the process should bind (supervisor chooses and injects) (`supervisor/src/config.rs`).  
  - `ENCORE_PROCESS_CONFIG`: base64 JSON `ProcessConfig { local_service_ports, hosted_gateways, hosted_services }` (`supervisor/src/config.rs`, `runtimes/core/src/lib.rs`).  
  - All other inherited env (`std::env::vars()`) including `ENCORE_RUNTIME_CONFIG`, `ENCORE_APP_META`, log envs, etc.  

- **Lifecycle hooks / events to observe**:  
  - Process start / restart logs via `log::info!(proc = name; "starting process")` and `"process exited"` (`supervisor/src/supervisor.rs`).  
  - Shutdown via `CancellationToken` (AxiomRegent should trigger cancellation for clean shutdown).  
  - Optional HTTP proxy’s lifecycle if using `GatewayProxy` (exposes port `PORT` for external traffic) (`supervisor/src/bin/supervisor-encore.rs`, `supervisor/src/proxy.rs`).  

---

### TS Env Vars + Files by Phase

#### Parse phase

- **Env vars**  
  - `ENCORE_TSPARSER_PATH`: override path to `tsparser-encore` binary (`v2/tsbuilder/tsbuilder.go`).  
  - `ENCORE_JS_RUNTIME_PATH`: path to JS runtime root (e.g., `.../runtimes/js`); passed to `tsparser-encore` and used by `EncoreRuntimeResolver` to resolve `encore.dev/*` (`v2/tsbuilder/tsbuilder.go`, `tsparser/src/runtimeresolve/node.rs`).  
  - `ENCORE_APP_REVISION`: app revision string embedded in `meta.v1.Data.app_revision` (`v2/tsbuilder/tsbuilder.go`, `tsparser/src/legacymeta/mod.rs`).  

- **Files / dirs**  
  - App root (contains `encore.app`, TS services, endpoints).  
  - `tsconfig.json` in app root for TS path resolution (`tsparser/src/parser/parser.rs`, `tsparser/src/runtimeresolve/node.rs`).  
  - `package.json` in app root and potentially workspace root for package manager detection (`tsparser/src/builder/package_mgmt.rs`).  
  - JS runtime path tree (e.g., `runtimes/js/encore.dev/**`) for SWC module resolution (`tsparser/src/runtimeresolve/node.rs`).  

#### Codegen phase

- **Env vars**  
  - Inferred from parse; same as parse phase (no new TS-specific ones).  

- **Files / dirs (written)**  
  - `encore.gen/**` (TS service/gateway entrypoints, client catalog, auth catalog) (`tsparser/src/builder/codegen.rs`).  
  - `.gitignore` updated with `encore.gen` (via daemon, as per `docs/daemon_boot_analysis.md` and `cli/daemon/watch.go`).  

#### Build phase

- **Env vars**  
  - `ENCORE_TSBUNDLER_PATH`: override path to `tsbundler-encore`; default is `"tsbundler-encore"` (`tsparser/src/builder/transpiler.rs`).  
  - `BUN_INSTALL_BACKEND`: optional for bun’s install backend (`tsparser/src/builder/package_mgmt.rs`).  
  - System `PATH` must contain Node/Bun and tsbundler-encore.  

- **Files / dirs (read/written)**  
  - Reads `encore.gen/**` TS entrypoints as bundler inputs (`tsparser/src/builder/compile.rs`, `tsparser/src/builder/transpiler.rs`).  
  - Writes `.encore/build/**` containing bundled `.mjs` artifacts (`tsparser/src/builder/compile.rs`, `tsparser/src/builder/transpiler.rs`).  
  - Reads/writes `node_modules/` and `package.json` for dependency install (`tsparser/src/builder/package_mgmt.rs`).  

#### Run phase

- **Env vars (per process)**  
  - `ENCORE_RUNTIME_CONFIG` / `ENCORE_RUNTIME_CONFIG_PATH` / `ENCORE_INFRA_CONFIG_PATH`: runtime config for `encore_runtime_core` (`runtimes/core/src/lib.rs`, `supervisor/src/config.rs`).  
  - `ENCORE_APP_META` / `ENCORE_APP_META_PATH`: TS metadata (`runtimes/core/src/lib.rs`).  
  - `ENCORE_PROCESS_CONFIG`: per-process JSON, base64-encoded (service/gateway mapping, local ports) (`runtimes/core/src/lib.rs`, `supervisor/src/config.rs`).  
  - `ENCORE_LISTEN_ADDR`: optional override for listen address (used by API server) (`runtimes/core/src/api/manager.rs`).  
  - Logging: `ENCORE_RUNTIME_LOG`, `ENCORE_LOG`, `ENCORE_NOLOG`, `ENCORE_RUNTIME_TRACE`, `ENCORE_LOG_INCLUDE_ERROR_STACK`, `ENCORE_API_INCLUDE_ERROR_STACK`, `ENCORE_API_INCLUDE_INTERNAL_MESSAGE` (`runtimes/core/src/log/mod.rs`, `runtimes/core/src/api/error.rs`, `runtimes/core/src/api/endpoint.rs`).  
  - Node-level: `NODE_ENV` (test vs prod) interpreted both by TS runtime and TS metrics registry (`runtimes/js/encore.dev/internal/runtime/mod.ts`, `runtimes/js/encore.dev/internal/metrics/registry.ts`).  

- **Files / dirs**  
  - Bundled `.mjs` files under `.encore/build/**` referenced by `$ARTIFACT_DIR` in `CmdSpec.Command` (`tsparser/src/builder/transpiler.rs`, `pkg/builder/builder.go`).  
  - Optional `/encore/meta` file if `ENCORE_APP_META*` not present (`runtimes/core/src/lib.rs`).  

#### Watch / reload phase

- **Env vars**  
  - `ENCORE_DAEMON_WATCH`: enable/disable watching (Go daemon) (`cli/daemon/watch.go` as described in `docs/daemon_boot_analysis.md`).  
  - Same TS env vars as parse/codegen reused.  

- **Files / dirs**  
  - App root + runtimes dir are watched; `encore.gen/` and `.encore/` changes are often ignored (see ignore rules) (`cli/daemon/watch.go`, `pkg/watcher/util.go`).  

---

### Trail (key hops with links to next symbols)

1. **`tsparser/src/bin/tsparser-encore.rs` – `main`**  
   - Learned: TS parser entrypoint, command protocol (`prepare`, `parse`, `compile`, `test`, `gen-user-facing`), how it frames responses and uses `Builder`.  
   - Next hop: `Builder` implementation & `ParseContext` (`tsparser/src/builder/mod.rs`, `tsparser/src/parser/parser.rs`).  

2. **`tsparser/src/builder/mod.rs` – `Builder`, templates, `NodeJSRuntime`**  
   - Learned: TS-specific templates for entrypoints, clients, auth; available operations (`prepare`, `parse`, `compile`, `test`, `generate_code`); debug/runtime flags.  
   - Next hop: `parse`, `compile`, `codegen`, `package_mgmt`, `transpiler` modules.  

3. **`tsparser/src/parser/parser.rs` – `ParseContext`, `Parser::parse`**  
   - Learned: SWC loader and type checker wiring, directory traversal rules, `encore.service.ts` prioritization, service discovery, usage scanning.  
   - Next hop: `ModuleLoader` for SWC details; `service_discovery` for service root semantics.  

4. **`tsparser/src/parser/module_loader.rs` – `ModuleLoader`, SWC integration**  
   - Learned: Use of `Lexer`, `Parser`, `Syntax::Typescript`, AST resolution, virtual modules for `encore.gen/clients` and `encore.gen/auth`, error handling.  
   - Next hop: `EncoreRuntimeResolver` for resolving `encore.dev` runtime modules.  

5. **`tsparser/src/runtimeresolve/node.rs` – `EncoreRuntimeResolver`**  
   - Learned: Node-style resolution for `encore.dev` via `package.json` exports and `js_runtime_path`, integration with TSConfig path resolver, preference for `.d.ts` over `.js`.  
   - Next hop: How `ParseContext::new` wires this resolver; back to `parser.rs`.  

6. **`tsparser/src/legacymeta/mod.rs` – `compute_meta`**  
   - Learned: Transformation from `ParseResult` to `encore.parser.meta.v1.Data`, covering services, RPCs, gateways, auth, pubsub, cron, metrics, SQL, buckets, and setting `language = Typescript`.  
   - Next hop: Protobuf generation via `build.rs`; runtime consumption in `runtimes/core`.  

7. **`tsparser/build.rs` – prost build of meta proto**  
   - Learned: `encore.parser.meta.v1` Rust module is generated from `proto/encore/parser/meta/v1/meta.proto`.  
   - Next hop: `runtimes/core/src/lib.rs` to see how this meta is consumed in the runtime.  

8. **`runtimes/core/src/lib.rs` – `RuntimeBuilder`, `Runtime`**  
   - Learned: How runtime config and meta are loaded from env / files, and how they drive API, infra, and metrics managers; `with_meta_autodetect` behavior including `/encore/meta`.  
   - Next hop: JS bridge `runtimes/js/src/runtime.rs`.  

9. **`runtimes/js/src/runtime.rs` – N-API `Runtime` wrapper**  
   - Learned: Node/TS layer embeds `encore_runtime_core::Runtime`, controls test mode, exposes APIs (SQL DB, pubsub, bucket, gateway, secrets, api_call, stream, app_meta, runtime_config).  
   - Next hop: Understand how TS entrypoints interact with this JS runtime via generated code (`tsparser` codegen).  

10. **`tsparser/src/builder/codegen.rs` – `generate_code`, `codegen_data`**  
    - Learned: Exact structure of generated `encore.gen` TS files, per-service and per-gateway main entrypoints, client catalog, auth catalog, and combined main.  
    - Next hop: `transpiler` and `compile` to see bundling and build outputs.  

11. **`tsparser/src/builder/transpiler.rs` – `EsbuildCompiler` and `TranspileParams`**  
    - Learned: Use of `tsbundler-encore` to bundle TS entrypoints into `.mjs`, mapping them to `Entrypoint` specs with Node/Bun command lines and lists of services/gateways.  
    - Next hop: `compile.rs` to see how combined entrypoints are selected and build dir configured.  

12. **`tsparser/src/builder/compile.rs` – `compile`**  
    - Learned: Combined entrypoint bundling for all services+gateways, `.encore/build` structure, connection to `JSBuildOutput` and generic `BuildOutput` abstraction.  
    - Next hop: `pkg/builder/builder.go` to see how these outputs are consumed by the daemon; `v2/tsbuilder/tsbuilder.go` as process bridge.  

13. **`pkg/builder/builder.go` – `JSBuildOutput`, `Entrypoint`, `CmdSpec`**  
    - Learned: Shape of runtime commands for TS (artifact directory, command/env expansion, services/gateways per entrypoint).  
    - Next hop: `v2/tsbuilder/tsbuilder.go` to see the tsparser bridge and env.  

14. **`v2/tsbuilder/tsbuilder.go` – TS builder implementation**  
    - Learned: How Go side invokes `tsparser-encore` with env (`ENCORE_JS_RUNTIME_PATH`, `ENCORE_APP_REVISION`), handles its framed responses, passes `DebugMode` and `NodeJSRuntime`, and drives `Parse`, `Compile`, `TestSpec`, `GenUserFacing`.  
    - Next hop: `supervisor` crate and `runtimes/core` for standalone Rust integration.  

15. **`supervisor/src/config.rs` & `supervisor/src/bin/supervisor-encore.rs`**  
    - Learned: Supervisor config model (`BinaryConfig`, `InfraConfig`, `RuntimeConfig` lite), environment variables (`ENCORE_INFRA_CONFIG_PATH`, `ENCORE_RUNTIME_CONFIG`, `ENCORE_PROCESS_CONFIG`), process creation and restart strategy.  
    - Next hop: `runtimes/core/src/lib.rs` to align env expectations; design wrapper contract.  

16. **`docs/daemon_boot_analysis.md`**  
    - Learned: Holistic picture of daemon boot, app watching, codegen triggers, build/run pipeline, and readiness semantics, confirming TS builder involvement and wrappers generation.  
    - Next hop: Integrate all into the TS-only trace and AxiomRegent contract.  

---

### Evidence Index (files + symbols)

1. `tsparser/Cargo.toml` – crate name `encore-tsparser`, binary `tsparser-encore`.  
2. `tsparser/src/bin/tsparser-encore.rs` – `main`, `Command` enum, `parse_cmd`, `PrepareInput`, `ParseInput`, `CompileInput`, `TestInput`, `GenUserFacingInput`, `write_result`.  
3. `tsparser/src/builder/mod.rs` – `Builder`, `App`, `Template`, `DebugMode`, `NodeJSRuntime`, template constants for entrypoints/clients/auth.  
4. `tsparser/src/builder/parse.rs` – `ParseParams`, `ParseError`, `Builder::parse`.  
5. `tsparser/src/parser/parser.rs` – `ParseContext`, `Parser`, `ParseResult`, `Service`, `Parser::parse`, `inject_generated_service_clients`, `collect_services`.  
6. `tsparser/src/parser/module_loader.rs` – `ModuleLoader`, `Module`, `Error`, `load_fs_file`, `parse_file`, `universe`, `encore_app_clients`, `encore_auth`.  
7. `tsparser/src/runtimeresolve/node.rs` – `EncoreRuntimeResolver`, `resolve_encore_module`, `resolve_export`, `Resolve for EncoreRuntimeResolver`.  
8. `tsparser/src/parser/service_discovery.rs` – `discover_services`, `DiscoveredService`, `ServiceDiscoverer`.  
9. `tsparser/src/legacymeta/mod.rs` – `compute_meta`, `MetaBuilder`, `new_meta`, all transforms to `encore.parser.meta.v1`.  
10. `tsparser/build.rs` – `prost_build::compile_protos("../proto/encore/parser/meta/v1/meta.proto", "../proto/")`.  
11. `tsparser/src/builder/codegen.rs` – `CodegenParams`, `CodegenResult`, `CodegenFile`, `Builder::generate_code`, `codegen_data`, `write_gen_encore_dir`, `find_node_modules_dir`.  
12. `tsparser/src/builder/compile.rs` – `CompileParams`, `CompileResult`, `JSBuildOutput`, `Entrypoint`, `CmdSpec`, `Builder::compile`.  
13. `tsparser/src/builder/test.rs` – `TestParams`, `TestResult`, `Builder::test`.  
14. `tsparser/src/builder/package_mgmt.rs` – `resolve_package_manager`, `PackageManager` trait, `NpmPackageManager`, `YarnPackageManager`, `PnpmPackageManager`, `BunPackageManager`, `update_package_json`, `PackageJson`.  
15. `tsparser/src/builder/transpiler.rs` – `ExternalPackages`, `InputKind`, `Input`, `TranspileParams`, `TranspileResult`, `OutputTranspiler`, `EsbuildCompiler`, `file_stem_and_dir`, use of `ENCORE_TSBUNDLER_PATH`.  
16. `tsparser/src/parser/types/*` and `tsparser/src/parser/resources/*` – type system and resource encodings used by `compute_meta` (e.g. `Endpoint`, `Methods`, `Param`, `MetricType`, `CronJobSchedule`).  
17. `v2/tsbuilder/tsbuilder.go` – `BuilderImpl`, `Parse`, `Compile`, `RunTests`, `TestSpec`, `GenUserFacing`, `getTSParserPath`, `jsRuntimeRoot`, `parseInput`, `compileInput`, `NodeJSRuntime` (Go string type), `readResp`.  
18. `pkg/builder/builder.go` – `ParseParams`, `ParseResult`, `CompileParams`, `CompileResult`, `JSBuildOutput`, `Entrypoint`, `CmdSpec`, `BuildOutput` interface.  
19. `docs/daemon_boot_analysis.md` – narrative of daemon boot, app watching, TS wrappers/codegen, TS build/run usage.  
20. `cli/daemon/watch.go` – watcher logic, codegen triggers, gitignore updates for `encore.gen` and `.encore`.  
21. `pkg/watcher/util.go` – ignores `node_modules` and `encore.gen` in watches.  
22. `runtimes/core/src/lib.rs` – `RuntimeBuilder`, `Runtime`, env-parsing functions (`infra_config_from_env`, `runtime_config_from_env`, `meta_from_env`, `proc_config_from_env`), `version`, `build_commit`, `Hosted`.  
23. `runtimes/core/src/api/manager.rs` – respect for `ENCORE_LISTEN_ADDR`, service hosting model.  
24. `runtimes/core/src/api/error.rs`, `runtimes/core/src/api/endpoint.rs`, `runtimes/core/src/log/mod.rs`, `runtimes/core/src/log/writers.rs` – env vars for logging / error behavior.  
25. `runtimes/js/src/runtime.rs` – N-API `Runtime` type, `Runtime::new`, `run_forever`, `api_call`, `stream`, `sql_database`, `pubsub_topic`, `bucket`, `gateway`, `secret`, `app_meta`, `runtime_config`.  
26. `runtimes/js/encore.dev/**` – TS runtime library providing `encore.dev` APIs; uses `process.env.NODE_ENV` in `internal/runtime/mod.ts` and `internal/metrics/registry.ts`.  
27. `supervisor/src/lib.rs` – module re-exports `config`, `proxy`, `supervisor`.  
28. `supervisor/src/supervisor.rs` – `Supervisor`, `Process`, `RestartPolicy`, `kill_gracefully`.  
29. `supervisor/src/config.rs` – `load_supervisor_config`, `load_binary_config`, `load_hosted_processes`, `create_process_config`, `SupervisorConfig`, `BinaryConfig`, `InfraConfig`, `RuntimeConfig` (lightweight), `ProcessConfig`.  
30. `supervisor/src/bin/supervisor-encore.rs` – main wiring: loads config, assigns ports, constructs `Process` list, starts `Supervisor` and `GatewayProxy`, installs Ctrl+C handler.  
31. `runtimes/go/appruntime/apisdk/app/setup.go` – uses `ENCORE_LISTEN_ADDR` in Go path (reference for TS parity).  
32. `internal/env/env.go`, `docs/daemon_boot_analysis.md` – broader env vars like `ENCORE_RUNTIMES_PATH`, daemon listen addresses (mostly Go-level but relevant to TS runtime location).  

This covers the requested TS-only substrate, end-to-end flow from parsing to runtime, codegen and build hooks, and the Rust supervisor integration contract AxiomRegent can rely on.