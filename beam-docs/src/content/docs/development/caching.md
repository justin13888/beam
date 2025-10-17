---
title: Caching
description: How to efficiently cache terabytes of data on Beam.
---

Caching forms a critical part of Beam's distributed architecture. Streaming large media files, often generated on-the-fly, can lead to significant network and I/O. We prefer multiple layers of caching to catch various scenarios but it is important to agree on strategies to avoid redundant processing and excessive cache usage.

We consider the role of caching at various layers below...

## Client

- HTTP requests both small and big have caching headers. This is the cheapest as it does not involve any server-side resources but assumes clients behave correctly.
- Media files downloaded are used instead of re-downloading from server. Ephemeral IDs to all files are used to avoid accidental reuse of stale files.

## CDN

*Most self-hosted deployments won't have this but this is self-explanatory for those who chose to set it up.*

## Reverse Proxy (at origin)

A reverse proxy (could be different ones for different services) is strongly recommended to sit in front of all Beam services. Each service is written to be lean and avoids bulk caching itself (it simply serves Cache-Control headers).

For Kubernetes deployment specifically, you may choose any ingress controller of your choice for majority of the HTTP traffic but the large files from `beam-stream`, we recommend letting it go through and be handled by the application server's own caching layer (see below).

## Application Server

While all the previous layers cache on the HTTP level, certain optimizations require application-level caching. For example, let's say you want to download a 100GB+ file that is normally generated on-the-fly. Normally, a cold request would have high latency; instead, we may cache the generated file on disk for subsequent requests to be served directly from disk.

## Concluding Remarks

In practice, besides specific scenarios, responses are handled using various caches (e.g., in-memory Redis cache, cached metadata in Postgres). During development, just employ sufficient unit and integration testing while minimizing developer burden.
