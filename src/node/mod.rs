//! # Node
//!
//! A Tendermock `Node` encapsulates a storage and a chain.
//! A `SharedNode` is a thread-safe version of a node, for use by the various RPC interfaces.
//!
//! To integrate with IBC modules, the node implements the `Ics26Context` traits, which mainly deal
//! with storing and reading values from the store.

#![allow(unused_variables)]
mod bare;
mod error;
mod objects;
mod shared;

pub use bare::Node;
pub use error::Error;
pub use objects::Counter;
pub use shared::SharedNode;
