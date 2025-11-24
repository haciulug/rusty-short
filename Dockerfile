FROM rust:slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/rustyshort*

COPY . .
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rustyshort /usr/local/bin/rustyshort
COPY --from=builder /app/migrations ./migrations

ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

EXPOSE 8080

CMD ["rustyshort"]

