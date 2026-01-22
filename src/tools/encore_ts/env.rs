use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvInfo {
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub tsparser_path: Option<String>,
    pub tsbundler_path: Option<String>,
    pub details: Vec<String>,
}

pub fn check() -> Result<EnvInfo> {
    let mut info = EnvInfo {
        node_version: None,
        npm_version: None,
        tsparser_path: None,
        tsbundler_path: None,
        details: Vec::new(),
    };

    // Check Node
    match Command::new("node").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            info.node_version = Some(v);
        }
        _ => info.details.push("Node.js not found".to_string()),
    }

    // Check NPM
    match Command::new("npm").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            info.npm_version = Some(v);
        }
        _ => info.details.push("npm not found".to_string()),
    }

    // Check tsparser-encore
    // We assume it's in PATH or we might look in crates location for dev
    if let Ok(path) = which::which("tsparser-encore") {
        info.tsparser_path = Some(path.to_string_lossy().to_string());
    } else {
        // Fallback for dev environment: check cargo target dir?
        // This is heuristic for dev environment
        let dev_path = std::path::Path::new("target/debug/tsparser-encore");
        if dev_path.exists() {
            info.tsparser_path = Some(dev_path.to_string_lossy().to_string());
        } else {
            info.details.push("tsparser-encore not found".to_string());
        }
    }

    // Check tsbundler-encore (not used directly yet but good to check)
    if let Ok(path) = which::which("tsbundler-encore") {
        info.tsbundler_path = Some(path.to_string_lossy().to_string());
    } else {
        info.details.push("tsbundler-encore not found".to_string());
    }

    Ok(info)
}
