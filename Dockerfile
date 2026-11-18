# ---------- Stage 1: Build ----------
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release/deps/nonton_rust*

# Build the real app (templates are compiled into the binary by Askama)
COPY src ./src
COPY templates ./templates
RUN cargo build --release

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/nonton-rust /app/nonton-rust
COPY static ./static

ENV PORT=8080
EXPOSE 8080
CMD ["/app/nonton-rust"]
