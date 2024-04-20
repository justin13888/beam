# Compile source code
FROM rust:alpine as builder
RUN apk update && apk add --no-cache musl-dev gcc libressl-dev && mkdir -p /app
WORKDIR /app
COPY ./ ./
RUN cargo build --release

# Copy binary to a new image
FROM alpine
RUN adduser -D beam
COPY --from=builder --chown=beam:beam --chmod=755 /app/target/release/beam-stream /usr/local/bin/beam-stream
USER beam

ENV MEDIA_PATH=/media
EXPOSE 8080/tcp
EXPOSE 8081/tcp
CMD ["beam-index"]
