//! The Tendermock JsonRPC HTTP API.

use ibc::events::IbcEvent;
use ibc::ics26_routing::handler::deliver;
use ibc_proto::cosmos::tx::v1beta1::{TxBody, TxRaw};
use prost::Message;
use tendermint::abci::responses::Codespace;
use tendermint::abci::tag::Tag;
use tendermint::abci::{transaction::Hash, Code, Event, Info};
use tendermint_rpc::endpoint::{
    abci_info::Request as AbciInfoRequest, abci_info::Response as AbciInfoResponse,
    abci_query::Request as AbciQueryRequest, abci_query::Response as AbciQueryResponse,
    block::Request as BlockRequest, block::Response as BlockResponse,
    broadcast::tx_commit::Request as BroadcastTxCommitRequest,
    broadcast::tx_commit::Response as BroadcastTxCommitResponse, broadcast::tx_commit::TxResult,
    commit::Request as CommitRequest, commit::Response as CommitResponse,
    genesis::Request as GenesisRequest, genesis::Response as GenesisResponse,
    status::Request as StatusRequest, status::Response as StatusResponse,
    validators::Request as ValidatorsRequest, validators::Response as ValidatorResponse,
};

use crate::abci;
use crate::chain::to_full_block;
use crate::logger::Log;
use crate::node;
use crate::store;

use super::utils::{JrpcError, JrpcFilter, JrpcResult};

const PUBLICK_KEY: &str = "4A25C6640A1F72B9C975338294EF51B6D1C33158BB6ECBA69FBC3FB5A33C9DCE";
const HASH_LENGTH: usize = 32; // tendermint::abci::transaction::hash::LENGTH is not exposed...

/// A structure to build the JsonRPC HTTP API, see the `new` method.
pub struct Jrpc<S: store::Storage>
where
    node::SharedNode<S>: Clone,
{
    pub node: node::SharedNode<S>,
}

// See this [issue](https://github.com/rust-lang/rust/issues/41481)
impl<S: store::Storage> Clone for Jrpc<S> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<S> Jrpc<S>
where
    S: 'static + store::Storage,
    node::SharedNode<S>: Sync + Send + Clone,
{
    /// Creates a new `warp` filter that mimics Tendermint's JsonRPC HTTP API.
    pub fn new_mimic(
        node: node::SharedNode<S>,
    ) -> impl warp::Filter<Extract = (String,), Error = warp::Rejection> + Clone {
        let state = Self { node };
        JrpcFilter::new(state)
            .add("block", Self::block)
            .add("commit", Self::commit)
            .add("genesis", Self::genesis)
            .add("validators", Self::validators)
            .add("status", Self::status)
            .add("abci_info", Self::abci_info)
            .add("abci_query", Self::abci_query)
            .add("broadcast_tx_commit", Self::broadcast_tx_commit)
            .build()
    }

    /// JsonRPC /block endpoint.
    fn block(req: BlockRequest, state: Self) -> JrpcResult<BlockResponse> {
        log!(Log::Jrpc, "/block      {:?}", req);
        let height = match req.height {
            None => 0,
            Some(height) => height.into(),
        };
        let node = state.node.read();
        let block = node
            .chain()
            .get_block(height)
            .ok_or(JrpcError::InvalidRequest)?;
        let tm_block = to_full_block(block);
        let hash = tm_block.header.hash();
        Ok(BlockResponse {
            block_id: tendermint::block::Id {
                part_set_header: tendermint::block::parts::Header::new(1, hash).unwrap(),
                hash,
            },
            block: tm_block,
        })
    }

    /// JsonRPC /commit endpoint.
    fn commit(req: CommitRequest, state: Self) -> JrpcResult<CommitResponse> {
        log!(Log::Jrpc, "/commit     {:?}", req);
        let height = match req.height {
            None => 0,
            Some(height) => height.into(),
        };
        let node = state.node.read();
        let block = node
            .chain()
            .get_block(height)
            .ok_or(JrpcError::InvalidRequest)?;
        let signed_header = block.signed_header;
        Ok(CommitResponse {
            signed_header,
            canonical: false,
        })
    }

    /// JsonRPC /genesis endpoint.
    #[allow(clippy::unnecessary_wraps)]
    fn genesis(req: GenesisRequest, state: Self) -> JrpcResult<GenesisResponse> {
        log!(Log::Jrpc, "/genesis    {:?}", req);
        let node = state.node.read();
        let genesis_block = node.chain().get_block(1).unwrap();
        let genesis = tendermint::Genesis {
            genesis_time: genesis_block.signed_header.header.time,
            chain_id: node.chain_id().clone(),
            consensus_params: node.consensus_params().clone(),
            validators: genesis_block.validators.validators().clone(),
            app_hash: vec![100, 200],
            app_state: serde_json::Value::Null,
        };
        Ok(GenesisResponse { genesis })
    }

    /// JsonRPC /validators endpoint.
    fn validators(req: ValidatorsRequest, state: Self) -> JrpcResult<ValidatorResponse> {
        log!(Log::Jrpc, "/validators {:?}", req);
        let node = state.node.read();
        let block = node
            .chain()
            .get_block(req.height.unwrap().into())
            .ok_or(JrpcError::InvalidRequest)?;
        let validators = block.validators.validators().clone();
        let total = validators.len() as i32;

        Ok(ValidatorResponse::new(
            block.signed_header.header.height,
            validators,
            total,
        ))
    }

    /// JsonRPC /status endpoint.
    #[allow(clippy::unnecessary_wraps)]
    fn status(req: StatusRequest, state: Self) -> JrpcResult<StatusResponse> {
        log!(Log::Jrpc, "/status     {:?}", req);
        let node = state.node.read();
        let node_info = node.info().clone();
        let sync_info = node.get_sync_info();
        let validator_info = tendermint::validator::Info {
            address: tendermint::account::Id::new([41; 20]),
            pub_key: tendermint::public_key::PublicKey::from_raw_ed25519(
                &hex::decode(PUBLICK_KEY).unwrap(),
            )
            .unwrap(),
            voting_power: (1_u32).into(),
            proposer_priority: 1.into(),
        };
        Ok(StatusResponse {
            node_info,
            sync_info,
            validator_info,
        })
    }

    /// JsonRPC /abci_info endpoint.
    #[allow(clippy::unnecessary_wraps)]
    fn abci_info(req: AbciInfoRequest, state: Self) -> JrpcResult<AbciInfoResponse> {
        log!(Log::Jrpc, "/abci_info  {:?}", req);
        let node = state.node.read();
        Ok(AbciInfoResponse {
            response: abci::get_info(&node),
        })
    }

    /// JsonRPC /abci_query endpoint.
    #[allow(clippy::unnecessary_wraps)]
    fn abci_query(req: AbciQueryRequest, state: Self) -> JrpcResult<AbciQueryResponse> {
        log!(
            Log::Jrpc,
            "/abci_query {{ path: {:?}, data: {} }}",
            req.path,
            String::from_utf8(req.data.clone()).unwrap_or_else(|_| "".to_string())
        );
        let node = state.node.read();
        Ok(AbciQueryResponse {
            response: abci::handle_query(req, &node),
        })
    }

    /// JsonRPC /broadcast_tx_commit endpoint.
    fn broadcast_tx_commit(
        req: BroadcastTxCommitRequest,
        mut state: Self,
    ) -> JrpcResult<BroadcastTxCommitResponse> {
        log!(
            Log::Jrpc,
            "/broadcast_tx_commit {{ tx: {} bytes }}",
            req.tx.as_bytes().len()
        );
        // Grow chain
        let node = state.node.write();
        node.chain().grow();
        let block = node.chain().get_block(0).unwrap();
        drop(node); // Release write lock

        // Decode the txs
        let data: Vec<u8> = req.tx.into();
        let tx_raw = TxRaw::decode(&*data).map_err(|_| JrpcError::InvalidRequest)?;
        let tx_body = TxBody::decode(&*tx_raw.body_bytes).map_err(|_| JrpcError::InvalidRequest)?;

        // Deliver the txs
        let ibc_events = deliver(&mut state.node, tx_body.messages).map_err(|e| {
            log!(Log::Jrpc, "deliver error: '{}'", e);
            JrpcError::ServerError
        })?;

        // Transform `IBCEvent` into `abci::Event`
        // TODO: This is a workaround for https://github.com/informalsystems/ibc-rs/issues/838
        let events = ibc_events
            .iter()
            .filter_map(|e| match e {
                IbcEvent::CreateClient(c) => Some(Event {
                    type_str: "create_client".to_string(),
                    attributes: vec![Tag {
                        key: "client_id".parse().unwrap(),
                        value: c.client_id().to_string().parse().unwrap(),
                    }],
                }),
                _ => None,
            })
            .collect();

        // Build a response, for now with arbitrary values.
        let tx_result = TxResult {
            code: Code::Ok,
            data: None,
            log: "Success".into(),
            codespace: Codespace::default(),
            gas_used: 10.into(),
            gas_wanted: 10.into(),
            info: Info::default(),
            events,
        };
        Ok(BroadcastTxCommitResponse {
            check_tx: tx_result.clone(),
            deliver_tx: tx_result,
            hash: Hash::new([61; HASH_LENGTH]),
            height: block.signed_header.header.height,
        })
    }
}
