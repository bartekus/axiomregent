// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus

use axiomregent::featuregraph::tools::FeatureGraphTools;
use axiomregent::io::fs::RealFs;
use axiomregent::resolver::order::ResolveEngine;
use axiomregent::router::JsonRpcRequest;
use axiomregent::router::Router;
use axiomregent::router::mounts::MountRegistry;
use axiomregent::snapshot::tools::SnapshotTools;
use axiomregent::workspace::WorkspaceTools;
use serde_json::json;
use std::sync::Arc;
// use tempfile::TempDir;

fn create_router() -> Router {
    let fs = RealFs;
    let resolver = Arc::new(ResolveEngine::new(fs, vec![]));
    let mounts = MountRegistry::new();
    let lease_store = Arc::new(axiomregent::snapshot::lease::LeaseStore::new());
    let storage_config = axiomregent::config::StorageConfig::default();
    let store = Arc::new(axiomregent::snapshot::store::Store::new(storage_config).unwrap());

    let snapshot_tools = Arc::new(SnapshotTools::new(lease_store.clone(), store.clone()));
    let workspace_tools = Arc::new(WorkspaceTools::new(lease_store.clone(), store.clone()));
    let featuregraph_tools = Arc::new(FeatureGraphTools::new());

    Router::new(
        resolver,
        mounts,
        snapshot_tools,
        workspace_tools,
        featuregraph_tools,
    )
}

#[test]
fn test_features_overview() {
    let router = create_router();
    let repo_root = std::env::current_dir().unwrap(); // Scan self

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "features.overview",
            "arguments": {
                "repo_root": repo_root.to_string_lossy()
            }
        })),
        id: Some(json!(1)),
    };

    let resp = router.handle_request(&req);
    assert!(resp.error.is_none());

    let result = resp.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let graph_json = content[0].get("json").unwrap();

    // Check if we found FEATUREGRAPH_REGISTRY (ourselves)
    let features = graph_json.get("features").unwrap().as_array().unwrap();
    let found = features
        .iter()
        .any(|f| f.get("feature_id").unwrap().as_str().unwrap() == "FEATUREGRAPH_REGISTRY");

    assert!(found, "Should find FEATUREGRAPH_REGISTRY in overview");
}

#[test]
fn test_features_locate() {
    let router = create_router();
    let repo_root = std::env::current_dir().unwrap();

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "features.locate",
            "arguments": {
                "repo_root": repo_root.to_string_lossy(),
                "feature_id": "FEATUREGRAPH_REGISTRY"
            }
        })),
        id: Some(json!(1)),
    };

    let resp = router.handle_request(&req);
    assert!(resp.error.is_none());

    let result = resp.result.unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let node = content[0].get("json").unwrap();

    assert_eq!(
        node.get("feature_id").unwrap().as_str().unwrap(),
        "FEATUREGRAPH_REGISTRY"
    );
    assert_eq!(
        node.get("spec_path").unwrap().as_str().unwrap(),
        "spec/core/featuregraph.md"
    );
    // Check new fields
    assert_eq!(node.get("owner").unwrap().as_str().unwrap(), "core-team");
    assert_eq!(
        node.get("governance").unwrap().as_str().unwrap(),
        "approved"
    );
}
