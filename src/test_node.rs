//! Tests for a Tendermock node.
//!

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::str::FromStr;

    use ibc::ics02_client::client_consensus::AnyConsensusState;
    use ibc::ics02_client::client_state::AnyClientState;
    use ibc::ics02_client::client_type::ClientType;
    use ibc::ics02_client::context::{ClientKeeper, ClientReader};
    use ibc::ics07_tendermint::client_state::{AllowUpdate, ClientState};
    use ibc::ics07_tendermint::consensus_state::ConsensusState;
    use ibc::ics23_commitment::commitment::CommitmentRoot;
    use ibc::ics24_host::identifier::{ChainId, ClientId};
    use ibc::Height;
    use tendermint::trust_threshold::TrustThresholdFraction;

    use crate::config;
    use crate::node::*;

    #[test]
    /// Test storage and retrieval of client and consensus states.
    fn client() {
        let node = Node::new(&config::Config::default());
        let mut node = node.shared();
        let height = Height::new(1, 1);
        let client_id = ClientId::from_str("UncleScrooge").unwrap();
        let client_state = dummy_client_state();
        let consensus_state = dummy_consensus_state();

        node.store_client_type(client_id.clone(), ClientType::Tendermint)
            .unwrap();
        node.store_client_state(client_id.clone(), client_state.clone())
            .unwrap();
        node.store_consensus_state(client_id.clone(), height, consensus_state.clone())
            .unwrap();
        println!("{:?}", node.read().store());
        node.grow();
        println!("{:?}", node.read().store());
        let client_type = node.client_type(&client_id).unwrap();
        assert_eq!(client_type, ClientType::Tendermint);
        let retrieved_client = node.client_state(&client_id).unwrap();
        assert_eq!(client_state, retrieved_client);
        let retrieved_consensus = node.consensus_state(&client_id, height).unwrap();
        assert_eq!(consensus_state, retrieved_consensus);
    }

    fn dummy_consensus_state() -> AnyConsensusState {
        let root = CommitmentRoot::from_bytes(b"root");
        let tm_consensus_state = ConsensusState {
            timestamp: std::time::SystemTime::now().into(),
            next_validators_hash: vec![14; tendermint::hash::SHA256_HASH_SIZE]
                .try_into()
                .unwrap(),
            root,
        };
        AnyConsensusState::Tendermint(tm_consensus_state)
    }

    fn dummy_client_state() -> AnyClientState {
        let duration = std::time::Duration::new(60, 0);
        let height = Height::new(1, 1);
        let client_state = ClientState {
            chain_id: ChainId::from_str("test_chain").unwrap(),
            trusting_period: duration,
            trust_level: TrustThresholdFraction::new(1, 3).unwrap(),
            unbonding_period: duration,
            max_clock_drift: duration,
            frozen_height: height,
            latest_height: height,
            upgrade_path: vec![String::from("path")],
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };
        AnyClientState::Tendermint(client_state)
    }
}
