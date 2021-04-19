//! # Storage initialization
//!
//! This modules initializes the storage, by inserting values into the node using the ICS26
//! interface.
//!
//! The initial values are taken fron the configuration (see `config` module).
use std::str::FromStr;

use ibc::{
    ics02_client::client_state::AnyClientState, ics02_client::client_type::ClientType,
    ics02_client::context::ClientKeeper, ics07_tendermint::client_state::AllowUpdate,
    ics07_tendermint::client_state::ClientState, ics24_host::identifier::ClientId, Height,
};
use tendermint::trust_threshold::TrustThresholdFraction;

use crate::config::{Client, Config};

/// Initialize the client keeper by registering all the client present in the configuration.
pub fn init<T: ClientKeeper>(keeper: &mut T, config: &Config) {
    for client in &config.clients {
        add_client(keeper, client, config);
    }
}

fn add_client<T: ClientKeeper>(keeper: &mut T, client: &Client, config: &Config) {
    let client_id = ClientId::from_str(&client.id)
        .unwrap_or_else(|_| panic!("Invalid client id: {}", &client.id));
    let client_state = new_client_state(config);
    keeper
        .store_client_state(client_id.clone(), client_state)
        .unwrap();
    keeper
        .store_client_type(client_id, ClientType::Tendermint)
        .unwrap();
}

fn new_client_state(config: &Config) -> AnyClientState {
    let duration = std::time::Duration::new(3600 * 24 * 30, 0);
    let height = Height::new(1, 1);
    let client_state = ClientState {
        chain_id: String::from(&config.chain_id).parse().unwrap(),
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
