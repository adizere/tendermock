//! Tests for a tendermock node.
#[cfg(test)]
mod tests {
    use crate::config;
    use crate::node::*;
    use ibc::ics02_client::client_def::{AnyClientState, AnyConsensusState};
    use ibc::ics02_client::client_type::ClientType;
    use ibc::ics02_client::context::{ClientKeeper, ClientReader};
    use ibc::ics07_tendermint::client_state::ClientState;
    use ibc::ics07_tendermint::consensus_state::ConsensusState;
    use ibc::ics23_commitment::commitment::CommitmentRoot;
    use ibc::ics24_host::identifier::ClientId;
    use ibc::Height;
    use std::convert::TryInto;
    use std::str::FromStr;
    use tendermint;
    use tendermint::trust_threshold::TrustThresholdFraction;

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
        node.store_consensus_state(client_id.clone(), height.clone(), consensus_state.clone())
            .unwrap();
        println!("{:?}", node.read().get_store());
        node.grow();
        println!("{:?}", node.read().get_store());
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
            chain_id: String::from("test_chain"),
            trusting_period: duration.clone(),
            trust_level: TrustThresholdFraction::new(1, 3).unwrap(),
            unbonding_period: duration.clone(),
            max_clock_drift: duration,
            frozen_height: height.clone(),
            latest_height: height,
            upgrade_path: vec![String::from("path")],
            allow_update_after_expiry: false,
            allow_update_after_misbehaviour: false,
        };
        AnyClientState::Tendermint(client_state)
    }
}
