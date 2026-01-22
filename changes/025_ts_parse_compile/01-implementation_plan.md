# Implementation Plan: TS Parse + Codegen + Compile

## Goal
Implement tools to parse, codegen, and compile Encore TS apps directly using `tsparser-encore` binary, bypassing Encore Go daemon.

## Proposed Changes

### [NEW] `src/tools/encore_ts/tools.rs`
- **Execution Strategy**: Spawn and hold one `tsparser-encore` process per workspace.
- **`encore.ts.env.check`**: Report versions of `node`, `npm`, and `tsparser-encore`.
- **`encore.ts.parse`**: Call `tsparser-encore` -> `prepare` -> `parse`. Cache `meta.pb`.
- **`encore.ts.codegen`**: Call `tsparser-encore` -> `gen-user-facing`.
- **`encore.ts.compile`**: Call `tsparser-encore` -> `compile`. Returns bundle info and `CmdSpec`.

### [NEW] `src/tools/encore_ts/client.rs`
- Implement `TsParserClient` standard input/output JSON-RPC wrapper.

### [MODIFY] `src/router/mod.rs`
- Register `encore.ts.*` tools.

## Verification Plan
- Unit test `EnvInfo` struct.
- Integration test `test_parse_encore_app` using fixture.
