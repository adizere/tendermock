use std::cmp::Ordering;
use std::sync::RwLock;

use crate::avl::AvlTree;
use crate::store::Storage;

/// An in-memory store backed by a simple hashmap.
pub struct Memory {
    store: RwLock<Vec<AvlTree<Vec<u8>, Vec<u8>>>>,
    pending: RwLock<AvlTree<Vec<u8>, Vec<u8>>>,
}

impl Memory {
    /// The store starts out by comprising the state of a single committed block, the genesis
    /// block, at height 1. The pending block is on top of that at height 2.
    pub fn new() -> Self {
        let genesis = AvlTree::new();
        let pending = genesis.clone();

        Memory {
            store: RwLock::new(vec![genesis]),
            pending: RwLock::new(pending),
        }
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let store = self.store.read().unwrap();
        let pending = self.pending.read().unwrap();
        let last_store_keys = store.last().unwrap().get_keys();

        write!(
            f,
            "InMemoryStore {{ height: {}, keys: [{}] \n\tpending keys: [{}] }}",
            store.len(),
            last_store_keys
                .iter()
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

impl Storage for Memory {
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
    use crate::store::memory::Memory;
    use crate::store::*;

    #[test]
    fn store() {
        let store = Memory::new();
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
