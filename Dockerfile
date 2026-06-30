FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features native-tls,postgres

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/url_backend /app/url_backend
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

ENV PORT=3000

EXPOSE 3000

CMD sqlx migrate run && /app/url_backend