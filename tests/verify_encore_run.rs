use anyhow::Result;
use axiomregent::tools::encore_ts::tools::EncoreTools;
use std::path::PathBuf;

// Global lock for environment modification tests
use std::sync::Mutex;
use std::sync::OnceLock;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn test_parse_golden_stable() -> Result<()> {
    // Parser shouldn't depend on path if we pass absolute paths
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("tests/fixtures/encore_app");

    // Use new EncoreTools
    let tools = EncoreTools::new();

    let result = tools.parse(&root);
    if let Err(e) = &result {
        // Check if environment has binary
        let env_check = tools.env_check()?;
        let env_info = env_check.as_object().unwrap();
        let is_missing = match env_info.get("tsparser_path") {
            None => true,
            Some(v) => v.is_null(),
        };
        if is_missing {
            println!("Skipping test_parse_golden_stable: tsparser-encore not found");
            return Ok(());
        }

        println!("Parse failed: {:?}", e);
        // Expect success if env is valid
        panic!("Parse failed unexpectedly");
    } else {
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.get("meta_pb_base64").is_some());
    }

    Ok(())
}

#[allow(unused_mut)]
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
    let _lock = get_env_lock();
    setup_path();
    // We reuse logic from tools
    let tools = EncoreTools::new();
    let result = tools.env_check()?;

    // Check structure
    let env_info = result.as_object().unwrap();
    assert!(env_info.contains_key("node_version"));
    assert!(env_info.contains_key("npm_version"));
    Ok(())
}

struct EnvGuard {
    key: String,
    original_value: Option<String>,
}

impl EnvGuard {
    fn new(key: &str, value: &str) -> Self {
        let original_value = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, value);
        }
        Self {
            key: key.to_string(),
            original_value,
        }
    }
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

#[test]
fn test_env_check_missing_node() -> Result<()> {
    let _lock = get_env_lock();
    setup_path();

    // Use EnvGuard to safely modify PATH
    let _guard = EnvGuard::new("PATH", "");

    let tools = EncoreTools::new();
    let result = tools.env_check()?;
    let env_info = result.as_object().unwrap();

    // If PATH is empty, node should be missing
    assert!(env_info.get("node_version").unwrap().is_null());

    // details should complain
    let details = env_info.get("details").unwrap().as_array().unwrap();
    assert!(!details.is_empty());

    Ok(())
}
