# Compile source code
FROM rust:alpine as builder
RUN apk update && apk add --no-cache musl-dev gcc protobuf-dev && mkdir -p /app
WORKDIR /app
COPY ./ ./
RUN cargo build --release

# Copy binary to a new image
FROM alpine
RUN adduser -D beam
COPY --from=builder --chown=beam:beam --chmod=755 /app/target/release/beam-index /usr/local/bin/beam-index
USER beam

ENV MEDIA_PATH=/media
EXPOSE 50051
CMD ["beam-index"]
