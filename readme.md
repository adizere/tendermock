# Tendermock

A fake Tendermint for testing purposes.

## How to use:

The fake node can be run with:

```sh
cargo run
```

A few options are available:

```
USAGE:
    tendermock [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --block <block>            Seconds between two blocks, 0 for no growth [default: 3]
    -c, --config <config>          Path to json configuration file
    -g, --grpc-port <grpc-port>    [default: 50051]
    -j, --json-port <json-port>    JsonRPC port [default: 26657]
```

An example of a valid config can be found in `test/config.example.json`, which can be used like that:

```sh
cargo run -- -c config/config.example.json -v
```

## Sending queries

A few example queries are available in `./queries`, the node can easily be queried using curl:

```
curl -X POST -H 'Content-Type: application/json' -d @queries/block.json 127.0.0.1:26657/ | jq
```

## Using as a library

Tendermock can also be used as a library through a builder API, this can be useful for integration testing by spawning a server from Rust code:

```rust
use tendermock::Tendermock;

let jrpc_addr = format!("127.0.0.1:{}", 5000).parse().unwrap();
let grpc_addr = format!("127.0.0.1:{}", 6000).parse().unwrap();

Tendermock::new()
    .growth_rate(10)
    .add_interface(jrpc_addr, grpc_addr)
    .start();
```

## Building the doc

Run the following commands:

```sh
cargo doc --no-deps
rm -rf docs
mv target/doc docs
```

## Testing with the relayer

see [this example](https://gist.github.com/CharlyCst/48fc02d4c1e6ed59e6395ee6ae766703)
