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

### MP4 Streaming

A well-encoded MP4 is possible for single bitrate streams. It is ideal for direct playback of original quality streams with minimal server-side processing. It is supported on most modern browsers and platforms.

When HLS/DASH are unavailable/unsupported, MP4 can be used as a fallback or more likely, to support offline viewing on certain platforms. While fMP4 could be live-remuxed, it is intentionally unimplemented to avoid scalability issues and potential for client abuse. Instead, non-MP4 files are remuxed as whole (and cached) as whole MP4 files when download is requested.

#### MP4 Remuxing Guidelines

Here are the parameters used when remuxing to MP4:

- `frag_keyframe`: Fragment at keyframes for better seeking support.
- `empty_moov`: Place `moov` box at the start of the file for progressive downloading.
- `default_base_moof`: Use default base for `moof` box to improve
- ~~If keyframes are sparse (e.g., >= 6s), we insert additional keyframes to improve seeking performance.~~ (Currently skipped to avoid slowing down remuxing.)

#### Conversion MKV to MP4

While MP4 is relatively featureful while being widely supported, there exists some video containers, such as MKV, that cannot have all their features mapped to MP4. In these cases, some compromises are necessary:

- Subtitle tracks: MP4 primarily supports mov_text (limited), WebVTT (via side files). Hence, we expose subtitles as separate .vtt files.
- Multiple audio tracks: Ordering may not be preserved on some clients.
- Chapter markers: MP4 supports them but they need conversion.
- Attachment streams: MKV could embed fonts but that is omitted in MP4.

To tie everything together, when downloading videos as MP4 for offline viewing, Beam exposes the following types of files:

- `index.mp4`: The main video file with video/audio tracks remuxed to MP4.
- `*.vtt`: Subtitle tracks in WebVTT format.

#### Reasons for Fragmented MP4 over Standard MP4

Pro's:

- Allow for progressive downloading and playback.
- Trickplay support

Con's:

- Large file size: Fragmentation adds space overhead
- Compatibility: Some legacy players may not support fMP4 (less of an issue nowadays)
