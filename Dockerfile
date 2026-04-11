# --- STAGE 1: Builder ---
# [ARCH-COMPLIANCE FIX]: edition2024 ve hashbrown 0.17.0 hatalarını önlemek için 
# eski cargo-chef yerine doğrudan güncel rust imajı kullanılıyor.
FROM rust:1.93-slim-bookworm AS builder

# Protoc ve gerekli derleme araçları
RUN apt-get update && \
    apt-get install -y git pkg-config libssl-dev protobuf-compiler curl cmake && \
    rm -rf /var/lib/apt/lists/*

# Build argümanlarını tanımla
ARG GIT_COMMIT="unknown"
ARG BUILD_DATE="unknown"
ARG SERVICE_VERSION="0.0.0"

WORKDIR /app
COPY . .

# Build-time environment değişkenlerini ayarla
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV SERVICE_VERSION=${SERVICE_VERSION}

# Release derlemesi
RUN cargo build --release --bin sentiric-tts-gateway-service

# --- STAGE 2: Final (Minimal) ---
FROM debian:bookworm-slim

# Healthcheck ve TLS için sistem paketleri
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl-dev netcat-openbsd curl \
    && rm -rf /var/lib/apt/lists/*

# Güvenlik: Non-root kullanıcı
RUN useradd -m -u 1001 appuser
USER appuser
WORKDIR /app

# Binary'i al
COPY --from=builder /app/target/release/sentiric-tts-gateway-service .

# Varsayılan ortam değişkenleri
ENV RUST_LOG=info
ENV TTS_GATEWAY_SERVICE_LISTEN_ADDRESS=0.0.0.0
ENV TTS_GATEWAY_SERVICE_HTTP_PORT=14010
ENV TTS_GATEWAY_SERVICE_GRPC_PORT=14011

EXPOSE 14010 14011 14012

ENTRYPOINT ["./sentiric-tts-gateway-service"]