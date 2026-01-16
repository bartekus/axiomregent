use anyhow::Result;
use axiomregent::tools::encore_ts::parse;
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
