//! # Node
//!
//! A tendermock node, which encapsulate a storage and a chain.
//! This also exposes a thread safe version called `SharedNode` for use by the various RPC
//! interfaces.
//!
//! To integrate with IBC modules, the node implements the `ICS26Context` traits, which mainly deal
//! with storing and reading values from the store.
#![allow(unused_variables)] // lot of todos...
use crate::chain::Chain;
use crate::config::Config;
use crate::store::{InMemoryStore, Storage};
use ibc::ics02_client::client_def::{AnyClientState, AnyConsensusState};
use ibc::ics02_client::client_type::ClientType;
use ibc::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::ics02_client::error::{Error as ClientError, Kind as ClientErrorKind};
use ibc::ics03_connection::connection::ConnectionEnd;
use ibc::ics03_connection::context::{ConnectionKeeper, ConnectionReader};
use ibc::ics03_connection::error::Error as ConnectionError;
use ibc::ics03_connection::version::Version;
use ibc::ics04_channel::channel::ChannelEnd;
use ibc::ics04_channel::context::{ChannelKeeper, ChannelReader};
use ibc::ics04_channel::error::Error as ChannelError;
use ibc::ics05_port::capabilities::Capability;
use ibc::ics05_port::context::PortReader;
use ibc::ics23_commitment::commitment::CommitmentPrefix;
use ibc::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::ics26_routing::context::ICS26Context;
use ibc::Height;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use prost::Message;
use prost_types::Any;
use serde::{Deserialize, Serialize};
use serde_json;
use std::convert::TryFrom;
use std::str::FromStr;
use tendermint::chain;
use tendermint::net::Address;
use tendermint::node;
use tendermint_proto::Protobuf;
use tendermint_rpc::endpoint::status::SyncInfo;

// System constant
const COMMITMENT_PREFIX: &'static str = "store/ibc/key";

/// An `Arc<RwLock<>>` wrapper around a Node.
pub struct SharedNode<S: Storage> {
    node: std::sync::Arc<std::sync::RwLock<Node<S>>>,
}

impl<S: Storage> Clone for SharedNode<S> {
    fn clone(&self) -> Self {
        Self {
            node: std::sync::Arc::clone(&self.node),
        }
    }
}

impl<S: Storage> SharedNode<S> {
    /// Read lock acquisition.
    pub fn read(&self) -> std::sync::RwLockReadGuard<Node<S>> {
        self.node.read().unwrap()
    }

    /// Write lock acquisition.
    pub fn write(&self) -> std::sync::RwLockWriteGuard<Node<S>> {
        self.node.write().unwrap()
    }

    /// Grow the chain.
    #[allow(dead_code)]
    pub fn grow(&self) {
        self.node.write().unwrap().grow();
    }
}

/// A node contains a store, a chain and some meta-data.
pub struct Node<S: Storage> {
    chain: Chain<S>,
    chain_id: tendermint::chain::Id,
    host_client_id: String,
    info: node::Info,
    consensus_params: tendermint::consensus::Params,
}

impl Node<InMemoryStore> {
    pub fn new(config: &Config) -> Self {
        // TODO: allow to pass custimized values
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
            host_client_id: config.host_client.id.to_owned(),
            consensus_params: config.consensus_params.clone(),
            info,
        }
    }

    /// Return the node in an Arc<RwLock> wrapper, ready to be shared among threads.
    pub fn shared(self) -> SharedNode<InMemoryStore> {
        SharedNode {
            node: std::sync::Arc::new(std::sync::RwLock::new(self)),
        }
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
            latest_app_hash: tendermint::AppHash::try_from(vec![61 as u8; 32]).unwrap(),
            latest_block_height: (latest_block_height.revision_height as u32).into(),
            latest_block_time: block.signed_header.header.time,
            catching_up: false,
        }
    }
}

impl<S: Storage> ClientReader for SharedNode<S> {
    fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
        let path = format!("clients/{}/clientType", client_id.as_str());
        let node = self.read();
        let store = node.get_store();
        let client_type = store.get(0, path.as_bytes())?;
        let client_type = String::from_utf8(client_type.to_vec());
        match client_type {
            Err(_) => None,
            Ok(client_type) => ClientType::from_str(&client_type).ok(),
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        let path = format!("clients/{}/clientState", client_id.as_str());
        let node = self.read();
        let store = node.get_store();
        let value = store.get(0, path.as_bytes())?;
        let client_state = AnyClientState::decode(value.as_slice());
        client_state.ok()
    }

    fn consensus_state(&self, client_id: &ClientId, height: Height) -> Option<AnyConsensusState> {
        let path = format!(
            "clients/{}/consensusState/{}",
            client_id.as_str(),
            height.to_string()
        );
        let node = self.read();
        let store = node.get_store();
        let value = store.get(0, path.as_bytes())?;
        let consensus_state = AnyConsensusState::decode(value.as_slice());
        consensus_state.ok()
    }

    fn client_counter(&self) -> u64 {
        todo!();
    }
}

impl<S: Storage> ClientKeeper for SharedNode<S> {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ClientError> {
        let path = format!("clients/{}/clientType", client_id.as_str());
        let node = self.read();
        let store = node.get_store();
        store.set(
            path.into_bytes(),
            client_type.as_string().as_bytes().to_owned(),
        );
        Ok(())
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), ClientError> {
        let path = format!("clients/{}/clientState", client_id.as_str());
        // Store the client type
        self.store_client_type(client_id.clone(), client_state.client_type())?;
        // Store the client state
        let data: Any = client_state.into();
        let mut buffer = Vec::new();
        data.encode(&mut buffer)
            .map_err(|e| ClientErrorKind::InvalidRawClientState.context(e))?;
        let node = self.read();
        let store = node.get_store();
        store.set(path.into_bytes(), buffer);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: AnyConsensusState,
    ) -> Result<(), ClientError> {
        let path = format!(
            "clients/{}/consensusState/{}",
            client_id.to_string(),
            height.to_string()
        );
        let data: Any = consensus_state.into();
        let mut buffer = Vec::new();
        data.encode(&mut buffer)
            .map_err(|e| ClientErrorKind::InvalidRawConsensusState.context(e))?;
        let node = self.read();
        let store = node.get_store();
        store.set(path.into_bytes(), buffer);
        Ok(())
    }

    fn increase_client_counter(&mut self) {
        todo!();
    }
}

impl<S: Storage> ConnectionKeeper for SharedNode<S> {
    fn store_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        let mut buffer = Vec::new();
        let path = format!("connections/{}", connection_id.as_str());
        let raw: RawConnectionEnd = connection_end.to_owned().into();
        raw.encode(&mut buffer).unwrap();
        let node = self.write();
        node.get_store().set(path.into_bytes(), buffer);
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: &ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), ConnectionError> {
        let path = format!("clients/{}/connections", client_id.as_str());
        let node = self.read();
        let store = node.get_store();
        let connections = store.get(0, path.as_bytes()).unwrap_or(vec![]);
        let connections = String::from_utf8(connections).unwrap_or(String::from(""));
        let mut connections =
            serde_json::from_str::<Connections>(&connections).unwrap_or(Connections::new());
        connections
            .connections
            .push(connection_id.as_str().to_owned());
        store.set(path.into_bytes(), connection_id.as_bytes().to_owned());
        Ok(())
    }

    fn next_connection_id(&mut self) -> ConnectionId {
        todo!();
    }
}

impl<S: Storage> ConnectionReader for SharedNode<S> {
    fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
        let path = format!("connections/{}", connection_id.as_str());
        let node = self.read();
        let store = node.get_store();
        let value = store.get(0, path.as_bytes())?;
        let raw = RawConnectionEnd::decode(&*value).ok()?;
        ConnectionEnd::try_from(raw).ok()
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        <SharedNode<S> as ClientReader>::client_state(self, client_id)
    }

    fn host_current_height(&self) -> Height {
        self.read().chain.get_height()
    }

    fn host_chain_history_size(&self) -> usize {
        100
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::from(COMMITMENT_PREFIX.as_bytes().to_owned())
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Option<AnyConsensusState> {
        self.consensus_state(client_id, height)
    }

    fn host_consensus_state(&self, height: Height) -> Option<AnyConsensusState> {
        let host_id = ClientId::from_str(&self.read().host_client_id).unwrap();
        self.consensus_state(&host_id, height)
    }

    // TODO: what is the correct version format?
    fn get_compatible_versions(&self) -> Vec<Version> {
        vec![Version::default()]
    }

    // TODO: what if there is no compatible versions?
    fn pick_version(
        &self,
        _supported_versions: Vec<Version>,
        counterparty_candidate_versions: Vec<Version>,
    ) -> Option<Version> {
        match counterparty_candidate_versions.get(0) {
            Some(version) => Some(version.to_owned()),
            None => None,
        }
    }
}

impl<S: Storage> ChannelKeeper for SharedNode<S> {
    fn next_channel_id(&mut self) -> ChannelId {
        todo!();
    }

    fn store_connection_channels(
        &mut self,
        conn_id: &ConnectionId,
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_channel(
        &mut self,
        port_channel_id: &(PortId, ChannelId),
        channel_end: &ChannelEnd,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_send(
        &mut self,
        port_channel_id: &(PortId, ChannelId),
        seq: u64,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_recv(
        &mut self,
        port_channel_id: &(PortId, ChannelId),
        seq: u64,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_ack(
        &mut self,
        port_channel_id: &(PortId, ChannelId),
        seq: u64,
    ) -> Result<(), ChannelError> {
        todo!();
    }
}

impl<S: Storage> ChannelReader for SharedNode<S> {
    fn channel_end(&self, port_channel_id: &(PortId, ChannelId)) -> Option<ChannelEnd> {
        todo!();
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
        todo!();
    }

    fn connection_channels(&self, cid: &ConnectionId) -> Option<Vec<(PortId, ChannelId)>> {
        todo!();
    }

    fn channel_client_state(
        &self,
        port_channel_id: &(PortId, ChannelId),
    ) -> Option<AnyClientState> {
        todo!();
    }

    fn channel_client_consensus_state(
        &self,
        port_channel_id: &(PortId, ChannelId),
        height: Height,
    ) -> Option<AnyConsensusState> {
        todo!();
    }

    fn port_capability(&self, port_id: &PortId) -> Option<Capability> {
        todo!();
    }

    fn capability_authentification(&self, port_id: &PortId, cap: &Capability) -> bool {
        todo!();
    }
}

impl<S: Storage> PortReader for SharedNode<S> {
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<Capability> {
        todo!();
    }

    fn autenthenticate(&self, key: &Capability, port_id: &PortId) -> bool {
        todo!();
    }
}

impl<S: Storage> ICS26Context for SharedNode<S> {}

/// A type representing connections in memory
#[derive(Serialize, Deserialize)]
struct Connections {
    pub connections: Vec<String>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            connections: Vec::new(),
        }
    }
}
