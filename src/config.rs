//! # Configuration
//!
//! This module handles the loading of tendermock configuration, which describes the content of the
//! genesis block.
//!
//! For now the configuration options are rather limited, but those can be extended by adding new
//! fields to the `Config` structs.
//!
//! The deserialization uses `serde` and `serde_json` to define and parse json config files.
use serde_json;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tendermint;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub chain_id: String,
    pub host_client: Client,
    pub clients: Vec<Client>,
    pub consensus_params: tendermint::consensus::Params,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub id: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let config = fs::read_to_string(&path)
            .expect(&format!("Could not find file: {}", path.as_ref().display()));
        serde_json::from_str(&config).expect("Could not parse config file")
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            chain_id: String::from("tendermock"),
            host_client: Client {
                id: String::from("donald_duck"),
            },
            clients: vec![],
            consensus_params: default_params(),
        }
    }
}

fn default_params() -> tendermint::consensus::Params {
    tendermint::consensus::Params {
        version: None,
        block: tendermint::block::Size {
            max_bytes: 22020096,
            max_gas: 1000,
        },
        evidence: tendermint::evidence::Params {
            max_bytes: 10000,
            max_age_duration: tendermint::evidence::Duration(std::time::Duration::from_secs(3600)),
            max_age_num_blocks: 10000,
        },
        validator: tendermint::consensus::params::ValidatorParams {
            pub_key_types: vec![tendermint::public_key::Algorithm::Ed25519],
        },
    }
}
