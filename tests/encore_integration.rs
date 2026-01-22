use anyhow::{Context, Result};
use axiomregent::tools::encore_ts::tools::EncoreTools;
use std::path::PathBuf;

#[test]
fn test_parse_encore_app() -> Result<()> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    // Ensure node_modules exists (needed for CI)
    if !root.join("node_modules").exists() {
        println!("Installing node_modules in {:?}", root);
        let status = std::process::Command::new("npm")
            .arg("install")
            .current_dir(&root)
            .status()
            .context("Failed to run npm install")?;
        if !status.success() {
            anyhow::bail!("npm install failed");
        }
    }

    let tools = EncoreTools::new();
    let result = tools.parse(&root);

    if let Err(e) = &result {
        // Warn if binary missing but expected pass in CI?
        // We panic if we are strict.
        // For development robustness, we allow checking if env is setups
        let env_check = tools.env_check()?;
        let env_info = env_check.as_object().unwrap();
        let is_missing = match env_info.get("tsparser_path") {
            None => true,
            Some(v) => v.is_null(),
        };
        if is_missing {
            println!("Skipping test_parse_encore_app: tsparser-encore not found");
            return Ok(());
        }
        panic!("Parse failed: {:?}", e);
    }

    let val = result.unwrap();
    assert!(val.get("meta_pb_base64").is_some());
    println!(
        "Parsed meta base64 length: {}",
        val.get("meta_pb_base64").unwrap().as_str().unwrap().len()
    );

    // We can't easily verify contents without decoding proto, but existence is good enough for tool check
    Ok(())
}

// PR-4 Run Test (Disabled until PR-4 implementation)
/*
#[test]
fn test_run_persistence() -> Result<()> {
    ...
}
*/
