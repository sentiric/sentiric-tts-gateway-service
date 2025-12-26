# --- STAGE 1: Chef (Planlama) ---
FROM lukemathwalker/cargo-chef:latest-rust-1.84-bookworm AS chef
WORKDIR /app

# --- STAGE 2: Planner ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- STAGE 3: Builder ---
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Protobuf derleyicisini kur
RUN apt-get update && apt-get install -y protobuf-compiler cmake && rm -rf /var/lib/apt/lists/*

# Bağımlılıkları derle
RUN cargo chef cook --release --recipe-path recipe.json

# Kaynak kodları kopyala ve derle
COPY . .
RUN cargo build --release --bin sentiric-tts-gateway-service

# --- STAGE 4: Runtime (Minimal) ---
FROM debian:bookworm-slim AS runtime

# curl ve netcat-openbsd ekledik
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser
USER appuser
WORKDIR /app

COPY --from=builder /app/target/release/sentiric-tts-gateway-service /app/

# Varsayılan Env Vars
ENV RUST_LOG=info
ENV TTS_GATEWAY_SERVICE_LISTEN_ADDRESS=0.0.0.0
ENV TTS_GATEWAY_SERVICE_GRPC_PORT=14011

EXPOSE 14010 14011 14012

ENTRYPOINT ["./sentiric-tts-gateway-service"]