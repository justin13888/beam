# beam-tasks

`beam-tasks` is a GRPC microservice that manages background tasks such as transcoding, indexing, and metadata retrieval.

## Features

WIP

<!-- TODO -->

## Development

### Prerequisites

- Rust and Cargo

### Setup

```sh
cargo run
```

or

```sh
cargo watch -x run
```

Default port is 8090. Prometheus metrics port is 8091.

### Building Docker image

```sh
docker build -t beam-tasks .
```
