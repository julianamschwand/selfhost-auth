FROM rust:1.95-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY data ./data
COPY .cargo ./.cargo
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.22.4

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/selfhost-auth ./selfhost-auth
RUN mkdir data
EXPOSE 8080
CMD ["./selfhost-auth"]
