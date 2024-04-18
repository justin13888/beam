# beam-index

A gRPC microservice for indexing and fetching metadata for media files.

## Development

### Prerequisites

- Rust and Cargo
- protoc
  - e.g. on Fedora: `sudo dnf install protobuf-compiler`
- (Recommended) `cargo install cargo-watch`

### Running

```sh
cargo watch -x 'run --bin beam-index'
```

or

```sh
cargo run
```

Default port is 50051.

### Building Docker image

```sh
docker build -t beam-index .
```
