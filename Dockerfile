# --- STAGE 1: Builder ---
FROM rust:1.88-bullseye AS builder

# YENİ: Build argümanlarını tanımla
ARG GIT_COMMIT
ARG BUILD_DATE
ARG SERVICE_VERSION

# Gerekli derleme araçlarını kur
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    curl \
    && curl -sSL "https://github.com/bufbuild/buf/releases/download/v1.35.1/buf-Linux-x86_64" -o /usr/local/bin/buf \
    && chmod +x /usr/local/bin/buf \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Projenin tamamını kopyala
COPY . .

# YENİ: Build-time environment değişkenlerini ayarla
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV SERVICE_VERSION=${SERVICE_VERSION}

# Derlemeyi yap
RUN cargo build --release

# --- STAGE 2: Final ---
FROM debian:bullseye-slim

# DÜZELTME: libssl3 yerine, bullseye'da bulunan libssl1.1'i kuruyoruz.
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 netcat-openbsd && rm -rf /var/lib/apt/lists/*

# YENİ: Build argümanlarını tekrar tanımla ki runtime'da da kullanılabilsin
ARG GIT_COMMIT
ARG BUILD_DATE
ARG SERVICE_VERSION
ARG SERVICE_NAME

# YENİ: Argümanları environment değişkenlerine ata
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV SERVICE_VERSION=${SERVICE_VERSION}
ENV SERVICE_NAME=${SERVICE_NAME}

WORKDIR /app

COPY --from=builder /app/target/release/sentiric-tts-gateway-service .

# Kopyalanan dosyaya çalıştırma izni ver
RUN chmod +x ./sentiric-tts-gateway-service

# Güvenlik için root olmayan bir kullanıcıyla çalıştır
RUN useradd -m -u 1001 appuser
USER appuser

ENTRYPOINT ["./sentiric-tts-gateway-service"]