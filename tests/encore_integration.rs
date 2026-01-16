use anyhow::Result;
use axiomregent::tools::encore_ts::{parse, run, state};
use std::path::PathBuf;

#[test]
fn test_parse_encore_app() -> Result<()> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    let snapshot = parse::parse(&root).expect("Failed to parse encore app");

    // Debug print
    println!("Snapshot: {:?}", snapshot);

    // Verify service "exampleService" exists
    let example_service = snapshot
        .services
        .iter()
        .find(|s| s.name == "exampleService")
        .expect("Service 'exampleService' not found");

    // Verify API "dynamicPathParamExample" exists
    let api = example_service
        .apis
        .iter()
        .find(|a| a.name == "dynamicPathParamExample")
        .expect("API 'dynamicPathParamExample' not found");

    assert_eq!(api.method, "GET");
    assert_eq!(api.path, "/hello/:name");
    assert_eq!(api.access, "public");

    // Verify service "anotherService" exists
    let another_service = snapshot
        .services
        .iter()
        .find(|s| s.name == "anotherService")
        .expect("Service 'anotherService' not found");

    // "anotherService" has no APIs in the simple fixture, just verify existence.
    assert_eq!(another_service.name, "anotherService");
    Ok(())
}

#[test]
fn test_run_persistence() -> Result<()> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    // Check if encore CLI is available
    if std::process::Command::new("encore")
        .arg("version")
        .output()
        .is_err()
    {
        println!("Skipping test_run_persistence: encore CLI not found");
        return Ok(());
    }

    let mut state = state::EncoreState::new();

    // Start
    let run_id = run::start(&mut state, &root, None)?;

    // Check if .axiomregent/runs/<run_id>/state.json exists
    let cwd = std::env::current_dir()?;
    let run_dir = cwd.join(".axiomregent").join("runs").join(&run_id);
    let state_path = run_dir.join("state.json");

    assert!(
        state_path.exists(),
        "State file should exist at {:?}",
        state_path
    );

    // Stop
    run::stop(&mut state, &run_id)?;

    // Cleanup
    // Commented out to allow inspection of artifacts as requested
    // let _ = std::fs::remove_dir_all(run_dir);

    Ok(())
}
