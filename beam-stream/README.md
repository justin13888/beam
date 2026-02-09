# Beam Stream

A high-performance streaming service built with Rust and Axum.

## Development

- Install ffmpeg/libav 8+ libraries on your system.
  - *Tip: Refer to [Containerfile](Containerfile) for ffmpeg build flags used in prod.>*
- `cargo install cargo-watch systemfd`

- Copy `.env.example` to `.env` and modify as needed:

    ```bash
    cp .env.example .env
    ```

- Install some dependencies:

    ```bash
    cargo install cargo-watch systemfd
    ```

- Start other dependencies:

    ```bash
    podman compose -f compose.dependencies.yaml up -d
    ```

- Start development server:

    ```bash
    systemfd --no-pid -s http::8000 -- cargo watch -x run # note: this will override your bind address in .env
    # cargo watch -x run
    ```

### Build container image

```bash
# In root directory
podman build -f beam-stream/Containerfile -t beam-stream .
```

## API Documentation

See OpenAPI docs: `http://localhost:8000/openapi`
