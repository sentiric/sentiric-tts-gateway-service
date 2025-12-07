
# 🗣️ Sentiric TTS Gateway Service

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()

Sentiric platformunun **akıllı ses üretim santralidir.** `agent-service` gibi orkestratörlerden gelen gRPC isteklerini karşılar, Redis üzerinde önbellekleme yapar ve isteği en uygun "uzman" TTS motoruna (`edge-tts`, `coqui-tts` vb.) yönlendirir.

## 🚀 Özellikler

*   **Protokol Dönüşümü:** gRPC (İç) -> HTTP/REST (Motorlar).
*   **Akıllı Yönlendirme:** İsteğin parametrelerine (hız, klonlama ihtiyacı) göre motor seçimi.
*   **Önbellekleme:** Sık kullanılan sentezleri Redis'te tutarak maliyet ve süre tasarrufu.
*   **Yüksek Performans:** Rust (Tonic/Axum) ile minimum kaynak tüketimi.

## 📦 Kurulum ve Çalıştırma

### Docker ile Hızlı Başlatma (Geliştirme)

```bash
# 1. Hazırlık
make setup

# 2. Başlatma (Mock motorlarla)
make up

# 3. Logları İzleme
make logs
```

### Manuel Çalıştırma (Rust)

```bash
# Bağımlılıkları yükle (Debian/Ubuntu)
sudo apt install protobuf-compiler libssl-dev

# Çalıştır
cargo run
```

## 🔌 API

*   **gRPC (14011):** `sentiric.tts.v1.TextToSpeechService`
*   **HTTP (14010):** `/healthz` (Sağlık kontrolü)
