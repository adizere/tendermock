//! # JsonRPC
//!
//! This module contains the the utilities needed do build the JsonRPC api, both HTTP and
//! WebSockets, which can then directly served by `warp`.
//!
//! `warp` is a HTTP framework which is built around filters, this module is used to build filters
//! that mimics the Tendermint API.
mod api;
mod utils;
mod websockets;

pub use api::Jrpc;
pub use websockets::Ws;

use crate::node;
use crate::store::Storage;
use futures::future::FutureExt;
use warp::Filter as _;

pub const WEBSOCKET_PATH: &str = "websocket";

/// Create a new gRPC server.
pub async fn serve<S: 'static + Storage + Sync + Send>(
    node: node::SharedNode<S>,
    verbose: bool,
    addr: std::net::SocketAddr,
) -> Result<(), std::convert::Infallible> {
    let jrpc_api = warp::path::end().and(Jrpc::new_mimic(verbose, node));
    let ws = warp::path(WEBSOCKET_PATH).and(Ws::new_mimic());
    warp::serve(jrpc_api.or(ws))
        .run(addr)
        .then(|()| async { Ok(()) })
        .await
}
