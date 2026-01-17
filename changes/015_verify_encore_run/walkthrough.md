# Failed Encore Test Fixes

I have resolved the regression failures in `verify_encore_run` and `encore_integration` tests.

## Changes

### 1. Fix `verify_encore_run` Flakiness
The `verify_encore_run` tests were flaky because they modified the global `PATH` environment variable insecurely. If a test panicked (which `test_env_check_missing_node` did), the environment was left in a corrupted state (empty PATH), causing subsequent tests to fail with `PoisonError` or logic errors.

I introduced an `EnvGuard` struct that uses RAII (Resource Acquisition Is Initialization) to safely manage environment variables. It restores the original value (or unsets if it didn't exist) when the guard is dropped, ensuring cleanup even on panic.

```rust
struct EnvGuard {
    key: String,
    original_value: Option<String>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(v) = &self.original_value {
                std::env::set_var(&self.key, v);
            } else {
                std::env::remove_var(&self.key);
            }
        }
    }
}
```

### 2. Fix `encore_integration` Parser Errors
The integration tests failed in CI with "unable to resolve module" because `node_modules` was missing. The `tests/fixtures/encore_app` directory git-ignores `node_modules`, so they weren't present in the fresh checkout.

I updated `tests/encore_integration.rs` to automatically run `npm install` in the fixture directory if `node_modules` is missing. This ensures the test environment is correctly set up regardless of the CI state.

```rust
    // Ensure node_modules exists (needed for CI)
    if !root.join("node_modules").exists() {
        println!("Installing node_modules in {:?}", root);
        let status = std::process::Command::new("npm")
            .arg("install")
            .current_dir(&root)
            .status()
            .context("Failed to run npm install")?;
        // ...
    }
```

### 3. Improved Debugging
I enhanced `module_loader.rs` in `encore-tsparser` to include the source file path (`from_file`) in module resolution error messages. This makes future debugging of import errors much easier.

## Verification

Ran the test suites locally:

```bash
cargo test --test verify_encore_run
cargo test --test encore_integration
```

Both suites now pass consistently.
