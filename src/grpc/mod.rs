//! # gRPC
//!
//! The gRPC interface of tendermock, for now most of the queries are unimplemented.
//!
//! The serialization is handled by [prost](https://github.com/danburkert/prost), a gRPC framework that generates all the
//! desialization/deserialization code from protobuf files. The protobuf files and generated Rust
//! code lives in the `ibc_proto` crate.
//!
//! The server code is also generated, this time by [tonic](https://github.com/hyperium/tonic) and it also lives in the `ibc_proto`
//! crate. This module simply implements the `Query` trait generated by `Tonic` on a custom
//! `QueryService` struct.
use crate::logger::Log;
use crate::node;
use crate::store::Storage;
use futures::future::FutureExt;
use tonic::transport::Server;

mod auth;
mod staking;

/// Create a new gRPC server.
pub async fn serve<S: 'static + Storage + Sync + Send>(
    node: node::SharedNode<S>,
    verbose: bool,
    addr: std::net::SocketAddr,
) -> Result<(), std::convert::Infallible> {
    Server::builder()
        .add_service(staking::get_service(node.clone(), verbose))
        .add_service(auth::get_service(node, verbose))
        .serve(addr)
        .then(|result| async {
            if let Err(e) = result {
                log!(Log::GRPC, "Server error: {}", e);
            }
            Ok(())
        })
        .await
}
