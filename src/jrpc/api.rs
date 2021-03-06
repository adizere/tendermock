//! The Tendermock JsonRPC HTTP API.
use ibc::ics26_routing::handler::deliver;
use ibc_proto::cosmos::tx::v1beta1::{TxBody, TxRaw};
use prost::Message;
use tendermint::abci::{transaction::Hash, Code, Info};
use tendermint::abci::responses::Codespace;
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
const HASH_LENGHT: usize = 32; // tendermint::abci::transaction::hash::LENGHT is not exposed...

/// A structure to build the JsonRPC HTTP API, see the `new` method.
pub struct Jrpc<S: store::Storage>
where
    node::SharedNode<S>: Clone,
{
    pub verbose: bool,
    pub node: node::SharedNode<S>,
}

// See this [issue](https://github.com/rust-lang/rust/issues/41481)
impl<S: store::Storage> Clone for Jrpc<S> {
    fn clone(&self) -> Self {
        Self {
            verbose: self.verbose,
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
    pub fn new(
        verbose: bool,
        node: node::SharedNode<S>,
    ) -> impl warp::Filter<Extract = (String,), Error = warp::Rejection> + Clone {
        let state = Self { verbose, node };
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
        if state.verbose {
            log!(Log::JRPC, "/block      {:?}", req);
        }
        let height = match req.height {
            None => 0,
            Some(height) => height.into(),
        };
        let node = state.node.read();
        let block = node
            .get_chain()
            .get_block(height)
            .ok_or_else(|| JrpcError::InvalidRequest)?;
        let tm_block = to_full_block(block);
        let hash = tm_block.header.hash();
        Ok(BlockResponse {
            block_id: tendermint::block::Id {
                part_set_header: tendermint::block::parts::Header::new(1, hash.clone()).unwrap(),
                hash,
            },
            block: tm_block,
        })
    }

    /// JsonRPC /commit endpoint.
    fn commit(req: CommitRequest, state: Self) -> JrpcResult<CommitResponse> {
        if state.verbose {
            log!(Log::JRPC, "/commit     {:?}", req);
        }
        let height = match req.height {
            None => 0,
            Some(height) => height.into(),
        };
        let node = state.node.read();
        let block = node
            .get_chain()
            .get_block(height)
            .ok_or_else(|| JrpcError::InvalidRequest)?;
        let signed_header = block.signed_header;
        Ok(CommitResponse {
            signed_header,
            canonical: false,
        })
    }

    /// JsonRPC /genesis endpoint.
    fn genesis(req: GenesisRequest, state: Self) -> JrpcResult<GenesisResponse> {
        if state.verbose {
            log!(Log::JRPC, "/genesis    {:?}", req);
        }
        let node = state.node.read();
        let genesis_block = node.get_chain().get_block(1).unwrap();
        let genesis = tendermint::Genesis {
            genesis_time: genesis_block.signed_header.header.time,
            chain_id: node.get_chain_id().clone(),
            consensus_params: node.get_consensus_params().clone(),
            validators: genesis_block.validators.validators().clone(),
            app_hash: vec![100, 200],
            app_state: serde_json::Value::Null,
        };
        Ok(GenesisResponse { genesis })
    }

    /// JsonRPC /validators endpoint.
    fn validators(req: ValidatorsRequest, state: Self) -> JrpcResult<ValidatorResponse> {
        if state.verbose {
            log!(Log::JRPC, "/validators {:?}", req);
        }
        let node = state.node.read();
        let block = node
            .get_chain()
            .get_block(req.height.into())
            .ok_or_else(|| JrpcError::InvalidRequest)?;
        let validators = block.validators.validators().clone();
        Ok(ValidatorResponse {
            block_height: block.signed_header.header.height,
            validators,
        })
    }

    /// JsonRPC /status endpoint.
    fn status(req: StatusRequest, state: Self) -> JrpcResult<StatusResponse> {
        if state.verbose {
            log!(Log::JRPC, "/status     {:?}", req);
        }
        let node = state.node.read();
        let node_info = node.get_info().clone();
        let sync_info = node.get_sync_info();
        let validator_info = tendermint::validator::Info {
            address: tendermint::account::Id::new([41; 20]),
            pub_key: tendermint::public_key::PublicKey::from_raw_ed25519(
                &hex::decode(PUBLICK_KEY).unwrap(),
            )
            .unwrap(),
            voting_power: (1 as u32).into(),
            proposer_priority: 1.into(),
        };
        Ok(StatusResponse {
            node_info,
            sync_info,
            validator_info,
        })
    }

    /// JsonRPC /abci_info endpoint.
    fn abci_info(req: AbciInfoRequest, state: Self) -> JrpcResult<AbciInfoResponse> {
        if state.verbose {
            log!(Log::JRPC, "/abci_info  {:?}", req);
        }
        let node = state.node.read();
        Ok(AbciInfoResponse {
            response: abci::get_info(&node),
        })
    }

    /// JsonRPC /abci_query endpoint.
    fn abci_query(req: AbciQueryRequest, state: Self) -> JrpcResult<AbciQueryResponse> {
        if state.verbose {
            log!(
                Log::JRPC,
                "/abci_query {{ path: {:?}, data: {} }}",
                req.path,
                String::from_utf8(req.data.clone()).unwrap_or("".to_string())
            );
        }
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
        if state.verbose {
            log!(Log::JRPC, "/broadcast_tx_commit {{ tx: {} bytes }}", req.tx.as_bytes().len());
        }
        // Grow chain
        let node = state.node.write();
        node.get_chain().grow();
        let block = node.get_chain().get_block(0).unwrap();
        drop(node); // Release write lock

        // Build transactions
        let data: Vec<u8> = req.tx.into();
        let tx_raw = TxRaw::decode(&*data).map_err(|_| JrpcError::InvalidRequest)?;
        let tx_body = TxBody::decode(&*tx_raw.body_bytes).map_err(|_| JrpcError::InvalidRequest)?;
        deliver(&mut state.node, tx_body.messages).map_err(|e| {
            log!(Log::JRPC, "deliver error: '{}'", e);
            JrpcError::ServerError
        })?;

        // Build a response, for now with arbitrary values.
        let tx_result = TxResult {
            code: Code::Ok,
            data: None,
            log: "Success".into(),
            codespace: Codespace::default(),
            gas_used: 10.into(),
            gas_wanted: 10.into(),
            info: Info::default(),
            events: vec![],
        };
        Ok(BroadcastTxCommitResponse {
            check_tx: tx_result.clone(),
            deliver_tx: tx_result,
            hash: Hash::new([61; HASH_LENGHT]),
            height: block.signed_header.header.height,
        })
    }
}
