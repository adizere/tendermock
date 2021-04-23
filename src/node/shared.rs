use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use ibc::{
    application::ics20_fungible_token_transfer::context::Ics20Context,
    ics02_client::client_consensus::AnyConsensusState,
    ics02_client::client_state::AnyClientState,
    ics02_client::client_type::ClientType,
    ics02_client::context::{ClientKeeper, ClientReader},
    ics02_client::error::{Error as ClientError, Kind as ClientErrorKind},
    ics03_connection::connection::ConnectionEnd,
    ics03_connection::context::{ConnectionKeeper, ConnectionReader},
    ics03_connection::error::Error as ConnectionError,
    ics03_connection::version::Version,
    ics04_channel::channel::ChannelEnd,
    ics04_channel::context::{ChannelKeeper, ChannelReader},
    ics04_channel::error::{Error as ChannelError, Error},
    ics04_channel::packet::{Receipt, Sequence},
    ics05_port::capabilities::Capability,
    ics05_port::context::PortReader,
    ics23_commitment::commitment::CommitmentPrefix,
    ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    ics26_routing::context::Ics26Context,
    Height,
};
use ibc_proto::ibc::core::client;
use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use prost::Message;
use prost_types::Any;
use tendermint_proto::Protobuf;

use crate::grpc::GrpcContext;
use crate::logger::Log;
use crate::node::bare::Node;
use crate::node::objects::{Connections, Counter};
use crate::store::{Location, PathValue, Storage};

// System constant
const COMMITMENT_PREFIX: &str = "store/ibc/key";

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
    pub fn new(bare: Node<S>) -> Self {
        Self {
            node: std::sync::Arc::new(std::sync::RwLock::new(bare)),
        }
    }

    /// Read lock acquisition.
    pub fn read(&self) -> std::sync::RwLockReadGuard<Node<S>> {
        self.node.read().unwrap()
    }

    /// Write lock acquisition.
    pub fn write(&self) -> std::sync::RwLockWriteGuard<Node<S>> {
        self.node.write().unwrap()
    }

    /// Grow the chain.
    pub fn grow(&self) {
        self.node.write().unwrap().grow();
    }
}

impl<S: Storage> ClientReader for SharedNode<S> {
    fn client_type(&self, client_id: &ClientId) -> Option<ClientType> {
        let path = format!("clients/{}/clientType", client_id.as_str());
        let node = self.read();
        let store = node.store();
        let client_type = store.get(Location::LatestStable, path.as_bytes())?;
        let client_type = String::from_utf8(client_type.to_vec());
        match client_type {
            Err(_) => None,
            Ok(client_type) => ClientType::from_str(&client_type).ok(),
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        let path = format!("clients/{}/clientState", client_id.as_str());
        let node = self.read();
        let store = node.store();
        let value = store.get(Location::LatestStable, path.as_bytes())?;
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
        let store = node.store();
        let value = store.get(Location::LatestStable, path.as_bytes())?;
        let consensus_state = AnyConsensusState::decode(value.as_slice());
        consensus_state.ok()
    }

    fn client_counter(&self) -> u64 {
        let path = "meta/clients/counter".to_string();
        let node = self.read();
        let store = node.store();

        match store.get(Location::LatestStable, path.as_bytes()) {
            None => 0,
            Some(counter_raw) => {
                let res: Counter = counter_raw.try_into().unwrap();
                res.into()
            }
        }
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
        let store = node.store();
        store.set(
            path.clone().into_bytes(),
            client_type.as_string().as_bytes().to_owned(),
        );
        log!(Log::Store, "Storing client type at {}", path);
        Ok(())
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), ClientError> {
        let path = format!("clients/{}/clientState", client_id.as_str());
        // Store the client state
        let data: Any = client_state.into();
        let mut buffer = Vec::new();
        data.encode(&mut buffer)
            .map_err(|e| ClientErrorKind::InvalidRawClientState.context(e))?;
        let node = self.read();
        let store = node.store();
        store.set(path.clone().into_bytes(), buffer);
        log!(Log::Store, "Storing client state at {}", path);
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
        let store = node.store();
        store.set(path.clone().into_bytes(), buffer);
        log!(Log::Store, "Storing client consensus state at {}", path);
        Ok(())
    }

    fn increase_client_counter(&mut self) {
        let cnt = Counter::from(self.client_counter() + 1);
        let path = "meta/clients/counter".to_string();
        let node = self.read();
        let store = node.store();
        log!(
            Log::Store,
            "Storing new client counter state at {}: {}",
            path,
            cnt
        );
        store.set(path.into_bytes(), cnt.into());
    }
}

impl<S: Storage> ConnectionKeeper for SharedNode<S> {
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        let mut buffer = Vec::new();
        let path = format!("connections/{}", connection_id.as_str());
        let raw: RawConnectionEnd = connection_end.to_owned().into();
        raw.encode(&mut buffer).unwrap();
        let node = self.write();
        node.store().set(path.into_bytes(), buffer);
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), ConnectionError> {
        let path = format!("clients/{}/connections", client_id.as_str());
        let node = self.read();
        let store = node.store();
        let connections = store
            .get(Location::LatestStable, path.as_bytes())
            .unwrap_or_default();
        let connections = String::from_utf8(connections).unwrap_or_else(|_| String::from(""));
        let mut connections = serde_json::from_str::<Connections>(&connections)
            .unwrap_or_else(|_| Connections::new());
        connections
            .connections
            .push(connection_id.as_str().to_owned());
        store.set(path.into_bytes(), connection_id.as_bytes().to_owned());
        Ok(())
    }

    fn increase_connection_counter(&mut self) {
        let cnt = Counter::from(self.connection_counter() + 1);
        let path = "meta/connections/counter".to_string();
        let node = self.read();
        let store = node.store();
        store.set(path.into_bytes(), cnt.into());
    }
}

impl<S: Storage> ConnectionReader for SharedNode<S> {
    fn connection_end(&self, connection_id: &ConnectionId) -> Option<ConnectionEnd> {
        let path = format!("connections/{}", connection_id.as_str());
        let node = self.read();
        let store = node.store();
        let value = store.get(Location::LatestStable, path.as_bytes())?;
        let raw = RawConnectionEnd::decode(&*value).ok()?;
        ConnectionEnd::try_from(raw).ok()
    }

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        <SharedNode<S> as ClientReader>::client_state(self, client_id)
    }

    fn host_current_height(&self) -> Height {
        self.read().chain().get_height()
    }

    fn host_oldest_height(&self) -> Height {
        todo!()
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
        unimplemented!() // TODO(Adi)
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

    fn connection_counter(&self) -> u64 {
        let path = "meta/connections/counter".to_string();
        let node = self.read();
        let store = node.store();

        match store.get(Location::LatestStable, path.as_bytes()) {
            None => 0,
            Some(counter_raw) => {
                let res: Counter = counter_raw.try_into().unwrap(); // convert from `[u8]`
                res.into() // convert to `u64`.
            }
        }
    }
}

impl<S: Storage> ChannelKeeper for SharedNode<S> {
    fn store_packet_commitment(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        timestamp: u64,
        heigh: Height,
        data: Vec<u8>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Error> {
        todo!()
    }

    fn store_packet_receipt(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        receipt: Receipt,
    ) -> Result<(), Error> {
        todo!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        ack: Vec<u8>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn delete_packet_acknowledgement(
        &mut self,
        key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Error> {
        todo!()
    }

    fn store_connection_channels(
        &mut self,
        conn_id: ConnectionId,
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_channel(
        &mut self,
        port_channel_id: (PortId, ChannelId),
        channel_end: &ChannelEnd,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_send(
        &mut self,
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_recv(
        &mut self,
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn store_next_sequence_ack(
        &mut self,
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), ChannelError> {
        todo!();
    }

    fn increase_channel_counter(&mut self) {
        todo!()
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

    fn client_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        todo!()
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Option<AnyConsensusState> {
        self.consensus_state(client_id, height)
    }

    fn authenticated_capability(&self, port_id: &PortId) -> Result<Capability, Error> {
        todo!()
    }

    fn get_next_sequence_send(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
        todo!()
    }

    fn get_next_sequence_recv(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
        todo!()
    }

    fn get_next_sequence_ack(&self, port_channel_id: &(PortId, ChannelId)) -> Option<Sequence> {
        todo!()
    }

    fn get_packet_commitment(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
        todo!()
    }

    fn get_packet_receipt(&self, key: &(PortId, ChannelId, Sequence)) -> Option<Receipt> {
        todo!()
    }

    fn get_packet_acknowledgement(&self, key: &(PortId, ChannelId, Sequence)) -> Option<String> {
        todo!()
    }

    fn hash(&self, value: String) -> String {
        todo!()
    }

    fn host_height(&self) -> Height {
        todo!()
    }

    fn host_timestamp(&self) -> u64 {
        todo!()
    }

    fn channel_counter(&self) -> u64 {
        todo!()
    }
}

impl<S: Storage> PortReader for SharedNode<S> {
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<Capability> {
        todo!();
    }

    fn authenticate(&self, key: &Capability, port_id: &PortId) -> bool {
        todo!()
    }
}

impl<S: Storage> Ics20Context for SharedNode<S> {}

impl<S: Storage> Ics26Context for SharedNode<S> {}

impl<S: Storage> GrpcContext for SharedNode<S> {
    fn consensus_states(&self, client_id: &ClientId) -> Vec<ConsensusStateWithHeight> {
        log!(Log::Store, "Fetching all consensus state of {}", client_id);
        let path = format!("clients/{}/consensusState/", client_id.as_str(),);
        let node = self.read();
        let store = node.store();
        let hits = store.get_by_prefix(Location::LatestStable, path.as_bytes());

        // Convert each pair into a `ConsensusStateWithHeight`
        let mut res = vec![];
        for PathValue { path: p, value: v } in hits {
            let path = String::from_utf8_lossy(p.as_slice()).into_owned();
            // The height is encoded in the path
            let height = path
                .split('/')
                .last()
                .map(|h| Height::from_str(h).ok())
                .flatten()
                .map(client::v1::Height::from);
            // Decode the `ConsensusState` from the value
            let consensus_state = AnyConsensusState::decode(v.as_slice()).ok().map(Any::from);
            res.push(ConsensusStateWithHeight {
                height,
                consensus_state,
            });
        }

        res
    }
}
