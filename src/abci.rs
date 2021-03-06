//! # ABCI interface
//!
//! This modules handles operations of the ABCI interface, which mostly interact with the on-chain
//! store.
use tendermint::abci::{Code, Log as AbciLog};
use tendermint::block;
use tendermint_rpc::endpoint::{
    abci_info::AbciInfo, abci_query::AbciQuery, abci_query::Request as AbciQueryRequest,
};

use crate::node::Node;
use crate::store::Storage;

/// Return information about the ABCI API.
pub fn get_info<S: Storage>(node: &Node<S>) -> AbciInfo {
    let chain = node.get_chain();
    // TODO: inject valid informations
    AbciInfo {
        data: "data_placeholder".to_string(),
        version: "v0.17.0".to_string(),
        app_version: 1,
        last_block_height: tendermint::block::Height::from(
            chain.get_height().revision_height as u32,
        ),
        last_block_app_hash: vec![],
    }
}

/// Handle an ABCI query.
pub fn handle_query<S: Storage>(query: AbciQueryRequest, node: &Node<S>) -> AbciQuery {
    let height = match query.height {
        None => 0,
        Some(h) => h.value(),
    };
    let store = node.get_store();
    let item = store.get(height, &query.data);
    if let Some(item) = item {
        AbciQuery {
            code: Code::Ok,
            log: AbciLog::from("exists"),
            info: "".to_string(),
            index: 0,
            key: query.data,
            value: item.to_vec(),
            proof: None,
            height: block::Height::from(height as u32),
            codespace: "".to_string(),
        }
    } else {
        AbciQuery {
            code: Code::Err(1),
            log: AbciLog::from("data do not exist"),
            info: "Data not found".to_string(),
            index: 0,
            key: query.data,
            value: vec![],
            proof: None,
            height: block::Height::from(height as u32),
            codespace: "".to_string(),
        }
    }
}
