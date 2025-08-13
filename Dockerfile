# --- STAGE 1: Builder ---
# 'bullseye' (Debian 11) tabanını kullanmaya devam ediyoruz.
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

# --- STAGE 2: Final Runtime Image ---
# ---- DEĞİŞİKLİK BURADA ----
# 'debian:bookworm-slim' (Debian 12) yerine, builder ile aynı temel dağıtım olan
# 'debian:bullseye-slim' (Debian 11) kullanıyoruz.
FROM debian:bullseye-slim

# Çalışma zamanı için ca-certificates ve OpenSSL 1.1 kütüphanesi olan libssl1.1'i kuruyoruz.
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ARG SERVICE_NAME
COPY --from=builder /app/target/release/${SERVICE_NAME} .

# Güvenlik için root olmayan bir kullanıcıyla çalıştır
RUN useradd -m -u 1001 appuser
USER appuser

# ENTRYPOINT'in düzgün çalışması için ENV değişkenini eklemeyi unutmuyoruz
ENV SERVICE_NAME=${SERVICE_NAME}
ENTRYPOINT ["sh", "-c", "./$SERVICE_NAME"]