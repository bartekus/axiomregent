#!/bin/bash
set -e

# Build the MCP server
cargo build --quiet

# Define the server binary
SERVER="./target/debug/axiomregent"

# Function to send JSON-RPC request
send_request() {
    local method=$1
    local params=$2
    local id=$3
    
    local payload=$(jq -n --arg method "$method" --argjson params "$params" --arg id "$id" \
        '{jsonrpc: "2.0", method: $method, params: $params, id: $id}')
    
    local len=${#payload}
    printf "Content-Length: %d\r\n\r\n%s" "$len" "$payload" | $SERVER
}

echo ">>> Testing tools/list..."
send_request "tools/list" "{}" "1" | grep -o '"name":"[^"]*"' | sort | uniq

echo -e "\n>>> Testing features.locate (non-existent feature)..."
PARAMS='{
  "name": "features.locate",
  "arguments": {
    "repo_root": "'$(pwd)'",
    "selector_kind": "feature_id",
    "selector_value": "NON_EXISTENT_FEATURE"
  }
}'
send_request "tools/call" "$PARAMS" "2"

echo -e "\n>>> Testing gov.preflight (empty change)..."
PARAMS='{
  "name": "gov.preflight",
  "arguments": {
    "repo_root": "'$(pwd)'",
    "intent": "edit",
    "mode": "worktree",
    "changed_paths": ["Cargo.toml"]
  }
}'
send_request "tools/call" "$PARAMS" "3"

echo -e "\n>>> Testing xray.scan..."
PARAMS='{
  "name": "xray.scan",
  "arguments": {
    "target": "crates/featuregraph"
  }
}'
send_request "tools/call" "$PARAMS" "4"
