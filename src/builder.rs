//! # Tendermock Builder API
//!
//! This modules holds the builder API, which can be use to create and start a mocked chain easily.
//!
//! It is the public API for interacting with Tendermock.

use crate::config::Config;
use crate::grpc;
use crate::init;
use crate::jrpc;
use crate::logger::Log;
use crate::node;
use crate::store;

use futures::future::try_join_all;
use futures::try_join;
use tokio;

use std::net::SocketAddr;
use std::path::Path;

/// Tendermock builder object.
pub struct Tendermock {
    /// Interval between new blocks, in seconds.
    growth_interval: u64,

    /// A list of interfaces for the chain, the first address for JsonRPC and the second for gRPC.
    interfaces: Vec<(SocketAddr, SocketAddr)>,

    /// The genesis block configuration.
    config: Config,

    /// Activate/deactivate verbose logging.
    verbose: bool,
}

impl Tendermock {
    /// Return a new Tendermock object with default configuration:
    /// - Grow interval: 0 (no growth)
    /// - Interfaces: [] (no interfaces)
    /// - Config: default
    /// - verbose: false (no logs)
    pub fn new() -> Self {
        Tendermock {
            growth_interval: 0,
            interfaces: vec![],
            config: Config::default(),
            verbose: false,
        }
    }

    /// Set the interval between two new blocks, in seconds.
    pub fn growth_rate(&mut self, interval: u64) -> &mut Self {
        self.growth_interval = interval;
        self
    }

    /// Add a new interface (one JsonRPC and one gRPC address).
    pub fn add_interface(&mut self, jrpc: SocketAddr, grpc: SocketAddr) -> &mut Self {
        self.interfaces.push((jrpc, grpc));
        self
    }

    /// Load the configuration of the genesis block.
    pub fn load_config<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.config = Config::load(path);
        self
    }

    /// Enable or disable verbose logging, default to off.
    pub fn verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    /// Start the Tendermock node.
    ///
    /// This call is blocking, for running multiple nodes simultaneously threading can be used (a
    /// scheduler will run on each thread).
    pub fn start(&self) {
        // Initialize node
        let node = node::Node::new(&self.config);
        let mut node = node.shared();
        init::init(&mut node, &self.config);

        // Build servers
        let mut jrpc_servers = Vec::new();
        let mut grpc_servers = Vec::new();
        if self.interfaces.is_empty() {
            log!(Log::Chain, "Warning: no interface configured");
        }
        for (jrpc_addr, grpc_addr) in &self.interfaces {
            if self.verbose {
                log!(Log::GRPC, "Listening on: {}", &grpc_addr);
                log!(Log::JRPC, "Listening on: {}", &jrpc_addr);
            }
            let jrpc_server = jrpc::serve(node.clone(), self.verbose, jrpc_addr.clone());
            let grpc_server = grpc::serve(node.clone(), self.verbose, grpc_addr.clone());
            jrpc_servers.push(jrpc_server);
            grpc_servers.push(grpc_server);
        }

        // Start servers
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                try_join!(
                    try_join_all(jrpc_servers),
                    try_join_all(grpc_servers),
                    schedule_growth(node, self.growth_interval, self.verbose)
                )
            })
            .unwrap();
    }
}

/// Push a new block on the chain every `interval` seconds.
async fn schedule_growth<S: store::Storage>(
    node: node::SharedNode<S>,
    interval: u64,
    verbose: bool,
) -> Result<(), std::convert::Infallible> {
    node.grow();
    if verbose {
        display_last_block(&node);
    }
    if interval == 0 {
        return Ok(());
    }
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        let node_ref = node.write();
        node_ref.get_chain().grow();
        drop(node_ref);
        if verbose {
            display_last_block(&node);
        }
    }
}

/// Displays the last block of the node's chain.
fn display_last_block<S: store::Storage>(node: &node::SharedNode<S>) {
    let node = node.read();
    let block = node.get_chain().get_block(0).unwrap();
    let header = block.signed_header.header;
    log!(
        Log::Chain,
        "Height: {} - Hash: {}",
        header.height,
        &header.hash()
    );
}
