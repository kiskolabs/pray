# Fly.io deployment

Fly.io is a strong fit for a compact self-hosted distribution point.

## Recommended shape

- run `pray serve` in a container
- bind to `0.0.0.0:$PORT`
- use a persistent volume if you want local storage on the machine
- or keep artifacts in object storage and metadata in a database

## Dockerfile

```dockerfile
FROM rust:1.78-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev git
WORKDIR /src
COPY . .
RUN cargo build --release -p pray

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /src/target/release/pray /usr/local/bin/pray
EXPOSE 8080
CMD ["pray", "serve", "--root", "/data/prayers", "--host", "0.0.0.0", "--port", "8080"]
```

## fly.toml

```toml
app = "pray-registry"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 1

[[mounts]]
  source = "pray_data"
  destination = "/data"
```

## Operational notes

- Fly handles edge TLS
- volumes are useful for cache or local persistence
- external storage is still safer for durability and backups

## When Fly.io is a good fit

- one small service
- private team registry
- mirror with moderate traffic
- you want a straightforward container deployment