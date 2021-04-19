//! The Tendermock JsonRPC Websocket API.
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tendermint_rpc::endpoint::subscribe::{Request, Response};
use warp::ws::{Message, WebSocket, Ws as WarpWs};
use warp::Filter;

use super::utils::{JrpcEnvelope, JrpcError, JrpcResponse, JrpcResult, JRPC_VERSION};
use crate::logger::Log;

/// A struct that can be used to build the Websocket `warp` filter, see the `new` method.
pub struct Ws {}

impl Ws {
    /// Creates a `warp` filter that mimics the Tendermint Websocket API.
    pub fn new_mimic() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
        warp::ws().map(|ws: WarpWs| ws.on_upgrade(handler)).boxed()
    }
}

/// Handle a websocket connection.
async fn handler(ws: WebSocket) {
    log!(Log::Websocket, "Connection");
    let (mut sending_ws, mut listening_ws) = ws.split();
    while let Some(result) = listening_ws.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log!(Log::Websocket, "Receiving error: '{}'", e);
                break;
            }
        };
        let msg = if let Ok(msg) = msg.to_str() {
            msg
        } else {
            log!(Log::Websocket, "Could not interpret message as str");
            break;
        };
        if let Err(e) = sending_ws.send(Message::text(handle_request(msg))).await {
            log!(Log::Websocket, "Sending error: '{}'", e);
            break;
        };
    }
    if let Ok(ws) = sending_ws.reunite(listening_ws) {
        if let Err(e) = ws.close().await {
            log!(Log::Websocket, "Closing error: '{}'", e);
        };
    };
}

/// Parses the request, dispatch the query to the method handler and return the serialized Jrpc
/// response.
fn handle_request(msg: &str) -> String {
    let req = if let Ok(req) = parse_message(msg) {
        req
    } else {
        return serde_json::to_string(&JrpcResponse::<()> {
            id: "0".to_string(),
            jsonrpc: JRPC_VERSION.to_string(),
            error: Some(JrpcError::InvalidRequest.into()),
            result: None,
        })
        .unwrap();
    };
    if req.jsonrpc != JRPC_VERSION {
        return serialize_response::<()>(Err(JrpcError::WrongVersion), req);
    }
    match req.method.as_str() {
        "subscribe" => {
            let res = subscribe(&req);
            serialize_response(res, req)
        }
        _ => serialize_response::<()>(Err(JrpcError::WrongMethod), req),
    }
}

/// Parses the websocket message into a JsonRPC request.
fn parse_message(msg: &str) -> JrpcResult<JrpcEnvelope> {
    serde_json::from_str(msg).map_err(|_| JrpcError::InvalidRequest)
}

/// Serializes a JrpcResult into an actual JsonRPC response String.
fn serialize_response<T: Serialize>(result: JrpcResult<T>, envelope: JrpcEnvelope) -> String {
    let (error, result) = match result {
        Ok(res) => (None, Some(res)),
        Err(e) => (Some(e.into()), None),
    };
    serde_json::to_string(&JrpcResponse {
        id: envelope.id,
        jsonrpc: envelope.jsonrpc,
        error,
        result,
    })
    .unwrap()
}

/// Handles the /subscribe endpoint.
fn subscribe(req: &JrpcEnvelope) -> JrpcResult<Response> {
    let _params: Request =
        serde_json::from_value(req.params.clone()).map_err(|_| JrpcError::WrongParameters)?;
    Ok(Response {})
}
