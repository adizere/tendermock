//! # Tendermock
//!
//! A builder API to build and run Tendermock node.
//!
//! ```no_run
//! use tendermock::Tendermock;
//!
//! let jrpc_addr = format!("127.0.0.1:{}", 5000).parse().unwrap();
//! let grpc_addr = format!("127.0.0.1:{}", 6000).parse().unwrap();
//!
//! Tendermock::default()
//!     .growth_rate(10)
//!     .register_node_interface(jrpc_addr, grpc_addr)
//!     .start();
//! ```

#[macro_use]
mod logger;

mod abci;
mod avl;
mod builder;
mod chain;
mod config;
mod grpc;
mod init;
mod jrpc;
mod node;
mod store;
mod test_node;

pub use builder::Tendermock;
