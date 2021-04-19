use std::convert::TryFrom;
use std::str::FromStr;

use tendermint::net::Address;
use tendermint::{chain, node};
use tendermint_rpc::endpoint::status::SyncInfo;

use crate::chain::Chain;
use crate::config::Config;
use crate::node::shared::SharedNode;
use crate::store::{InMemoryStore, Storage};

/// A bare node contains:
///     - a chain, plus its associated store,
///     - and some meta-data.
pub struct Node<S: Storage> {
    chain: Chain<S>,
    chain_id: tendermint::chain::Id,
    info: node::Info,
    consensus_params: tendermint::consensus::Params,
}

impl Node<InMemoryStore> {
    pub fn new(config: &Config) -> Self {
        // TODO: allow to pass customized values
        let info = node::Info {
            // Node id
            id: node::Id::new([61; 20]),
            listen_addr: node::info::ListenAddress::new(String::from("localhost:26657")),
            network: chain::Id::from_str(&config.chain_id).unwrap(),
            protocol_version: node::info::ProtocolVersionInfo {
                p2p: 0,
                block: 0,
                app: 0,
            },
            version: serde_json::from_value(serde_json::Value::String("v0.1.0".to_string()))
                .unwrap(),
            channels: serde_json::from_value(serde_json::Value::String("channels".to_string()))
                .unwrap(),
            moniker: tendermint::Moniker::from_str("moniker").unwrap(),
            other: node::info::OtherInfo {
                tx_index: node::info::TxIndexStatus::Off,
                rpc_address: Address::from_str("tcp://127.0.0.1:26657").unwrap(),
            },
        };
        Node {
            chain: Chain::new(InMemoryStore::new()),
            chain_id: tendermint::chain::Id::try_from(config.chain_id.to_owned()).unwrap(),
            consensus_params: config.consensus_params.clone(),
            info,
        }
    }

    /// Return the node in an Arc<RwLock> wrapper, ready to be shared among threads.
    pub fn shared(self) -> SharedNode<InMemoryStore> {
        SharedNode::new(self)
    }
}

impl<S: Storage> Node<S> {
    pub fn get_store(&self) -> &S {
        &self.chain.get_store()
    }

    pub fn get_chain(&self) -> &Chain<S> {
        &self.chain
    }

    pub fn get_info(&self) -> &node::Info {
        &self.info
    }

    pub fn get_chain_id(&self) -> &chain::Id {
        &self.chain_id
    }

    pub fn get_consensus_params(&self) -> &tendermint::consensus::Params {
        &self.consensus_params
    }

    pub fn grow(&self) {
        self.chain.grow();
    }

    /// Get sync infos. For now only the field `latest_block_height` contains a valid value.
    pub fn get_sync_info(&self) -> SyncInfo {
        let latest_block_height = self.chain.get_height();
        let block = self
            .chain
            .get_block(0)
            .expect("The chain should always contain a block");
        let hash = block.signed_header.header.hash();
        SyncInfo {
            latest_block_hash: hash,
            latest_app_hash: tendermint::AppHash::try_from(vec![61_u8; 32]).unwrap(),
            latest_block_height: (latest_block_height.revision_height as u32).into(),
            latest_block_time: block.signed_header.header.time,
            catching_up: false,
        }
    }
}
