use mcp_spec::protocol::{ErrorData, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::{json, Value};
use crate::messages::JSONRPC_VERSION;

// JSON-RPC Error Codes
pub const JSON_RPC_PARSE_ERROR: i32 = -32700;
pub const JSON_RPC_INVALID_REQUEST: i32 = -32600;
pub const JSON_RPC_METHOD_NOT_FOUND: i32 = -32601;
pub const JSON_RPC_INVALID_PARAMS: i32 = -32602;
pub const JSON_RPC_INTERNAL_ERROR: i32 = -32603;
pub const JSON_RPC_APPLICATION_ERROR_START: i32 = -32000; // Range -32000 to -32099 for app-specific errors

// MCP Error Codes
pub const MCP_INVALID_REQUEST: i32 = 100;
pub const MCP_INVALID_METHOD: i32 = 101;
pub const MCP_INVALID_PARAMS: i32 = 102;
pub const MCP_AUTH_ERROR: i32 = 103;
pub const MCP_TIMEOUT_ERROR: i32 = 104;
pub const MCP_INTERNAL_SERVER_ERROR: i32 = 105;
pub const MCP_SERVICE_UNAVAILABLE: i32 = 106;
pub const MCP_FORBIDDEN_ERROR: i32 = 107;


/// Helper functions for JSON-RPC handling
pub struct JsonRpcUtils;

impl JsonRpcUtils {
    /// Parses raw JSON into a JSON-RPC request
    pub fn parse_request(json_str: &str) -> Result<JsonRpcRequest, JsonRpcError> {
        serde_json::from_str(json_str).map_err(|e| JsonRpcUtils::invalid_request(Some(e.to_string())))
    }

    /// Parses raw JSON into a JSON-RPC message (handles requests and notifications)
    pub fn parse_message(json_str: &str) -> Result<Value, JsonRpcError> {
        serde_json::from_str(json_str).map_err(|e| JsonRpcUtils::invalid_request(Some(e.to_string())))
    }

    /// Serializes a JSON-RPC response into a string
    pub fn serialize_response(response: &JsonRpcResponse) -> String {
        serde_json::to_string(response).unwrap_or_else(|_| "{}".to_string())
    }

    /// Creates a generic JSON-RPC error response
    pub fn error_response(id: Option<u64>, code: i32, message: &str, data: Option<Value>) -> JsonRpcError {
        JsonRpcError {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: id,
            error: ErrorData {
                code,
                message: message.to_string(),
                data,
            },
        }
    }

    /// Returns a predefined error for invalid requests
    pub fn invalid_request(detail: Option<String>) -> JsonRpcError {
        JsonRpcError {
           jsonrpc: JSONRPC_VERSION.to_owned(),
            id: None,
            error: ErrorData {
                code: -32600, // Invalid Request
                message: "Invalid JSON-RPC request".to_string(),
                data: detail.map(|d| json!(d)),
            
            }
        }
    }

    /// Returns a predefined error for method not found
    pub fn method_not_found(id: u64, method: &str) -> JsonRpcError {
        JsonRpcUtils::error_response(Some(id), -32601, &format!("Method '{}' not found", method), None)
    }

    /// Returns a predefined error for internal server errors
    pub fn internal_error(id: Option<u64>, detail: Option<String>) -> JsonRpcError {
        JsonRpcUtils::error_response(id, -1, "Internal server error", detail.map(|d| json!(d)))
    }
}
