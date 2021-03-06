//! # Chain
//!
//! This modules defines the tendermock chain. The chain is a vector of light blocks, which are
//! stripped down versions of 'real' tendermint blocks.
use crate::store::Storage;
use ibc::Height;
use std::sync::RwLock;
use tendermint::Block as TMBlock;
use tendermint_testgen::light_block::TMLightBlock;
use tendermint_testgen::{Generator, LightBlock};

pub struct Chain<S: Storage> {
    blocks: RwLock<Blocks>,
    store: S,
}

struct Blocks {
    /// The chain of validated blocks.
    chain: Vec<LightBlock>,
    /// The next block candidate, it will be considered valid once another block is added.
    pending_block: LightBlock,
}

impl<S: Storage> Chain<S> {
    pub fn new(store: S) -> Self {
        // To ease testing, the second block is always created at midnight, this fixes the second
        // header until next midnight.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let midnight = now - (now % 86_400);
        // Create genesis and pending block
        let genesis = LightBlock::new_default(1);
        let mut pending = genesis.next();
        let mut header_ref = pending.header.as_mut().unwrap();
        header_ref.time = Some(midnight);
        Chain {
            blocks: RwLock::new(Blocks {
                chain: vec![genesis],
                pending_block: pending,
            }),
            store,
        }
    }

    /// Returns a reference to the inner store.
    pub fn get_store(&self) -> &S {
        &self.store
    }

    /// Returns the height of the chain.
    ///
    /// The height is defined as the height of the latest validated blocks.
    pub fn get_height(&self) -> Height {
        let blocks = &self.blocks.read().unwrap().chain;
        let height = blocks
            .last()
            .expect("[Internal] Chain should be initialized with a block.")
            .height();
        Height::new(1, height)
    }

    /// Returns a Tendermint Light Block or None if no block exist at that height.
    pub fn get_block(&self, height: u64) -> Option<TMLightBlock> {
        let chain = &self.blocks.read().unwrap();
        let block = Chain::<S>::get_block_at_height(height, &chain.chain, &chain.pending_block)?;
        block.generate().ok()
    }

    /// Grow the chain by adding a new block.
    pub fn grow(&self) {
        // Date of the growth
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Create new block
        let mut blocks = self.blocks.write().unwrap();
        let mut next_block = blocks.pending_block.next();
        let mut header_ref = next_block.header.as_mut().unwrap();
        header_ref.time = Some(now);
        // Set next_block to pending and push the old pending to the chain
        std::mem::swap(&mut blocks.pending_block, &mut next_block);
        blocks.chain.push(next_block);
        drop(blocks); // Release lock
                      // Grow the store
        self.store.grow();
    }

    /// Returns the store at a given height, where 0 means latest.
    fn get_block_at_height<'a>(
        height: u64,
        blocks: &'a Vec<LightBlock>,
        pending: &'a LightBlock,
    ) -> Option<&'a LightBlock> {
        if height == 0 {
            blocks.last()
        } else if height == (blocks.len() + 1) as u64 {
            // Preview of the next (not yet validated) block
            Some(pending)
        } else {
            blocks.get((height - 1) as usize)
        }
    }
}

/// Build a Tendermint block from a Tendermint loght block.
pub fn to_full_block(light_block: TMLightBlock) -> TMBlock {
    let signed_header = light_block.signed_header;
    let block = tendermint::Block::new(
        signed_header.header,
        tendermint::abci::transaction::Data::default(), // TODO: should we include transaction data?
        tendermint::evidence::Data::new(vec![]),
        Some(signed_header.commit),
    )
    .unwrap();
    block
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::store::InMemoryStore;

    #[test]
    fn chain() {
        let chain = Chain::new(InMemoryStore::new());
        let height = chain.get_height();

        // Chain is expected to start at height 1 (same as Storage)
        assert_eq!(height.revision_height, 1);
        chain.grow();
        let height = chain.get_height();
        assert_eq!(height.revision_height, 2);
        let block = chain.get_block(3);
        assert!(block.is_some()); // The third block is not yet valid, but we can retrieve it anyway
        chain.grow();
        let height = chain.get_height();
        assert_eq!(height.revision_height, 3); // Now the third block is valid
    }
}
