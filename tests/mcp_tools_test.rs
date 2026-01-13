// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus

use axiomregent::io::fs::RealFs;
use axiomregent::resolver::order::ResolveEngine;
use axiomregent::router::mounts::MountRegistry;
use axiomregent::router::{JsonRpcRequest, Router};
use axiomregent::snapshot::{lease::LeaseStore, tools::SnapshotTools};
use axiomregent::workspace::WorkspaceTools;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

// Feature: MCP_TOOLS
// Spec: spec/core/tools.md

#[test]
fn test_mcp_tools_list() {
    let fs = RealFs;
    let resolver = Arc::new(ResolveEngine::new(fs, Vec::<PathBuf>::new()));
    let mounts = MountRegistry::new();

    let dir = tempfile::tempdir().unwrap();
    let config = axiomregent::config::StorageConfig {
        data_dir: dir.path().to_path_buf(),
        blob_backend: axiomregent::config::BlobBackend::Fs,
        compression: axiomregent::config::Compression::None,
    };
    let store = Arc::new(axiomregent::snapshot::store::Store::new(config).unwrap());
    let lease_store = Arc::new(LeaseStore::new());

    let snapshot_tools = Arc::new(SnapshotTools::new(lease_store.clone(), store.clone()));
    let workspace_tools = Arc::new(WorkspaceTools::new(lease_store.clone(), store.clone()));

    let featuregraph_tools = Arc::new(axiomregent::featuregraph::tools::FeatureGraphTools::new());
    let xray_tools = Arc::new(axiomregent::xray::tools::XrayTools::new());
    let router = Router::new(
        resolver,
        mounts,
        snapshot_tools,
        workspace_tools,
        featuregraph_tools,
        xray_tools,
    );

    // Test tools/list
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(json!(1)),
    };
    let resp = router.handle_request(&req);
    assert!(resp.result.is_some());
    let res = resp.result.unwrap();

    let tools = res["tools"].as_array().expect("tools should be an array");

    // Check for required tools
    let required_tools = vec!["resolve_mcp", "list_mounts"];
    for req_tool in required_tools {
        let found = tools.iter().any(|t| t["name"] == req_tool);
        assert!(found, "Tool {} not found", req_tool);
    }
}

#[test]
fn test_mcp_tools_call_validation() {
    let fs = RealFs;
    let resolver = Arc::new(ResolveEngine::new(fs, Vec::<PathBuf>::new()));
    let mounts = MountRegistry::new();

    let dir = tempfile::tempdir().unwrap();
    let config = axiomregent::config::StorageConfig {
        data_dir: dir.path().to_path_buf(),
        blob_backend: axiomregent::config::BlobBackend::Fs,
        compression: axiomregent::config::Compression::None,
    };
    let store = Arc::new(axiomregent::snapshot::store::Store::new(config).unwrap());
    let lease_store = Arc::new(LeaseStore::new());

    let snapshot_tools = Arc::new(SnapshotTools::new(lease_store.clone(), store.clone()));
    let workspace_tools = Arc::new(WorkspaceTools::new(lease_store.clone(), store.clone()));

    let featuregraph_tools = Arc::new(axiomregent::featuregraph::tools::FeatureGraphTools::new());
    let xray_tools = Arc::new(axiomregent::xray::tools::XrayTools::new());
    let router = Router::new(
        resolver,
        mounts,
        snapshot_tools,
        workspace_tools,
        featuregraph_tools,
        xray_tools,
    );

    // Call resolve_mcp without name -> Error
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "resolve_mcp",
            "arguments": {}
        })),
        id: Some(json!(2)),
    };
    let resp = router.handle_request(&req);
    assert!(resp.error.is_some());
    // Expect error about missing argument
}
