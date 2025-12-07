# --- STAGE 1: Chef (Caching Dependencies) ---
FROM lukemathwalker/cargo-chef:latest-rust-1.77-bookworm AS chef
WORKDIR /app

# --- STAGE 2: Planner ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- STAGE 3: Builder ---
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin sentiric-tts-gateway-service

# --- STAGE 4: Runtime ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser
USER appuser
WORKDIR /app

COPY --from=builder /app/target/release/sentiric-tts-gateway-service /app/

ENV RUST_LOG=info
EXPOSE 14010 14011 14012

ENTRYPOINT ["./sentiric-tts-gateway-service"]