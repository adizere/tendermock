use std::sync::RwLock;

use crate::avl::AvlTree;
use crate::store::{Location, Storage};

/// An in-memory store backed by a simple hashmap.
pub struct Memory {
    store: RwLock<Vec<AvlTree<Vec<u8>, Vec<u8>>>>,
    pending: RwLock<AvlTree<Vec<u8>, Vec<u8>>>,
}

impl Memory {
    /// The store starts out by comprising the state of a single committed block, the genesis
    /// block, at height 0, with an empty state. We also initialize the pending location as empty.
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
            "store::Memory {{ height: {}, keys: [{}] \n\tpending keys: [{}] }}",
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
    ///         becomes LatestStable
    ///  - height - 1 < store.len() -> access the block n° (height-1);
    ///         becomes Stable(_)
    ///  - height - 1 == store.len() -> access the pending block.
    ///         becomes Pending
    fn get(&self, loc: Location, path: &[u8]) -> Option<Vec<u8>> {
        let store = self.store.read().unwrap();

        match loc {
            // Request to access the pending blocks
            Location::Pending => {
                drop(store); // Release lock on the stable store
                let pending = self.pending.read().unwrap();
                pending.get(path).cloned()
            }
            // Access the last committed block
            Location::LatestStable => {
                // Access the last committed block
                return store.last().unwrap().get(path).cloned();
            }
            // Access one of the committed blocks
            Location::Stable(height) => {
                let h = height as usize;
                if h < store.len() {
                    let state = store.get(h).unwrap();
                    state.get(path).cloned()
                } else {
                    None
                }
            }
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
    use crate::store::Location;
    use crate::store::{Memory, Storage};

    #[test]
    fn store() {
        let store = Memory::new();
        test_with_store(store)
    }

    fn test_with_store<T: Storage>(store: T) {
        let data1 = b"hello";
        let data2 = b"hello2";
        let path_bar = b"foo/bar";
        let path_baz = b"foo/baz";

        // There should be nothing
        assert_eq!(store.get(Location::LatestStable, path_bar), None);
        assert_eq!(store.get(Location::Pending, path_bar), None);
        assert_eq!(store.get(Location::Stable(800), path_bar), None);
        assert_eq!(store.get(Location::Stable(0), path_bar), None);

        store.set(path_bar.to_vec(), data1.to_vec()); // Set value on pending block
        for height in 0..5 {
            assert_eq!(store.get(Location::Stable(height), path_bar), None);
        }
        assert_eq!(store.get(Location::LatestStable, path_bar), None);
        assert_eq!(store.get(Location::Pending, path_bar), Some(data1.to_vec()));

        store.grow(); // Commit value, will be seen as "last block" (in Stable(1), or LatestStable)
        assert_eq!(
            store.get(Location::LatestStable, path_bar),
            Some(data1.to_vec())
        );
        assert_eq!(store.get(Location::Stable(0), path_bar), None);
        assert_eq!(
            store.get(Location::Stable(1), path_bar),
            Some(data1.to_vec())
        );
        for height in 2..10 {
            assert_eq!(store.get(Location::Stable(height), path_bar), None);
        }

        store.set(path_baz.to_vec(), data1.to_vec());
        store.grow(); // Commit value into the stable location at height 2
        assert_eq!(store.get(Location::Stable(0), path_baz), None);
        assert_eq!(store.get(Location::Stable(1), path_baz), None);
        assert_eq!(
            store.get(Location::LatestStable, path_baz),
            Some(data1.to_vec())
        );
        assert_eq!(
            store.get(Location::Stable(2), path_baz),
            Some(data1.to_vec())
        );
        assert_eq!(store.get(Location::Pending, path_baz), Some(data1.to_vec()));
        assert_eq!(store.get(Location::Stable(3), path_baz), None);

        // Test that overwriting a key/value works
        store.set(path_baz.to_vec(), data2.to_vec());
        assert_eq!(store.get(Location::Pending, path_baz), Some(data2.to_vec()));
        store.set(path_baz.to_vec(), data1.to_vec());
        assert_eq!(store.get(Location::Pending, path_baz), Some(data1.to_vec()));
        store.set(path_baz.to_vec(), data2.to_vec());
        assert_eq!(store.get(Location::Pending, path_baz), Some(data2.to_vec()));

        store.grow(); // Advance the stable location to height 3.
        assert_eq!(
            store.get(Location::LatestStable, path_baz),
            Some(data2.to_vec())
        );
        assert_eq!(store.get(Location::Stable(0), path_baz), None);
        assert_eq!(store.get(Location::Stable(1), path_baz), None);
        assert_eq!(
            store.get(Location::Stable(2), path_baz),
            Some(data1.to_vec())
        );
        assert_eq!(
            store.get(Location::Stable(3), path_baz),
            Some(data2.to_vec())
        );
        for height in 4..10 {
            assert_eq!(store.get(Location::Stable(height), path_baz), None);
        }
    }
}
