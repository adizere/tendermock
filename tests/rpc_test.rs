//! Integration tests for tendermock JsonRPC and gRPC server.
use std::process::{Command, Stdio};

use ibc_proto::cosmos::staking::v1beta1::query_client::QueryClient;
use ibc_proto::cosmos::staking::v1beta1::QueryParamsRequest;

use tendermock::Tendermock;

const JSON_RPC_ADDR: &str = "127.0.0.1:26657";
const JSON_RPC_ADDR_2: &str = "127.0.0.1:26658";
const GRPC_ADDR: &str = "127.0.0.1:50051";
const GRPC_ADDR_2: &str = "127.0.0.1:50052";
const GRPC_URL: &str = "http://127.0.0.1:50051";
const JRPC_QUERIES: &[&str] = &[
    "abci_info.json",
    "abci_query.json",
    "block.json",
    "commit.json",
    "genesis.json",
    "status.json",
    "validators.json",
];

/// Spawns a `Tendermock` instance in a separate thread.
fn start_server() {
    let mut node = Tendermock::default();
    node.add_interface(JSON_RPC_ADDR.parse().unwrap(), GRPC_ADDR.parse().unwrap())
        .add_interface(
            JSON_RPC_ADDR_2.parse().unwrap(),
            GRPC_ADDR_2.parse().unwrap(),
        );
    std::thread::spawn(move || node.start());
    std::thread::sleep(std::time::Duration::new(2, 0));
}

#[tokio::test]
async fn rpc() {
    start_server();
    test_grpc().await;
    for query in JRPC_QUERIES {
        test_json_rpc(query, JSON_RPC_ADDR);
    }
    test_json_rpc(JRPC_QUERIES[0], JSON_RPC_ADDR_2)
}

async fn test_grpc() {
    let mut client = QueryClient::connect(GRPC_URL).await.unwrap();
    let request = tonic::Request::new(QueryParamsRequest {});
    client
        .params(request)
        .await
        .expect("gRPC 'param' request failed");
}

fn test_json_rpc(query: &str, jrpc_addr: &str) {
    let json_response = Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(&format!("@queries/{}", query))
        .arg(jrpc_addr)
        .stdout(Stdio::piped())
        .spawn()
        .expect("HTTP request failed")
        .stdout
        .unwrap();

    let object_raw: serde_json::error::Result<serde_json::Value> =
        serde_json::from_reader(json_response);
    assert!(object_raw.is_ok(), "Failed for query {}", query);

    let object = object_raw.unwrap();
    assert!(object.is_object(), "Failed for query {}", query);
    let obj_kv = object.as_object().unwrap();

    // Check the 'result' field
    let res = obj_kv.get("result");
    assert!(res.is_some(), "Failed for query {}", query);
    let res_inner = res.unwrap();
    assert_ne!(res_inner.is_null(), true, "Failed for query {}", query); // Shouldn't have a 'null' here.

    // Check the 'error' field
    assert_ne!(
        obj_kv.contains_key("error"),
        true,
        "Failed for query {}",
        query
    ); // Shouldn't have an 'error'.
}
