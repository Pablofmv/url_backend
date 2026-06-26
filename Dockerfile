FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/url_backend /app/url_backend

ENV PORT=3000

EXPOSE 3000

CMD ["/app/url_backend"]