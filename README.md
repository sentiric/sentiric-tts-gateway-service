# 🗣️ Sentiric TTS Gateway Service

[![Status](https://img.shields.io/badge/status-production_ready-success.svg)]()
[![Security](https://img.shields.io/badge/security-mTLS-green.svg)]()
[![Architecture](https://img.shields.io/badge/architecture-layer_3_gateway-blue.svg)]()

**Sentiric İletişim İşletim Sistemi**'nin "Sözcü"südür. Platformdaki tüm ses sentezleme (Text-to-Speech) isteklerinin tek güvenli giriş noktasıdır. İsteğin `voice_selector` parametresine göre trafiği doğru "Uzman Motora" (Coqui veya MMS) yönlendirir.

## 🎯 Temel Yetenekler

1.  **Çoklu Motor Desteği (Multi-Engine Routing):**
    *   **Coqui XTTS v2:** Duygusal ve yüksek kaliteli sesler (`coqui:` ön eki).
    *   **Facebook MMS:** Hızlı ve düşük kaynaklı Türkçe sesler (`mms:` ön eki).
2.  **Güvenli İletişim (Zero Trust):** Hem istemcilerle (Agent) hem de motorlarla (Upstream) olan iletişimi **mTLS** ile şifreler.
3.  **Gerçek Zamanlı Akış (Streaming):** Motorlardan gelen ses parçalarını (chunks) bellekte biriktirmeden (Zero-Copy) anlık olarak istemciye iletir.
4.  **Dayanıklılık (Resilience):** Upstream servisler kapalı olsa bile Gateway çökmez (Lazy Connection).

## 🏗️ Mimari Konum

*   **Üst Akış (Callers):** `telephony-action-service` (mTLS Client).
*   **Alt Akış (Upstreams):**
    *   `tts-coqui-service` (Python / gRPC / mTLS / GPU) - Port 14031
    *   `tts-mms-service` (Python / gRPC / mTLS / GPU) - Port 14061

## 📦 Kurulum ve Çalıştırma

### Gereksinimler
*   Docker & Docker Compose
*   NVIDIA GPU (Önerilen)
*   `sentiric-certificates` sertifikaları

### Başlatma (Full Stack)
Gateway ve arkasındaki tüm motorları (MMS + Coqui) tek komutla ayağa kaldırır:

```bash
make up
```

### Test Etme (Manuel)
Gateway üzerinden motorları test etmek için (Sertifikalar gerektirir):

```bash
# MMS Motoru
grpcurl -insecure -d '{"voice_id": "mms:tur", "text": "Test"}' localhost:14011 sentiric.tts.v1.TtsGatewayService/SynthesizeStream

# Coqui Motoru
grpcurl -insecure -d '{"voice_id": "coqui:default", "text": "Test"}' localhost:14011 sentiric.tts.v1.TtsGatewayService/SynthesizeStream
```

## 🛠️ Konfigürasyon

| Değişken | Varsayılan | Açıklama |
|---|---|---|
| `TTS_COQUI_SERVICE_URL` | `https://tts-coqui-service:14031` | Coqui Motoru Adresi |
| `TTS_MMS_SERVICE_URL` | `https://tts-mms-service:14061` | MMS Motoru Adresi |
| `GRPC_TLS_CA_PATH` | `/certs/ca.crt` | Root CA Yolu |

---
