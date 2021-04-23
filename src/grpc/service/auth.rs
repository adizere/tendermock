//! # gRPC Auth
//!
//! The auth tendermint gRPC API.

use ibc_proto::cosmos::auth::v1beta1;
use ibc_proto::cosmos::auth::v1beta1::query_server::{Query, QueryServer};
use prost::Message;
use prost_types::Any;
use tonic::{Request, Response, Status};

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
    async fn account(
        &self,
        request: Request<v1beta1::QueryAccountRequest>,
    ) -> Result<Response<v1beta1::QueryAccountResponse>, Status> {
        log!(Log::Grpc, "/auth/account {:?}", request);
        let base_account = v1beta1::BaseAccount {
            address: String::from("ACCOUNT_ADDRESS"),
            pub_key: None,
            account_number: 42,
            sequence: 42,
        };
        let mut buffer = Vec::new();
        base_account.encode(&mut buffer).unwrap();
        let response = v1beta1::QueryAccountResponse {
            account: Some(Any {
                type_url: String::from("TODO!"), // TODO: What is the `BaseAccount` url?
                value: buffer,
            }),
        };
        //let response = v1beta1::QueryAccountResponse { account: None };
        Ok(Response::new(response))
    }

    async fn params(
        &self,
        request: Request<v1beta1::QueryParamsRequest>,
    ) -> Result<Response<v1beta1::QueryParamsResponse>, Status> {
        log!(Log::Grpc, "/auth/params {:?}", request);
        unimplemented!()
    }
}
