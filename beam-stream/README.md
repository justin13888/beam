# Beam Stream

A high-performance streaming service built with Rust and Axum.

## Development

- Install ffmpeg/libav 8+ libraries on your system.

- Copy `.env.example` to `.env` and modify as needed:

    ```bash
    cp .env.example .env
    ```

- Install some dependencies:

    ```bash
    cargo install cargo-watch
    ```

- Start development server:

    ```bash
    cargo watch -x run
    ```

## API Documentation

See OpenAPI docs: `http://localhost:3000/openapi`
