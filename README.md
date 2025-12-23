# 🗣️ Sentiric TTS Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Security](https://img.shields.io/badge/security-mTLS-green.svg)]()
[![Architecture](https://img.shields.io/badge/architecture-layer_3_gateway-blue.svg)]()

**Sentiric İletişim İşletim Sistemi**'nin "Sözcü"südür. Platformdaki tüm ses sentezleme (Text-to-Speech) isteklerinin tek güvenli giriş noktasıdır. İsteğin `voice_selector` parametresine göre trafiği doğru "Uzman Motora" (Coqui veya MMS) yönlendirir.

## 🎯 Temel Yetenekler

1.  **Çoklu Motor Desteği (Multi-Engine Routing):**
    *   **Coqui XTTS v2:** Duygusal ve yüksek kaliteli sesler (`coqui:` ön eki).
    *   **Facebook MMS:** Hızlı ve düşük kaynaklı Türkçe sesler (`mms:` ön eki).
2.  **Güvenli İletişim (Zero Trust):** Hem istemcilerle (Agent) hem de motorlarla (Upstream) olan iletişimi **mTLS** ile şifreler.
3.  **Gerçek Zamanlı Akış (Streaming):** Motorlardan gelen ses parçalarını (chunks) bellekte biriktirmeden (Zero-Copy) anlık olarak istemciye iletir.
4.  **Protokol Standardizasyonu:** Tüm motorları `sentiric.tts.v1` gRPC kontratı arkasında soyutlar.

## 🏗️ Mimari Konum

*   **Üst Akış (Callers):** `telephony-action-service` (mTLS Client).
*   **Alt Akış (Upstreams):**
    *   `tts-coqui-service` (Python / gRPC / mTLS)
    *   `tts-mms-service` (Python / gRPC / mTLS)

## 📦 Kurulum ve Çalıştırma

### Gereksinimler
*   Rust (1.75+)
*   `sentiric-certificates` tarafından üretilmiş sertifikalar (`/certs` dizininde olmalı).

### Ortam Değişkenleri (.env)
```bash
# Servis Ayarları
HOST=0.0.0.0
GRPC_PORT=14011

# Hedef Motorlar
TTS_COQUI_URL=http://tts-coqui-service:14031
TTS_MMS_URL=http://tts-mms-service:14061

# Güvenlik (Zorunlu)
GRPC_TLS_CA_PATH=../sentiric-certificates/certs/ca.crt
TTS_GATEWAY_SERVICE_CERT_PATH=../sentiric-certificates/certs/tts-gateway-service.crt
TTS_GATEWAY_SERVICE_KEY_PATH=../sentiric-certificates/certs/tts-gateway-service.key
```

### Başlatma
```bash
# Local Development
make up

# Production Build
cargo build --release
```