// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026  Bartek Kus

// use axiomregent::antigravity_tools::AntigravityTools;
// use axiomregent::feature_tools::FeatureTools;
// use axiomregent::snapshot::tools::SnapshotTools;
// use axiomregent::workspace::WorkspaceTools;
// use std::sync::Arc;

#[test]
fn test_tool_instantiation() {
    //     use axiomregent::config::{BlobBackend, Compression, StorageConfig};
    //     use axiomregent::snapshot::lease::LeaseStore;
    //     use axiomregent::snapshot::store::Store;
    //
    //     // Use a temp dir to avoid erroring if dir exists/locked
    //     let dir = tempfile::tempdir().unwrap();
    //     let config = StorageConfig {
    //         data_dir: dir.path().to_path_buf(),
    //         blob_backend: BlobBackend::Fs,
    //         compression: Compression::None,
    //     };
    //
    //     let store = Arc::new(Store::new(config).unwrap());
    //     let lease_store = Arc::new(LeaseStore::new());
    //
    //     let workspace_tools = Arc::new(WorkspaceTools::new(lease_store.clone(), store.clone()));
    //     let snapshot_tools = Arc::new(SnapshotTools::new(lease_store.clone(), store.clone()));
    //     let feature_tools = Arc::new(FeatureTools::new());
    //
    //     let _antigravity = AntigravityTools::new(workspace_tools, snapshot_tools, feature_tools);
}
