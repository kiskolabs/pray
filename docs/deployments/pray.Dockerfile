FROM rust:1.78-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev git
WORKDIR /src
COPY . .
RUN cargo build --release -p pray

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /src/target/release/pray /usr/local/bin/pray
EXPOSE 7429
CMD ["pray", "serve", "--root", "/data/prayers", "--host", "0.0.0.0", "--port", "7429"]
