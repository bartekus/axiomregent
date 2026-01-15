// Feature: MCP_TOOLS
// Spec: spec/core/tools.md

// Router module
pub mod cache;
pub mod mounts;

use crate::io::fs::RealFs;
// Feature: MCP_ROUTER
// Spec: spec/core/router.md
use crate::antigravity_tools::AntigravityTools;
use crate::resolver::order::ResolveEngine;
use crate::router::mounts::MountRegistry;
use crate::run_tools::RunTools;
use crate::snapshot::lease::StaleLeaseError;
use crate::snapshot::tools::SnapshotTools;
use crate::tools::encore_ts::tools::EncoreTools;
use crate::workspace::WorkspaceTools;
use featuregraph::tools::FeatureGraphTools;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use xray::tools::XrayTools;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub id: Option<Value>,
}

/// AxiomRegentError represents MCP-level errors that are surfaced to clients using
/// string error codes defined by the MCP common schema.
///
/// NOTE: We intentionally avoid adding new dependencies here (e.g. `thiserror`).
#[derive(Debug)]
pub enum AxiomRegentError {
    NotFound(String),
    InvalidArgument(String),
    RepoChanged(String),
    PermissionDenied(String),
    TooLarge(String),
    Internal(String),
}

impl AxiomRegentError {
    pub fn code(&self) -> &'static str {
        match self {
            AxiomRegentError::NotFound(_) => "NOT_FOUND",
            AxiomRegentError::InvalidArgument(_) => "INVALID_ARGUMENT",
            AxiomRegentError::RepoChanged(_) => "REPO_CHANGED",
            AxiomRegentError::PermissionDenied(_) => "PERMISSION_DENIED",
            AxiomRegentError::TooLarge(_) => "TOO_LARGE",
            AxiomRegentError::Internal(_) => "INTERNAL",
        }
    }

    fn message(&self) -> &str {
        match self {
            AxiomRegentError::NotFound(m)
            | AxiomRegentError::InvalidArgument(m)
            | AxiomRegentError::RepoChanged(m)
            | AxiomRegentError::PermissionDenied(m)
            | AxiomRegentError::TooLarge(m)
            | AxiomRegentError::Internal(m) => m.as_str(),
        }
    }
}

impl std::fmt::Display for AxiomRegentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for AxiomRegentError {}

pub struct Router {
    resolver: Arc<ResolveEngine<RealFs>>,
    mounts: MountRegistry,
    snapshot_tools: Arc<SnapshotTools>,
    workspace_tools: Arc<WorkspaceTools>,
    featuregraph_tools: Arc<FeatureGraphTools>,
    xray_tools: Arc<XrayTools>,
    antigravity_tools: Arc<AntigravityTools>,
    encore_tools: Arc<EncoreTools>,
    run_tools: Arc<RunTools>,
}

impl Router {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        resolver: Arc<ResolveEngine<RealFs>>,
        mounts: MountRegistry,
        snapshot_tools: Arc<SnapshotTools>,
        workspace_tools: Arc<WorkspaceTools>,
        featuregraph_tools: Arc<FeatureGraphTools>,
        xray_tools: Arc<XrayTools>,
        antigravity_tools: Arc<AntigravityTools>,
        encore_tools: Arc<EncoreTools>,
        run_tools: Arc<RunTools>,
    ) -> Self {
        Self {
            resolver,
            mounts,
            snapshot_tools,
            workspace_tools,
            featuregraph_tools,
            xray_tools,
            antigravity_tools,
            encore_tools,
            run_tools,
        }
    }

    pub fn handle_request(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "initialize" => json_rpc_ok(
                req.id.clone(),
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": get_server_capabilities(),
                    "serverInfo": { "name": "mcp", "version": "0.1.0" }
                }),
            ),
            "tools/list" => json_rpc_ok(
                req.id.clone(),
                json!({
                    "tools": [
                        {
                            "name": "resolve_mcp",
                            "description": "Resolve an MCP server name to a local path or alias",
                            "inputSchema": {
                                "type": "object",
                                "properties": { "name": { "type": "string" } },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "list_mounts",
                            "description": "List currently resolved/mounted servers",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                         {
                            "name": "get_capabilities",
                            "description": "Get server capabilities",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        // FeatureGraph Tools
                        {
                            "name": "features.overview",
                            "description": "Get full feature graph",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        {
                            "name": "features.locate",
                            "description": "Locate feature definition or impl",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "feature_id": { "type": "string" },
                                    "spec_path": { "type": "string" },
                                    "file_path": { "type": "string" }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        // Governance Tools
                        {
                            "name": "gov.preflight",
                            "description": "Check governance policy for proposed changes",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "intent": { "type": "string", "enum": ["edit", "create", "delete", "refactor"] },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "changed_paths": { "type": "array", "items": { "type": "string" } },
                                    "snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root", "intent", "mode", "changed_paths"]
                            }
                        },
                        {
                            "name": "gov.drift",
                            "description": "Check for drift and violations",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        // Xray Tools
                        {
                            "name": "xray.scan",
                            "description": "Scan repository to build index",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        // Snapshot Tools
                        {
                            "name": "snapshot.list",
                            "description": "List files in a snapshot or worktree",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "lease_id": { "type": "string" },
                                    "snapshot_id": { "type": "string" },
                                    "limit": { "type": "integer" },
                                    "offset": { "type": "integer" }
                                },
                                "required": ["repo_root", "path", "mode"]
                            }
                        },
                        {
                            "name": "snapshot.create",
                            "description": "Create a new snapshot",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "lease_id": { "type": "string" },
                                    "paths": { "type": "array", "items": { "type": "string" } }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        {
                            "name": "snapshot.read",
                            "description": "Read file content",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "lease_id": { "type": "string" },
                                    "snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root", "path", "mode"]
                            }
                        },
                        {
                            "name": "snapshot.grep",
                            "description": "Search for patterns",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "pattern": { "type": "string" },
                                    "paths": { "type": "array", "items": { "type": "string" } },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "lease_id": { "type": "string" },
                                    "snapshot_id": { "type": "string" },
                                    "case_insensitive": { "type": "boolean" }
                                },
                                "required": ["repo_root", "pattern", "mode"]
                            }
                        },
                        {
                            "name": "snapshot.diff",
                            "description": "Generate unified diff",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "lease_id": { "type": "string" },
                                    "snapshot_id": { "type": "string" },
                                    "from_snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root", "path", "mode"]
                            }
                        },
                        {
                            "name": "snapshot.changes",
                            "description": "List changed files between snapshots",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "snapshot_id": { "type": "string" },
                                    "from_snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root", "snapshot_id"]
                            }
                        },
                         {
                            "name": "snapshot.export",
                            "description": "Export snapshot as tarball",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root", "snapshot_id"]
                            }
                        },
                        {
                            "name": "snapshot.info",
                            "description": "Get snapshot or repository info",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "snapshot_id": { "type": "string" }
                                },
                                "required": ["repo_root"]
                            }
                        },
                        // Antigravity Tools
                        {
                            "name": "antigravity.propose",
                            "description": "Propose a change",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "subject": { "type": "string" },
                                    "repo_key": { "type": "string" },
                                    "base_state": { "type": "string" },
                                    "goal": { "type": "string" },
                                    "tasks": {
                                        "type": "array",
                                        "items": { "type": "object" }
                                    },
                                    "tiers": {
                                        "type": "array",
                                        "items": { "type": "string" }
                                    },
                                    "architecture_doc": { "type": "string" },
                                    "base_state_created_at": { "type": "string" }
                                },
                                "required": ["repo_root", "subject", "repo_key", "goal", "tasks"]
                            }
                        },
                        {
                            "name": "antigravity.execute",
                            "description": "Execute a changeset",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "changeset_id": { "type": "string" }
                                },
                                "required": ["repo_root", "changeset_id"]
                            }
                        },
                        {
                            "name": "antigravity.verify",
                            "description": "Verify a changeset",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "changeset_id": { "type": "string" },
                                    "profile": { "type": "string" }
                                },
                                "required": ["repo_root", "changeset_id"]
                            }
                        },
                        // Encore TS Tools
                        {
                            "name": "encore.ts.env.check",
                            "description": "Check Encore TS environment",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        {
                            "name": "encore.ts.parse",
                            "description": "Parse Encore TS application",
                             "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "root": { "type": "string" }
                                },
                                "required": ["root"]
                            }
                        },
                        {
                            "name": "encore.ts.meta",
                            "description": "Get Encore TS application metadata",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "root": { "type": "string" }
                                },
                                "required": ["root"]
                            }
                        },
                        {
                            "name": "encore.ts.run.start",
                            "description": "Start Encore TS application",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "root": { "type": "string" },
                                    "env": { "type": "object" },
                                    "profile": { "type": "string" }
                                },
                                "required": ["root"]
                            }
                        },
                        {
                            "name": "encore.ts.run.stop",
                            "description": "Stop Encore TS application",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "run_id": { "type": "string" }
                                },
                                "required": ["run_id"]
                            }
                        },
                        {
                            "name": "encore.ts.logs.stream",
                            "description": "Stream logs from Encore TS application",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "run_id": { "type": "string" },
                                    "from_seq": { "type": "integer" }
                                },
                                "required": ["run_id"]
                            }
                        },
                        // Run Tools
                        {
                            "name": "run.execute",
                            "description": "Execute a run skill",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "skill": { "type": "string" },
                                    "env": { "type": "object" }
                                },
                                "required": ["skill"]
                            }
                        },
                        {
                            "name": "run.status",
                            "description": "Get run status",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "run_id": { "type": "string" }
                                },
                                "required": ["run_id"]
                            }
                        },
                        {
                            "name": "run.logs",
                            "description": "Get run logs",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "run_id": { "type": "string" }
                                },
                                "required": ["run_id"]
                            }
                        },
                        // Workspace Tools
                        {
                            "name": "workspace.write_file",
                            "description": "Write file content",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" },
                                    "content_base64": { "type": "string" },
                                    "lease_id": { "type": "string" },
                                    "create_dirs": { "type": "boolean" },
                                    "dry_run": { "type": "boolean" }
                                },
                                "required": ["repo_root", "path", "content_base64", "lease_id"]
                            }
                        },
                        {
                            "name": "workspace.delete",
                            "description": "Delete a file or directory",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "path": { "type": "string" },
                                    "lease_id": { "type": "string" },
                                    "dry_run": { "type": "boolean" }
                                },
                                "required": ["repo_root", "path", "lease_id"]
                            }
                        },
                        {
                            "name": "workspace.apply_patch",
                            "description": "Apply a patch",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_root": { "type": "string" },
                                    "patch": { "type": "string" },
                                    "mode": { "type": "string", "enum": ["worktree", "snapshot"] },
                                    "lease_id": { "type": "string" },
                                    "snapshot_id": { "type": "string" },
                                    "strip": { "type": "integer" },
                                    "reject_on_conflict": { "type": "boolean" },
                                    "dry_run": { "type": "boolean" }
                                },
                                "required": ["repo_root", "patch", "mode"]
                            }
                        }
                    ]
                }),
            ),

            "tools/call" => {
                let params = match req.params.as_ref().and_then(|p| p.as_object()) {
                    Some(p) => p,
                    None => return json_rpc_error(req.id.clone(), -32602, "Invalid params"),
                };
                let name = match params.get("name").and_then(|n| n.as_str()) {
                    Some(n) => n,
                    None => return json_rpc_error(req.id.clone(), -32602, "Missing tool name"),
                };
                let args = params.get("arguments").and_then(|a| a.as_object());
                let args = match args {
                    Some(a) => a,
                    None => return json_rpc_error(req.id.clone(), -32602, "Missing arguments"),
                };

                match name {
                    "resolve_mcp" => {
                        let target = args.get("name").and_then(|n| n.as_str());
                        if let Some(target) = target {
                            match self.resolver.resolve(target) {
                                Ok(resp) => {
                                    if resp.status
                                        == crate::protocol::types::ResolveStatus::Resolved
                                        && let (Some(root), Some(rid)) =
                                            (&resp.root, &resp.resolved_id)
                                    {
                                        self.mounts.register(crate::router::mounts::Mount {
                                            name: target.to_string(),
                                            root: root.clone(),
                                            resolved_id: Some(rid.clone()),
                                            kind: resp.kind.clone(),
                                            capabilities: resp.capabilities.clone(),
                                        });
                                    }
                                    let content = json!([{ "type": "json", "json": resp }]);
                                    json_rpc_ok(req.id.clone(), json!({ "content": content }))
                                }
                                Err(e) => json_rpc_error(
                                    req.id.clone(),
                                    -32603,
                                    &format!("Resolution failed: {}", e),
                                ),
                            }
                        } else {
                            json_rpc_error(req.id.clone(), -32602, "Missing name argument")
                        }
                    }
                    "list_mounts" => {
                        let list = self.mounts.list();
                        json_rpc_ok(
                            req.id.clone(),
                            json!({ "content": [{ "type": "json", "json": list }] }),
                        )
                    }
                    "get_capabilities" => {
                        let caps = json!({
                            "name": "mcp",
                            "server_capabilities": get_server_capabilities(),
                        });
                        json_rpc_ok(
                            req.id.clone(),
                            json!({ "content": [{ "type": "json", "json": caps }] }),
                        )
                    }

                    // --- FeatureGraph Tools ---
                    "features.overview" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        match self
                            .featuregraph_tools
                            .features_overview(repo_root, snapshot_id)
                        {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }
                    "features.locate" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let feature_id = args
                            .get("feature_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let spec_path = args
                            .get("spec_path")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let file_path = args
                            .get("file_path")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        match self
                            .featuregraph_tools
                            .features_locate(repo_root, feature_id, spec_path, file_path)
                        {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }

                    "gov.preflight" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };

                        match self
                            .featuregraph_tools
                            .governance_preflight(repo_root, Value::Object(args.clone()))
                        {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }
                    "gov.drift" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        match self.featuregraph_tools.governance_drift(repo_root) {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }

                    // --- Xray Tools ---
                    "xray.scan" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = args.get("path").and_then(|v| v.as_str()).map(String::from);

                        match self.xray_tools.xray_scan(repo_root, path) {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }

                    // --- Antigravity Tools ---
                    "antigravity.propose" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        // We need to parse everything into AgentConfig.
                        // Ideally we deserialize `args` directly to AgentConfig,
                        // but `args` is Map<String, Value>.
                        // Let's use serde_json::from_value.
                        let config_res: Result<antigravity::agent::AgentConfig, _> =
                            serde_json::from_value(Value::Object(args.clone()));

                        match config_res {
                            Ok(config) => match self.antigravity_tools.propose(repo_root, config) {
                                Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                                Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                            },
                            Err(e) => json_rpc_error(
                                req.id.clone(),
                                -32602,
                                &format!("Invalid AgentConfig: {}", e),
                            ),
                        }
                    }
                    "antigravity.execute" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let changeset_id = match args.get("changeset_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "changeset_id required",
                                );
                            }
                        };

                        match self.antigravity_tools.execute(repo_root, changeset_id) {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }
                    "antigravity.verify" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let changeset_id = match args.get("changeset_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "changeset_id required",
                                );
                            }
                        };
                        let profile = args.get("profile").and_then(|v| v.as_str()).unwrap_or("pr");

                        match self
                            .antigravity_tools
                            .verify(repo_root, changeset_id, profile)
                        {
                            Ok(val) => handle_tool_result_value(req.id.clone(), Ok(val)),
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }

                    // --- Encore TS Tools ---
                    "encore.ts.env.check" => {
                        handle_tool_result_value(req.id.clone(), self.encore_tools.env_check())
                    }
                    "encore.ts.parse" => {
                        let root = match args.get("root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => return json_rpc_error(req.id.clone(), -32602, "root required"),
                        };
                        handle_tool_result_value(req.id.clone(), self.encore_tools.parse(root))
                    }
                    "encore.ts.meta" => {
                        let root = match args.get("root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => return json_rpc_error(req.id.clone(), -32602, "root required"),
                        };
                        handle_tool_result_value(req.id.clone(), self.encore_tools.meta(root))
                    }
                    "encore.ts.run.start" => {
                        let root = match args.get("root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => return json_rpc_error(req.id.clone(), -32602, "root required"),
                        };
                        // Note: env and profile parsing omitted for skeleton, just passing placeholders or improving later
                        // Actually, I should parse them properly to match the signature.
                        // But for now, let's keep it minimal as tools.rs implementation is just Err.
                        // I'll assume Env is Option<HashMap<String, String>>
                        // For now I'll just pass None for optional args to keep it simple in this skeleton phase.
                        // Or better, let's just parse them if present.

                        let env = args.get("env").and_then(|v| v.as_object()).map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        });
                        let profile = args
                            .get("profile")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.encore_tools.run_start(root, env, profile),
                        )
                    }
                    "encore.ts.run.stop" => {
                        let run_id = match args.get("run_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "run_id required");
                            }
                        };
                        handle_tool_result_value(req.id.clone(), self.encore_tools.run_stop(run_id))
                    }
                    "encore.ts.logs.stream" => {
                        let run_id = match args.get("run_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "run_id required");
                            }
                        };
                        let from_seq = args.get("from_seq").and_then(|v| v.as_u64());
                        handle_tool_result_value(
                            req.id.clone(),
                            self.encore_tools.logs_stream(run_id, from_seq),
                        )
                    }

                    // --- Run Tools ---
                    "run.execute" => {
                        let skill = match args.get("skill").and_then(|v| v.as_str()) {
                            Some(v) => v.to_string(),
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "skill required");
                            }
                        };
                        let env = args.get("env").and_then(|v| v.as_object()).map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        });

                        match self.run_tools.execute(skill, env) {
                            Ok(val) => {
                                handle_tool_result_value(req.id.clone(), Ok(Value::String(val)))
                            }
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }
                    "run.status" => {
                        let run_id = match args.get("run_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "run_id required");
                            }
                        };
                        match self.run_tools.status(run_id) {
                            Ok(val) => {
                                handle_tool_result_value(req.id.clone(), Ok(Value::String(val)))
                            }
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }
                    "run.logs" => {
                        let run_id = match args.get("run_id").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "run_id required");
                            }
                        };
                        match self.run_tools.logs(run_id) {
                            Ok(val) => {
                                handle_tool_result_value(req.id.clone(), Ok(Value::String(val)))
                            }
                            Err(e) => handle_tool_result_value(req.id.clone(), Err(e)),
                        }
                    }

                    // --- Snapshot Tools Call Handlers ---
                    "snapshot.list" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        let mode = match args.get("mode").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "mode required"),
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let limit = args
                            .get("limit")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize);
                        let offset = args
                            .get("offset")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_list(
                                repo_root,
                                path,
                                mode,
                                lease_id,
                                snapshot_id,
                                limit,
                                offset,
                            ),
                        )
                    }
                    "snapshot.create" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let paths = args.get("paths").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        });

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools
                                .snapshot_create(repo_root, lease_id, paths),
                        )
                    }
                    "snapshot.read" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = match args.get("path").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "path required"),
                        };
                        let mode = match args.get("mode").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "mode required"),
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_file(
                                repo_root,
                                path,
                                mode,
                                lease_id,
                                snapshot_id,
                            ),
                        )
                    }
                    "snapshot.grep" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "pattern required");
                            }
                        };
                        let mode = match args.get("mode").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "mode required"),
                        };
                        let paths = args.get("paths").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        });
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let case_insensitive = args
                            .get("case_insensitive")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_grep(
                                repo_root,
                                pattern,
                                paths,
                                mode,
                                lease_id,
                                snapshot_id,
                                case_insensitive,
                            ),
                        )
                    }
                    "snapshot.diff" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = match args.get("path").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "path required"),
                        };
                        let mode = match args.get("mode").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "mode required"),
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let from_snapshot_id = args
                            .get("from_snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_diff(
                                repo_root,
                                path,
                                mode,
                                lease_id,
                                snapshot_id,
                                from_snapshot_id,
                            ),
                        )
                    }
                    "snapshot.changes" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let from_snapshot_id = args
                            .get("from_snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_changes(
                                repo_root,
                                snapshot_id,
                                from_snapshot_id,
                            ),
                        )
                    }
                    "snapshot.export" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_export(repo_root, snapshot_id),
                        )
                    }
                    "snapshot.info" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.snapshot_tools.snapshot_info(repo_root, snapshot_id),
                        )
                    }

                    // --- Workspace Tools Call Handlers ---
                    "workspace.write_file" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = match args.get("path").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "path required"),
                        };
                        let content_base64 =
                            match args.get("content_base64").and_then(|v| v.as_str()) {
                                Some(v) => v,
                                None => {
                                    return json_rpc_error(
                                        req.id.clone(),
                                        -32602,
                                        "content_base64 required",
                                    );
                                }
                            };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let create_dirs = args
                            .get("create_dirs")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let dry_run = args
                            .get("dry_run")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        handle_tool_result_bool(
                            req.id.clone(),
                            self.workspace_tools.write_file(
                                repo_root,
                                path,
                                content_base64,
                                lease_id,
                                create_dirs,
                                dry_run,
                            ),
                        )
                    }
                    "workspace.delete" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let path = match args.get("path").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "path required"),
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let dry_run = args
                            .get("dry_run")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        handle_tool_result_bool(
                            req.id.clone(),
                            self.workspace_tools
                                .delete(repo_root, path, lease_id, dry_run),
                        )
                    }
                    "workspace.apply_patch" => {
                        let repo_root = match args.get("repo_root").and_then(|v| v.as_str()) {
                            Some(v) => std::path::Path::new(v),
                            None => {
                                return json_rpc_error(
                                    req.id.clone(),
                                    -32602,
                                    "repo_root required",
                                );
                            }
                        };
                        let patch = match args.get("patch").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => {
                                return json_rpc_error(req.id.clone(), -32602, "patch required");
                            }
                        };
                        let mode = match args.get("mode").and_then(|v| v.as_str()) {
                            Some(v) => v,
                            None => return json_rpc_error(req.id.clone(), -32602, "mode required"),
                        };
                        let lease_id = args
                            .get("lease_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let snapshot_id = args
                            .get("snapshot_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let strip = args
                            .get("strip")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize);
                        let reject_on_conflict = args
                            .get("reject_on_conflict")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let dry_run = args
                            .get("dry_run")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        handle_tool_result_value(
                            req.id.clone(),
                            self.workspace_tools.apply_patch(
                                repo_root,
                                patch,
                                mode,
                                lease_id,
                                snapshot_id,
                                strip,
                                reject_on_conflict,
                                dry_run,
                            ),
                        )
                    }
                    _ => {
                        json_rpc_error(req.id.clone(), -32601, &format!("Tool not found: {}", name))
                    }
                }
            }
            _ => json_rpc_error(req.id.clone(), -32601, "Method not found"),
        }
    }
}

fn json_rpc_ok(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    }
}

fn json_rpc_error(id: Option<Value>, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(json!({
            "code": code,
            "message": message
        })),
        id,
    }
}

fn get_server_capabilities() -> Value {
    json!({
        "tools": {
            "listChanged": true
        },
        "logging": {}
    })
}

fn handle_tool_result_value(id: Option<Value>, result: anyhow::Result<Value>) -> JsonRpcResponse {
    match result {
        Ok(val) => json_rpc_ok(id, json!({ "content": [{ "type": "json", "json": val }] })),
        Err(e) => handle_tool_error(id, e),
    }
}

fn handle_tool_result_bool(id: Option<Value>, result: anyhow::Result<bool>) -> JsonRpcResponse {
    match result {
        Ok(true) => json_rpc_ok(id, json!({ "content": [{ "type": "text", "text": "ok" }] })),
        Ok(false) => json_rpc_error(id, -32603, "Tool returned false"),
        Err(e) => handle_tool_error(id, e),
    }
}

fn handle_tool_error(id: Option<Value>, e: anyhow::Error) -> JsonRpcResponse {
    if let Some(stale) = e.downcast_ref::<StaleLeaseError>() {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(json!({
                "code": "STALE_LEASE",
                "message": stale.msg,
                "data": {
                    "lease_id": stale.lease_id,
                    "current_fingerprint": stale.current_fingerprint
                }
            })),
            id,
        };
    }
    json_rpc_error(id, -32603, &format!("Tool failed: {}", e))
}
