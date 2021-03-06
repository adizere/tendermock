use clap::Clap;
use tendermock::Tendermock;

// Define the CLI interface (see Clap doc)
#[derive(Clap)]
#[clap(version = "0.1.0")]
#[clap(verbatim_doc_comment)]
/// Tendermock - a mocked Tendermint node
pub struct Args {
    /// Verbode mode
    #[clap(short, long)]
    pub verbose: bool,

    /// JsonRPC port
    #[clap(short, long, default_value = "26657")]
    pub json_port: u16,

    /// gRPC port
    #[clap(short, long, default_value = "50051")]
    pub grpc_port: u16,

    /// Path to json configuration file
    #[clap(short, long)]
    pub config: Option<String>,

    /// Seconds between two blocks, 0 for no growth
    #[clap(short, long, default_value = "20")]
    pub block: u64,
}

fn main() {
    // Parse cli arguments & initialize store
    let args = Args::parse();
    let jrpc_addr = format!("127.0.0.1:{}", args.json_port).parse().unwrap();
    let grpc_addr = format!("127.0.0.1:{}", args.grpc_port).parse().unwrap();
    let mut tendermock = Tendermock::new();
    tendermock
        .verbose(args.verbose)
        .add_interface(jrpc_addr, grpc_addr)
        .growth_rate(args.block);
    if let Some(config_path) = args.config {
        tendermock.load_config(config_path);
    }
    tendermock.start();
}

