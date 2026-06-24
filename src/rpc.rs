use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

const REQUEST_ID: u64 = 1;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("RPC request failed")]
    Transport,
    #[error("RPC endpoint returned HTTP status {0}")]
    Http(StatusCode),
    #[error("RPC endpoint returned invalid JSON")]
    InvalidJson,
    #[error("RPC response was invalid: {0}")]
    InvalidResponse(&'static str),
    #[error("RPC error {code}: {message}")]
    Protocol { code: i64, message: String },
}

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: &'a Value,
}

#[derive(Deserialize)]
struct RpcResponse {
    jsonrpc: Option<String>,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<RpcResponseError>,
}

#[derive(Deserialize)]
struct RpcResponseError {
    code: i64,
    message: String,
}

pub async fn call(
    client: &Client,
    url: Url,
    method: &str,
    params: &Value,
) -> Result<Value, RpcError> {
    let request = RpcRequest {
        jsonrpc: "2.0",
        id: REQUEST_ID,
        method,
        params,
    };

    let response = client
        .post(url)
        .json(&request)
        .send()
        .await
        .map_err(|_| RpcError::Transport)?;

    if !response.status().is_success() {
        return Err(RpcError::Http(response.status()));
    }

    let body: RpcResponse = response.json().await.map_err(|_| RpcError::InvalidJson)?;

    if body.jsonrpc.as_deref() != Some("2.0") {
        return Err(RpcError::InvalidResponse("missing JSON-RPC 2.0 marker"));
    }
    if body.id != Some(Value::from(REQUEST_ID)) {
        return Err(RpcError::InvalidResponse(
            "response ID did not match request",
        ));
    }
    if let Some(error) = body.error {
        return Err(RpcError::Protocol {
            code: error.code,
            message: error.message,
        });
    }

    body.result
        .ok_or(RpcError::InvalidResponse("response contained no result"))
}
