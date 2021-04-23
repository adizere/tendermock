use ibc::ics24_host::identifier::ClientId;
use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;

pub trait GrpcContext {
    /// Fetches the vector of all the consensus states associated to client with id `client_id`.
    fn consensus_states(&self, client_id: &ClientId) -> Vec<ConsensusStateWithHeight>;
}
