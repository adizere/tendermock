//! # Store
//!
//! A storage trait for Tendermock `Node`s.
//!
//! For now the only available storage is the `InMemoryStore`.
//! As its name implies, this resides in volatile memory. However, implementations of
//! persistent storage are possible without impacting the rest of the code base as it only relies
//! on the `Storage` trait, which may be implemented for new kinds of storage in the future.
//!
//! A storage has two jobs:
//!  - persist the state of committed blocks, via the `grow` API.
//!  - update the state of the pending block and access the state for any block,
//!     via a `get` and `set` API.

pub use memory::Memory;

mod memory;

/// A concurrent storage for on-chain data, using interior mutability.
pub trait Storage: std::fmt::Debug {
    /// Set a value in the store at the last (pending) height.
    /// The storage starts up by having height 1 committed (or stable); consequently the mutable
    /// (pending) height in the beginning is 2.
    fn set(&self, path: Vec<u8>, value: Vec<u8>);

    /// Access the value at a given path and height.
    /// Returns `None` if no block matches `height`.
    /// If height = 0, then it accesses the store for the last committed block (initially, this is
    /// height 1).
    fn get(&self, height: u64, path: &[u8]) -> Option<Vec<u8>>;

    /// Freeze the pending store by adding it to the committed chain and create a new pending.
    fn grow(&self);
}
