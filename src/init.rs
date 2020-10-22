use ibc::ics02_client::client_def::AnyClientState;
use ibc::ics02_client::context::ClientKeeper;
use ibc::ics07_tendermint::client_state::ClientState;
use ibc::ics24_host::identifier::ClientId;
use ibc::Height;
use std::str::FromStr;

const CLIENT_ID: &'static str = "glomgold_chain";

pub fn init<T: ClientKeeper>(keeper: &mut T) {
    let client_id = ClientId::from_str(CLIENT_ID).unwrap();
    let client_state = new_client_state(CLIENT_ID);
    keeper.store_client_state(client_id, client_state).unwrap();
}

fn new_client_state(chain_id: &str) -> AnyClientState {
    let duration = std::time::Duration::new(60, 0);
    let height = Height::new(1, 1);
    let client_state = ClientState {
        chain_id: String::from(chain_id),
        trusting_period: duration.clone(),
        unbonding_period: duration.clone(),
        max_clock_drift: duration,
        frozen_height: height.clone(),
        latest_height: height,
        upgrade_path: String::from("path"),
        allow_update_after_expiry: false,
        allow_update_after_misbehaviour: false,
    };
    AnyClientState::Tendermint(client_state)
}