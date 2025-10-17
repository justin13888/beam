---
title: Streaming
description: How streaming works in Beam.
---

Streaming is achieved through an optimized streaming server [beam-stream](https://github.com/justin13888/beam/tree/master/beam-stream) and native clients for various platforms.

## Protocols

### HLS

This is the primary protocol used for adaptive bitrate (ABR) streaming. It supports H.264, H.265, and AV1 (as of September 2023). More importantly, it is natively supported to Apple devices, which is important for a native experience on Apple platforms (e.g., iOS, iPadOS, tvOS).

### DASH

*This is being developed but by no means is complete/stabilized.*

### Fragmented MP4 Streaming

Fragmented MP4 or fMP4 is possible for single bitrate streams. It is ideal for direct playback of original quality streams with minimal server-side processing. It is supported on most modern browsers and platforms.

When HLS/DASH are unavailable/unsupported, fMP4 can be used as a fallback or more likely, to support offline viewing on certain platforms. While fMP4 could be live-remuxed, it is intentionally unimplemented to avoid scalability issues and potential for client abuse. Instead, fMP4 files could be remuxed as whole (and cached) when download is requested.
