use anyhow::Result;
use axiomregent::tools::encore_ts::tools::EncoreTools;
use axiomregent::tools::encore_ts::{parse, run, state};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[test]
fn test_parse_golden_stable() -> Result<()> {
    // Only works if we run sequentially due to PATH changes in other tests
    setup_path();

    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    let snapshot = parse::parse(&root)?;

    let json = serde_json::to_string_pretty(&snapshot)?;

    let mut golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    golden_path.push("tests/golden/encore_parse_snapshot.json");

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(&golden_path, &json)?;
    }

    let golden = fs::read_to_string(&golden_path).unwrap_or_else(|_| "".to_string());

    assert_eq!(
        golden.trim(),
        json.trim(),
        "Snapshot does not match golden file at {:?}",
        golden_path
    );

    Ok(())
}

#[test]
fn test_meta_error_handling() -> Result<()> {
    setup_path();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app_error");

    let result = parse::parse(&root);

    match result {
        Ok(_) => {
            panic!("Expected parse to fail due to syntax error in fixture, but it succeeded.");
        }
        Err(e) => {
            println!("Got expected error: {:?}", e);
        }
    }

    Ok(())
}

fn setup_path() {
    let mut path = std::env::var("PATH").unwrap_or_default();
    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    let bin_path = bin_dir.to_string_lossy();
    if !path.contains(&*bin_path) {
        let new_path = format!("{}:{}", bin_path, path);
        unsafe {
            std::env::set_var("PATH", new_path);
        }
    }
}

#[test]
fn test_env_check_present() -> Result<()> {
    setup_path();
    let env = axiomregent::tools::encore_ts::env::check()?;
    assert!(env.deployed, "Env check should pass with mock encore");
    assert_eq!(env.version, "v1.0.0-mock");
    Ok(())
}

#[test]
fn test_env_check_missing_node() -> Result<()> {
    setup_path();

    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe {
        std::env::set_var("PATH", "");
    }

    let result = axiomregent::tools::encore_ts::env::check();

    unsafe {
        std::env::set_var("PATH", original_path);
    }

    let env = result?;
    assert!(
        !env.deployed,
        "Env check should return deployed=false when PATH is empty"
    );

    Ok(())
}

#[test]
fn test_run_idempotency_determinism_and_logs() -> Result<()> {
    setup_path();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    // Check mock
    let out = Command::new("encore").arg("version").output()?;
    if !out.status.success() {
        println!("Skipping encore run tests: encore mock not working");
        return Ok(());
    }

    // Use EncoreTools to test logs.stream as well
    let tools = EncoreTools::new();

    // Start via tools wrapper (or internal, but tools wrapper needs mutex)
    // Let's use internal run logic manually first to populate state inside tools?
    // Tools has its own state.
    // If we use tools.run_start, we test end-to-end.

    // Tools::run_start returns Value.
    let res = tools.run_start(&root, None, None)?;
    let run_id_1 = res.get("run_id").unwrap().as_str().unwrap().to_string();

    // Idempotency: call again
    let res2 = tools.run_start(&root, None, None)?;
    let run_id_2 = res2.get("run_id").unwrap().as_str().unwrap().to_string();

    assert_eq!(run_id_1, run_id_2, "Run ID should be same via tools");

    // Check files
    let cwd = std::env::current_dir()?;
    let run_dir = cwd.join(".axiomregent").join("runs").join(&run_id_1);
    let logs_path = run_dir.join("logs.ndjson");

    assert!(logs_path.exists(), "logs.ndjson should exist");

    // Wait a bit for mock logs to appear
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Check internal logs via tool
    let logs_res = tools.logs_stream(&run_id_1, None)?;
    let logs_arr = logs_res.get("logs").unwrap().as_array().unwrap();
    println!("Logs from tool: {:?}", logs_arr);

    assert!(logs_arr.len() > 0, "Should have logs");
    // Mock encore prints "Mock Encore Run Started", "Log line 1", "Log line 2"

    // Logs from file
    let file_content = fs::read_to_string(&logs_path)?;
    println!("Logs from file: {}", file_content);
    assert!(file_content.contains("Mock Encore Run Started"));

    // Verify determinism of state.json
    let state_path = run_dir.join("state.json");
    let state_content = fs::read_to_string(&state_path)?;
    // Should contain env (check if null or match)
    // RunProcess serializes Env.
    assert!(state_content.contains("root_path"));

    // Stop
    tools.run_stop(&run_id_1)?;

    Ok(())
}

#[test]
fn test_error_codes() -> Result<()> {
    setup_path();
    let root = PathBuf::from("/non-existent/path");
    let err = parse::parse(&root).unwrap_err();

    let err_str = err.to_string();
    // Generic error
    assert!(
        err_str.contains("Encore TS parsing failed with errors")
            || err_str.contains("No such file"),
        "Error string did not contain expected text. Got: '{}'",
        err_str
    );

    Ok(())
}
