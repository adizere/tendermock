//! # gRPC Staking
//!
//! The staking tendermint gRPC API.
use ibc_proto::cosmos::staking::v1beta1;
use ibc_proto::cosmos::staking::v1beta1::query_server::{Query, QueryServer};
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
    async fn validators(
        &self,
        request: Request<v1beta1::QueryValidatorsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorsResponse>, Status> {
        log!(Log::Grpc, "/staking/validators {:?}", request);
        unimplemented!();
    }

    async fn validator(
        &self,
        request: Request<v1beta1::QueryValidatorRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorResponse>, Status> {
        log!(Log::Grpc, "/staking/validator {:?}", request);
        unimplemented!();
    }

    async fn validator_delegations(
        &self,
        request: Request<v1beta1::QueryValidatorDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorDelegationsResponse>, Status> {
        log!(Log::Grpc, "/staking/validator_delegations {:?}", request);
        unimplemented!();
    }

    async fn validator_unbonding_delegations(
        &self,
        request: Request<v1beta1::QueryValidatorUnbondingDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorUnbondingDelegationsResponse>, Status> {
        log!(
            Log::Grpc,
            "/staking/validator_unbounding_delegations {:?}",
            request
        );
        unimplemented!();
    }

    async fn delegation(
        &self,
        request: Request<v1beta1::QueryDelegationRequest>,
    ) -> Result<Response<v1beta1::QueryDelegationResponse>, Status> {
        log!(Log::Grpc, "/staking/delegation {:?}", request);
        unimplemented!();
    }

    async fn unbonding_delegation(
        &self,
        request: Request<v1beta1::QueryUnbondingDelegationRequest>,
    ) -> Result<Response<v1beta1::QueryUnbondingDelegationResponse>, Status> {
        log!(Log::Grpc, "/staking/unbounding_delegation {:?}", request);
        unimplemented!();
    }

    async fn delegator_delegations(
        &self,
        request: Request<v1beta1::QueryDelegatorDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorDelegationsResponse>, Status> {
        log!(Log::Grpc, "/staking/delegator_delegations {:?}", request);
        unimplemented!();
    }

    async fn delegator_unbonding_delegations(
        &self,
        request: Request<v1beta1::QueryDelegatorUnbondingDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorUnbondingDelegationsResponse>, Status> {
        log!(
            Log::Grpc,
            "/staking/delegator_unbounding_delegations {:?}",
            request
        );
        unimplemented!();
    }

    async fn redelegations(
        &self,
        request: Request<v1beta1::QueryRedelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryRedelegationsResponse>, Status> {
        log!(Log::Grpc, "/staking/redelegations {:?}", request);
        unimplemented!();
    }

    async fn delegator_validators(
        &self,
        request: Request<v1beta1::QueryDelegatorValidatorsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorValidatorsResponse>, Status> {
        log!(Log::Grpc, "/staking/delegator_validators {:?}", request);
        unimplemented!();
    }

    async fn delegator_validator(
        &self,
        request: Request<v1beta1::QueryDelegatorValidatorRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorValidatorResponse>, Status> {
        log!(Log::Grpc, "/staking/delegator_validator {:?}", request);
        unimplemented!();
    }

    async fn historical_info(
        &self,
        request: Request<v1beta1::QueryHistoricalInfoRequest>,
    ) -> Result<Response<v1beta1::QueryHistoricalInfoResponse>, Status> {
        log!(Log::Grpc, "/staking/historical_info {:?}", request);
        unimplemented!();
    }

    async fn pool(
        &self,
        request: Request<v1beta1::QueryPoolRequest>,
    ) -> Result<Response<v1beta1::QueryPoolResponse>, Status> {
        log!(Log::Grpc, "/staking/pool   {:?}", request);
        unimplemented!();
    }

    async fn params(
        &self,
        request: Request<v1beta1::QueryParamsRequest>,
    ) -> Result<Response<v1beta1::QueryParamsResponse>, Status> {
        log!(Log::Grpc, "/staking/params {:?}", request);
        let response = v1beta1::QueryParamsResponse {
            params: Some(v1beta1::Params {
                bond_denom: "bond_denom".to_owned(),
                historical_entries: 0,
                max_entries: 3,
                max_validators: 3,
                unbonding_time: Some(std::time::Duration::new(3600 * 24 * 30, 0).into()),
            }),
        };
        Ok(Response::new(response))
    }
}
