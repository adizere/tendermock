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
//! A storage has two kinds of `Location`s:
//!     1. a pending location, which represents the current block being processed, but not yet
//!         committed;
//!     2. a stable location, which is versioned by height.

pub use memory::Memory;

mod memory;

/// Defines a location in a `Storage`.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Location {
    /// Represents the pending location.
    /// This is the location being manipulated by the `set` method.
    Pending,

    /// Represents the location in the stable storage, for the last block.
    LatestStable,

    /// Represents the location in the stable storage, for an arbitrary block.
    Stable(u64),
}

pub struct PathValue {
    pub path: Vec<u8>,
    pub value: Vec<u8>,
}

/// A concurrent storage for on-chain data, using interior mutability.
pub trait Storage: std::fmt::Debug {
    /// Set a value in the store at the `Pending` location.
    /// The storage starts up by having height 0 committed (or `Stable`); consequently the mutable
    /// `Pending` height in the beginning is 1.
    fn set(&self, path: Vec<u8>, value: Vec<u8>);

    /// Access the value at a given path and location.
    /// Returns `None` if nothing found.
    fn get(&self, loc: Location, path: &[u8]) -> Option<Vec<u8>>;

    /// Access the value(s) at a given path prefix and location.
    /// Returns `None` if nothing found.
    fn get_by_prefix(&self, loc: Location, prefix: &[u8]) -> Vec<PathValue>;

    /// Freeze the pending store by adding it to the committed chain, and create a new pending.
    fn grow(&self);
}
