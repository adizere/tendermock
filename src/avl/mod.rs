/// A simple in-memory AVL tree implementation.
use sha2::{Digest, Sha256};
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering};
use tendermint::hash::{Algorithm, Hash};

mod as_bytes;
mod proof;
mod tests;

pub use as_bytes::AsBytes;

const HASH_ALGO: Algorithm = Algorithm::Sha256;

type NodeRef<T, V> = Option<Box<AvlNode<T, V>>>;

#[derive(Eq, PartialEq, Debug)]
struct AvlNode<K: Ord, V> {
    key: K,
    value: V,
    hash: Hash,
    merkle_hash: Hash,
    height: u32,
    left: NodeRef<K, V>,
    right: NodeRef<K, V>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct AvlTree<K: Ord + AsBytes, V> {
    root: NodeRef<K, V>,
}

fn as_node_ref<K: Ord + AsBytes, V>(key: K, value: V) -> NodeRef<K, V>
where
    V: Borrow<[u8]>,
{
    Some(Box::new(AvlNode::new(key, value)))
}

impl<K: Ord + AsBytes, V> AvlNode<K, V>
where
    V: Borrow<[u8]>,
{
    fn new(key: K, value: V) -> Self {
        let mut sha = Sha256::new();
        sha.update(key.as_bytes());
        sha.update(value.borrow());
        let hash = sha.finalize();
        let merkle_hash = Hash::from_bytes(HASH_ALGO, &Sha256::digest(&hash)).unwrap();
        let hash = Hash::from_bytes(HASH_ALGO, &hash).unwrap();
        return AvlNode {
            key,
            value,
            hash,
            merkle_hash,
            height: 0,
            left: None,
            right: None,
        };
    }

    /// The left height, or None if there is no left child.
    fn left_height(&self) -> Option<u32> {
        if let Some(ref left) = self.left {
            Some(left.height)
        } else {
            None
        }
    }

    /// The right height, or None if there is no right child.
    fn right_height(&self) -> Option<u32> {
        if let Some(ref right) = self.right {
            Some(right.height)
        } else {
            None
        }
    }

    /// Update the hight of the node by looking at the hight of its two children.
    fn update_height(&mut self) {
        match &self.right {
            None => match &self.left {
                None => self.height = 0,
                Some(left) => self.height = left.height + 1,
            },
            Some(right) => match &self.left {
                None => self.height = right.height + 1,
                Some(left) => self.height = std::cmp::max(left.height, right.height) + 1,
            },
        }
    }

    /// Update the node's merkle hash by looking at the hashes of its two children.
    fn update_hashes(&mut self) {
        let mut sha = Sha256::new();
        if let Some(left) = &self.left {
            sha.update(&left.merkle_hash.as_bytes());
        }
        sha.update(&self.hash.as_bytes());
        if let Some(right) = &self.right {
            sha.update(right.merkle_hash.as_bytes())
        }
        self.merkle_hash = Hash::from_bytes(HASH_ALGO, sha.finalize().as_slice()).unwrap();
    }

    /// Update node meda data, such as its height and merkle hash, by lookind at its two
    /// children.
    fn update(&mut self) {
        self.update_hashes();
        self.update_height();
    }

    /// Returns the node's balance factor (left_height - right_height).
    fn balance_factor(&self) -> i32 {
        match (self.left_height(), self.right_height()) {
            (None, None) => 0,
            (None, Some(h)) => -(h as i32),
            (Some(h), None) => h as i32,
            (Some(h_l), Some(h_r)) => (h_l as i32) - (h_r as i32),
        }
    }
}

impl<K: Ord + AsBytes, V> AvlTree<K, V>
where
    V: Borrow<[u8]>,
{
    /// Return an ampty AVL tree.
    pub fn new() -> Self {
        AvlTree { root: None }
    }

    /// Return the hash of the merkle tree root, if it has at least one node.
    pub fn root_hash(&self) -> Option<&Hash> {
        if let Some(root) = &self.root {
            Some(&root.merkle_hash)
        } else {
            None
        }
    }

    /// Return the value corresponding to the key, if it exists.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut node_ref = &self.root;
        while let Some(ref node) = node_ref {
            match node.key.borrow().cmp(key) {
                Ordering::Greater => node_ref = &node.left,
                Ordering::Less => node_ref = &node.right,
                Ordering::Equal => return Some(&node.value),
            }
        }
        None
    }

    /// Insert a value into the AVL tree, this operation runs in amortized O(log(n)).
    pub fn insert(&mut self, key: K, value: V) {
        let node_ref = &mut self.root;
        AvlTree::insert_rec(node_ref, key, value);
    }

    /// Insert a value and return the node height.
    fn insert_rec(node_ref: &mut NodeRef<K, V>, key: K, value: V) {
        if let Some(node) = node_ref {
            match node.key.cmp(&key) {
                Ordering::Greater => AvlTree::insert_rec(&mut node.left, key, value),
                Ordering::Less => AvlTree::insert_rec(&mut node.right, key, value),
                Ordering::Equal => node.key = key,
            }
            node.update();
            AvlTree::balance_node(node_ref);
        } else {
            *node_ref = as_node_ref(key, value);
        }
    }

    /// Rebalance the AVL tree by performing rotations, if needed.
    fn balance_node(node_ref: &mut NodeRef<K, V>) {
        let node = node_ref
            .as_mut()
            .expect("[AVL]: Empty node in node balance");
        let balance_factor = node.balance_factor();
        if balance_factor >= 2 {
            let left = node
                .left
                .as_mut()
                .expect("[AVL]: Unexpected empty left node");
            if left.balance_factor() >= 1 {
                AvlTree::rotate_right(node_ref);
            } else {
                AvlTree::rotate_left(&mut node.left);
                AvlTree::rotate_right(node_ref);
            }
        } else if balance_factor <= -2 {
            let right = node
                .right
                .as_mut()
                .expect("[AVL]: Unexpected empty right node");
            if right.balance_factor() <= -1 {
                AvlTree::rotate_left(node_ref);
            } else {
                AvlTree::rotate_right(&mut node.right);
                AvlTree::rotate_left(node_ref);
            }
        }
    }

    /// Performs a right rotation.
    fn rotate_right(root: &mut NodeRef<K, V>) {
        let mut node = root.take().expect("[AVL]: Empty root in right rotation");
        let mut left = node.left.take().expect("[AVL]: Unexpected right rotation");
        let mut left_right = left.right.take();
        std::mem::swap(&mut node.left, &mut left_right);
        node.update();
        std::mem::swap(&mut left.right, &mut Some(node));
        left.update();
        std::mem::swap(root, &mut Some(left));
    }

    /// Perform a left rotation.
    fn rotate_left(root: &mut NodeRef<K, V>) {
        let mut node = root.take().expect("[AVL]: Empty root in left rotation");
        let mut right = node.right.take().expect("[AVL]: Unexpected left rotation");
        let mut right_left = right.left.take();
        std::mem::swap(&mut node.right, &mut right_left);
        node.update();
        std::mem::swap(&mut right.left, &mut Some(node));
        right.update();
        std::mem::swap(root, &mut Some(right))
    }
}
