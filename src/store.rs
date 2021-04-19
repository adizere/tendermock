//! # Store
//!
//! A storage for tendermock. For now the only available storage is the `InMemoryStore`, which ,as
//! its name implies, is not persisted to the hard drive. However, implementations of
//! persistent storage are possible without impacting the rest of the code base as it only relies
//! on the `Storage` trait, which may be implemented for new kinds of storage in the future.
//!
//! A storage has two jobs:
//!  - persist the state of committed blocks, via the `grow` API.
//!  - update the state of the pending block and access the state for any block,
//!     via a `get` and `set` API.
use std::sync::RwLock;

use crate::avl::AvlTree;
use std::cmp::Ordering;

/// A concurrent, on chain storage using interior mutability.
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

/// An in-memory store backed by a simple hashmap.
pub struct InMemoryStore {
    store: RwLock<Vec<AvlTree<Vec<u8>, Vec<u8>>>>,
    pending: RwLock<AvlTree<Vec<u8>, Vec<u8>>>,
}

impl InMemoryStore {
    /// The store starts out by comprising the state of a single committed block, the genesis
    /// block, at height 1. The pending block is on top of that at height 2.
    pub fn new() -> Self {
        let genesis = AvlTree::new();
        let pending = genesis.clone();

        InMemoryStore {
            store: RwLock::new(vec![genesis]),
            pending: RwLock::new(pending),
        }
    }
}

impl std::fmt::Debug for InMemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let store = self.store.read().unwrap();
        let pending = self.pending.read().unwrap();
        let keys = store.last().unwrap().get_keys();

        write!(
            f,
            "InMemoryStore {{ height: {}, keys: [{}] \n\tpending: [{}] }}",
            store.len(),
            keys.iter()
                .map(|k| String::from_utf8_lossy(k).into_owned())
                .collect::<Vec<String>>()
                .join(", "),
            pending
                .get_keys()
                .iter()
                .map(|k| String::from_utf8_lossy(k).into_owned())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Storage for InMemoryStore {
    fn set(&self, path: Vec<u8>, value: Vec<u8>) {
        let mut store = self.pending.write().unwrap();
        store.insert(path, value);
    }

    /// Implementation details: three cases:
    ///  - height = 0 -> access the store for the last __committed__ block (initially, height 1);
    ///  - height - 1 < store.len() -> access the block n° (height-1);
    ///  - height - 1 == store.len() -> access the pending block.
    fn get(&self, height: u64, path: &[u8]) -> Option<Vec<u8>> {
        let store = self.store.read().unwrap();

        if height == 0 {
            // Access the last committed block
            return store.last().unwrap().get(path).cloned();
        }

        let h = (height - 1) as usize;
        match h.cmp(&store.len()) {
            Ordering::Less => {
                // Access one of the committed blocks
                let state = store.get(h).unwrap();
                state.get(path).cloned()
            }
            Ordering::Equal => {
                // Access the pending blocks
                drop(store); // Release lock
                let pending = self.pending.read().unwrap();
                pending.get(path).cloned()
            }
            Ordering::Greater => None,
        }
    }

    fn grow(&self) {
        let mut store = self.store.write().unwrap();
        let pending = self.pending.write().unwrap();
        let pending_copy = pending.clone();
        store.push(pending_copy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store() {
        let store = InMemoryStore::new();
        test_with_store(store)
    }

    fn test_with_store<T: Storage>(store: T) {
        let data = b"hello";
        let path_bar = b"foo/bar";
        let path_baz = b"foo/baz";

        assert_eq!(store.get(0, path_bar), None);
        assert_eq!(store.get(1000, path_bar), None);

        store.set(path_bar.to_vec(), data.to_vec()); // Set value on pending block (height 2)
        assert_eq!(store.get(0, path_bar), None);
        assert_eq!(store.get(2, path_bar), Some(data.to_vec()));

        store.grow(); // Commit value, will be seen as "last block" (height 2, or 0)
        store.set(path_baz.to_vec(), data.to_vec());

        store.grow(); // Commit value into block height 3
        assert_eq!(store.get(3, path_baz), Some(data.to_vec()));
    }
}
