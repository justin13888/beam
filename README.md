# beam

> NOTE: Major refactoring is to be expected next year on-and-off given time. It will be alpha-ready for testing and feedback later.

Beam is a high-performance, scalable media server to stream video, audio, and other content with real-time transcoding capabilities. It is ideal for home labs, small business that need to stream content on a variety of devices.

## Features

- [ ] HLS/DASH streaming
- [ ] Real-time remuxing/transcoding with hardware acceleration (NVENC, VAAPI)
- [ ] Fully-distributed and Kubernetes-native architecture

<!-- TODO: Finalize later -->

## Motivation

Beam originally started as a project to surpass the limitations of Jellyfin, a popular open-source media server. Jellyfin is a great project, but we need a more modern, straightforward solution that is as easy to use but more feature-rich and actively maintained.

## Architecture

Beam consists of multiple backend services that work together to provide a seamless media streaming experience. The main components are:

- `beam-stream`: Media streaming service that handles live transcoding, caching, and streaming of media files (Rust/GraphQL/gRPC/ffmpeg).
<!-- - `beam-auth`: Authentication and user management service that handles user registration, login, and permissions (Rust/GraphQL). -->
<!-- - `beam-tasks`: GRPC microservice that manages background tasks such as transcoding, indexing, and metadata retrieval (Rust/Tonic). -->
<!-- - `beam-index`: Media indexing service that scans and indexes media files on disk (Rust). -->
<!-- - `beam-recommendation`: Recommendation service that provides personalized content recommendations based on user preferences and viewing history (Python/PyTorch). TODO -->

Currently, there is one client app to interact with Beam:

- `beam-web`: Web frontend that provides a user-friendly interface to browse and stream media content (TypeScript/React).


<!-- TODO: Add architecture diagram -->

## Installation

WIP
<!-- TODO -->

## License

This project is licensed under the AGPL License - see the [LICENSE](LICENSE) file for details.
