use tendermint::abci::{Code, Log};
use tendermint::block;
use tendermint_rpc::endpoint::{
    abci_info::AbciInfo, abci_query::AbciQuery, abci_query::Request as AbciQueryRequest,
};

use crate::store::Storage;

pub fn get_info() -> AbciInfo {
    AbciInfo {
        data: "data_placeholder".to_string(),
        version: "v0.16.0".to_string(),
        app_version: 1,
        last_block_height: tendermint::block::Height::from(1 as u32),
        last_block_app_hash: vec![],
    }
}

pub fn handle_query<T: Storage>(query: AbciQueryRequest, store: &T) -> AbciQuery {
    println!(
        "ABCI data: {}",
        String::from_utf8(query.data.clone()).unwrap_or("".to_string())
    );
    let height = match query.height {
        None => 0,
        Some(h) => h.value(),
    };
    let item = store.get(height, &query.data);
    if let Some(item) = item {
        AbciQuery {
            code: Code::Ok,
            log: Log::from("exists"),
            info: "".to_string(),
            index: 0,
            key: query.data,
            value: item.to_vec(),
            proof: None,
            height: block::Height::from(1 as u32),
            codespace: "codespace".to_string(),
        }
    } else {
        AbciQuery {
            code: Code::Err(1),
            log: Log::from("data do not exist"),
            info: "Data not found".to_string(),
            index: 0,
            key: query.data,
            value: vec![],
            proof: None,
            height: block::Height::from(1 as u32),
            codespace: "codespace".to_string(),
        }
    }
}
