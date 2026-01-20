# Walkthrough - Workspace Root Discovery Fix

I have fixed the issue where the `run_root` was incorrectly selected from resolver roots instead of the actual workspace root.

## Changes

### 1. New Helper: `discover_workspace_root`

I added a helper function in `src/util/paths.rs` that walks up the directory tree from a starting path (CWD) to find a workspace marker (`.axiomregent` or `.git`).

```rust
pub fn discover_workspace_root(start: &Path) -> PathBuf {
    let mut cur = Some(start);

    while let Some(p) = cur {
        if p.join(".axiomregent").is_dir() {
            return p.to_path_buf();
        }
        if p.join(".git").is_dir() {
            return p.to_path_buf();
        }
        cur = p.parent();
    }

    start.to_path_buf()
}
```

### 2. Updated `main.rs`

I updated `src/main.rs` to use this discovery logic instead of incorrectly using `dirs.first()`. I also ensured that the `.axiomregent` directory is created if it doesn't exist, preventing issues where it might be created in the parent directory.

```rust
    // ...
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let run_root = axiomregent::util::paths::discover_workspace_root(&cwd);

    // ...

    if let Err(e) = std::fs::create_dir_all(run_root.join(".axiomregent")) {
        log::warn!("Failed to create .axiomregent directory: {}", e);
    }
    let lock_path = run_root.join(".axiomregent/encore.lock");
```

## Verification Results

I added a new test file `tests/workspace_discovery_test.rs` covering the following scenarios:

1.  **Repo Root Discovery**: START in root -> DETECTS root.
2.  **Walk Up**: START in `crates/foo` -> DETECTS root.
3.  **Fallback to CWD**: START in empty dir -> DETECTS CWD.
4.  **Git Fallback**: START in git repo (no `.axiomregent`) -> DETECTS git root.
5.  **Preference**: START in dir with BOTH `.axiomregent` and `.git` -> DETECTS `.axiomregent`.
6.  **Nested Workspaces**: START in nested `.axiomregent` inside `.git` -> DETECTS nested root.

### Automated Tests (`make check`)

All tests passed, including the new workspace discovery tests.

```
running 6 tests
test test_fallback_to_cwd ... ok
test test_repo_root_discovery ... ok
test test_git_fallback ... ok
test test_prefer_axiomregent_over_git ... ok
test test_wc_walk_up ... ok
test test_nested_boundary ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
