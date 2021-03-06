//! # gRPC Staking
//!
//! The staking tendermint gRPC API.
use crate::logger::Log;
use crate::node;
use crate::store::Storage;
use ibc_proto::cosmos::staking::v1beta1;
use tonic::{Status, Response, Request};
use ibc_proto::cosmos::staking::v1beta1::query_server::{Query, QueryServer};

pub fn get_service<S: 'static + Storage + Sync + Send>(
    node: node::SharedNode<S>,
    verbose: bool,
) -> QueryServer<QueryService<S>> {
    let query_service = QueryService::new(node, verbose);
    QueryServer::new(query_service)
}

/// A struct handling the `Query` service.
#[derive(Clone)]
pub struct QueryService<S: Storage> {
    #[allow(dead_code)]
    node: node::SharedNode<S>,
    verbose: bool,
}

impl<S: Storage> QueryService<S> {
    fn new(node: node::SharedNode<S>, verbose: bool) -> Self {
        QueryService { node, verbose }
    }
}

#[tonic::async_trait]
impl<S: 'static + Storage + Sync + Send> Query for QueryService<S> {
    async fn validator(
        &self,
        request: Request<v1beta1::QueryValidatorRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/validator {:?}", request);
        }
        unimplemented!();
    }

    async fn validators(
        &self,
        request: Request<v1beta1::QueryValidatorsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/validators {:?}", request);
        }
        unimplemented!();
    }

    async fn validator_delegations(
        &self,
        request: Request<v1beta1::QueryValidatorDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorDelegationsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/validator_delegations {:?}", request);
        }
        unimplemented!();
    }

    async fn validator_unbonding_delegations(
        &self,
        request: Request<v1beta1::QueryValidatorUnbondingDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryValidatorUnbondingDelegationsResponse>, Status>
    {
        if self.verbose {
            log!(
                Log::GRPC,
                "/staking/validator_unbounding_delegations {:?}",
                request
            );
        }
        unimplemented!();
    }

    async fn delegation(
        &self,
        request: Request<v1beta1::QueryDelegationRequest>,
    ) -> Result<Response<v1beta1::QueryDelegationResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/delegation {:?}", request);
        }
        unimplemented!();
    }

    async fn unbonding_delegation(
        &self,
        request: Request<v1beta1::QueryUnbondingDelegationRequest>,
    ) -> Result<Response<v1beta1::QueryUnbondingDelegationResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/unbounding_delegation {:?}", request);
        }
        unimplemented!();
    }

    async fn delegator_validator(
        &self,
        request: Request<v1beta1::QueryDelegatorValidatorRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorValidatorResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/delegator_validator {:?}", request);
        }
        unimplemented!();
    }

    async fn delegator_delegations(
        &self,
        request: Request<v1beta1::QueryDelegatorDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorDelegationsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/delegator_delegations {:?}", request);
        }
        unimplemented!();
    }

    async fn delegator_unbonding_delegations(
        &self,
        request: Request<v1beta1::QueryDelegatorUnbondingDelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorUnbondingDelegationsResponse>, Status>
    {
        if self.verbose {
            log!(
                Log::GRPC,
                "/staking/delegator_unbounding_delegations {:?}",
                request
            );
        }
        unimplemented!();
    }

    async fn redelegations(
        &self,
        request: Request<v1beta1::QueryRedelegationsRequest>,
    ) -> Result<Response<v1beta1::QueryRedelegationsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/redelegations {:?}", request);
        }
        unimplemented!();
    }

    async fn delegator_validators(
        &self,
        request: Request<v1beta1::QueryDelegatorValidatorsRequest>,
    ) -> Result<Response<v1beta1::QueryDelegatorValidatorsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/delegator_validators {:?}", request);
        }
        unimplemented!();
    }

    async fn historical_info(
        &self,
        request: Request<v1beta1::QueryHistoricalInfoRequest>,
    ) -> Result<Response<v1beta1::QueryHistoricalInfoResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/historical_info {:?}", request);
        }
        unimplemented!();
    }

    async fn pool(
        &self,
        request: Request<v1beta1::QueryPoolRequest>,
    ) -> Result<Response<v1beta1::QueryPoolResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/pool   {:?}", request);
        }
        unimplemented!();
    }

    async fn params(
        &self,
        request: Request<v1beta1::QueryParamsRequest>,
    ) -> Result<Response<v1beta1::QueryParamsResponse>, Status> {
        if self.verbose {
            log!(Log::GRPC, "/staking/params {:?}", request);
        }
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
