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

For Kubernetes deployment specifically, you may choose any ingress controller of your choice for majority of the HTTP traffic but the large files from `beam-stream` necessitates a dedicated caching proxy (e.g. Varnish or Nginx).

## Application Server

While files (which could be quite large) are not cached at the application layer, all the metadata used to construct responses are typically cached in-memory using Redis or in database (e.g., Postgres).
