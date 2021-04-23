//! # gRPC Client
//!
//! The gRPC API for the IBC Core Client functionality.

use std::str::FromStr;

use ibc::ics24_host::identifier::ClientId;
use ibc_proto::ibc::core::client::v1::query_server::{Query, QueryServer};
use ibc_proto::ibc::core::client::v1::{
    QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
    QueryConsensusStatesResponse,
};
use tonic::{Request, Response, Status};

use crate::grpc::GrpcContext;
use crate::logger::Log;
use crate::node;
use crate::store::Storage;

pub fn get_service<S: 'static + Storage + Sync + Send>(
    node: node::SharedNode<S>,
) -> QueryServer<QueryService<S>> {
    let query_service = QueryService::new(node);
    QueryServer::new(query_service)
}

/// A struct handling the `Query` service.
#[derive(Clone)]
pub struct QueryService<S: Storage> {
    #[allow(dead_code)]
    node: node::SharedNode<S>,
}

impl<S: Storage> QueryService<S> {
    fn new(node: node::SharedNode<S>) -> Self {
        QueryService { node }
    }
}

#[tonic::async_trait]
impl<S: 'static + Storage + Sync + Send> Query for QueryService<S> {
    async fn client_state(
        &self,
        _request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        todo!()
    }

    async fn client_states(
        &self,
        _request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        todo!()
    }

    async fn consensus_state(
        &self,
        _request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        todo!()
    }

    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        let client_id_raw = request.into_inner().client_id;
        log!(Log::Grpc, "/client/consensus_states {}", client_id_raw);

        let client_id_opt = ClientId::from_str(client_id_raw.as_str());
        match client_id_opt {
            Ok(client_id) => {
                let cs = self.node.consensus_states(&client_id);
                log!(Log::Grpc, "Consensus states found: {}", cs.len());

                let response = QueryConsensusStatesResponse {
                    consensus_states: cs,
                    pagination: None,
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                let message = format!("Malformed client id '{}'. Error: {}", client_id_raw, e);
                log!(Log::Grpc, message);
                Err(Status::invalid_argument(message))
            }
        }
    }

    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        todo!()
    }
}
