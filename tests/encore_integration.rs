use anyhow::Result;
use axiomregent::tools::encore_ts::{parse, run, state};
use std::path::PathBuf;

#[test]
fn test_parse_encore_app() -> Result<()> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    // The parser might fail if it strictly requires encore.service or tsconfig or other things.
    // encore-tsparser expects a valid structure.
    // If it fails, we might need to mock more of the app structure (encore.app, tsconfig.json).

    // For now, let's try with minimal.
    // Actually, `encore-tsparser` usually looks for `encore.app`.
    // Let's create `encore.app` as well.

    // Ensure we create encore.app in the fixture via write_to_file in next step

    let snapshot = parse::parse(&root).expect("Failed to parse encore app");

    // Debug print
    println!("Snapshot: {:?}", snapshot);

    // Verify service "exampleService" exists
    let example_service = snapshot
        .services
        .iter()
        .find(|s| s.name == "exampleService");
    assert!(
        example_service.is_some(),
        "Service 'exampleService' not found"
    );
    let service = example_service.unwrap();

    // Verify API "dynamicPathParamExample" exists
    let api = service
        .apis
        .iter()
        .find(|a| a.name == "dynamicPathParamExample");

    assert!(
        api.is_some(),
        "API 'dynamicPathParamExample' not found in service 'exampleService'"
    );
    let api = api.unwrap();

    assert_eq!(api.method, "GET");
    assert_eq!(api.path, "/hello/:name");
    assert_eq!(api.access, "public");

    // Note: this test will only pass if I add encore.app.
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

    // Cleanup - purely optional, but good for test hygiene
    let _ = std::fs::remove_dir_all(run_dir);

    Ok(())
}
