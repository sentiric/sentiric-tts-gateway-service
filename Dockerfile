# --- STAGE 1: Builder ---
FROM rust:1.88-bullseye AS builder

# Gerekli tüm derleme araçlarını kur
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

# Derlemeyi yap
RUN cargo build --release

# --- STAGE 2: Final ---
FROM debian:bullseye-slim

# DÜZELTME: Healthcheck için 'netcat' aracını ekliyoruz.
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 netcat-openbsd && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ARG SERVICE_NAME
COPY --from=builder /app/target/release/${SERVICE_NAME} .

# --- EN KRİTİK DÜZELTME BURADA ---
# Kopyalanan dosyaya çalıştırma izni veriyoruz.
RUN chmod +x ./${SERVICE_NAME}

# Güvenlik için root olmayan bir kullanıcıyla çalıştır
RUN useradd -m -u 1001 appuser
USER appuser

# Argümanı ENV değişkenine ata
ENV SERVICE_NAME=${SERVICE_NAME}

ENTRYPOINT ["./sentiric-tts-gateway-service"]