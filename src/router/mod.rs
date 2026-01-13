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
}

impl Router {
    pub fn new(resolver: Arc<ResolveEngine<RealFs>>, mounts: MountRegistry) -> Self {
        Self { resolver, mounts }
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
