# 🗣️ Sentiric TTS Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Architecture](https://img.shields.io/badge/architecture-layer_3_gateway-blue.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()

**Sentiric İletişim İşletim Sistemi**'nin "Sözcü"südür. Platformdaki tüm ses sentezleme (Text-to-Speech) isteklerinin tek giriş noktasıdır. İsteğin niteliğine, istenen sesin kalitesine ve maliyet politikasına göre doğru "Uzman Motoru" (Coqui, Edge, ElevenLabs vb.) seçer ve yönetir.

## 🎯 Temel Sorumluluklar

1.  **Protokol Soyutlama:** İç servislerden (Agent, Telephony) gelen gRPC isteklerini, arka plandaki motorların anlayacağı formatlara (REST/gRPC) dönüştürür.
2.  **Akıllı Yönlendirme (Smart Routing):** `voice_selector` parametresine bakarak trafiği yönlendirir (örn: `coqui:ana` -> Coqui Service, `eleven:rachel` -> ElevenLabs).
3.  **Streaming Proxy:** Arka plandaki motorlardan gelen ses parçalarını (chunks) biriktirmeden, gerçek zamanlı olarak istemciye (RTP sunucusuna) akıtır. Düşük gecikme (Latency) kritiktir.
4.  **Hata Yönetimi (Failover):** Bir motor çökerse, varsayılan (fallback) motora geçiş yapar.

## 🏗️ Mimari Konum

Bu servis **Katman 3 (Ağ Geçitleri)** seviyesinde yer alır.

*   **Üst Akış (Callers):** `telephony-action-service`, `agent-service`.
*   **Alt Akış (Downstreams):**
    *   `tts-coqui-service` (Yüksek Kalite / Yerel)
    *   Öncelikle Yerel alternatifler planlanıyor...
    *   `tts-edge-service` (Hızlı / Ücretsiz ( Alternatif))
    *   `tts-elevenlabs-service` (Premium / Bulut) ( Alternatif)

## 📦 Kurulum ve Çalıştırma

### Gereksinimler
*   Rust (1.75+)
*   Protobuf Compiler (`protoc`)

### Komutlar
```bash
# Ortamı hazırla (.env oluştur)
make setup

# Servisi başlat (Docker)
make up

# Logları izle
make logs
```

## 🔌 API ve Portlar

*   **gRPC (14011):** `sentiric.tts.v1.TextToSpeechService` (Ana Servis)
*   **HTTP (14010):** `/health`, `/metrics` (Operasyonel)