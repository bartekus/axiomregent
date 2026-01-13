// Feature: MCP_TOOLS
// Spec: spec/core/tools.md

// Router module
pub mod cache;
pub mod mounts;

use crate::io::fs::RealFs;
// Feature: MCP_ROUTER
// Spec: spec/core/router.md
use crate::resolver::order::ResolveEngine;
use crate::router::mounts::MountRegistry;
use crate::snapshot::lease::StaleLeaseError;
use crate::snapshot::tools::SnapshotTools;
use crate::workspace::WorkspaceTools;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

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
}

impl Router {
    pub fn new(
        resolver: Arc<ResolveEngine<RealFs>>,
        mounts: MountRegistry,
        snapshot_tools: Arc<SnapshotTools>,
        workspace_tools: Arc<WorkspaceTools>,
    ) -> Self {
        Self {
            resolver,
            mounts,
            snapshot_tools,
            workspace_tools,
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
