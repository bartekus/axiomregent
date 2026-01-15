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

    // Verify service "greeting" exists
    let greeting_service = snapshot.services.iter().find(|s| s.name == "greeting");
    assert!(greeting_service.is_some(), "Service 'greeting' not found");
    let service = greeting_service.unwrap();

    // Verify API "get" exists
    let api = service.apis.iter().find(|a| a.name == "get"); // Name might be "get" or inferred.
    // In TS: export const get = greeting.get(...) -> name is "get" usually or empty if not named explicitly?
    // encore-tsparser uses variable name for endpoint name if not overridden?
    // Let's print to see what it parsed.

    assert!(api.is_some(), "API 'get' not found in service 'greeting'");
    let api = api.unwrap();

    assert_eq!(api.method, "GET"); // Assuming default is GET for .get()
    assert_eq!(api.path, "/greeting/:name");
    assert_eq!(api.access, "public");

    // Note: this test will only pass if I add encore.app.
    Ok(())
}
