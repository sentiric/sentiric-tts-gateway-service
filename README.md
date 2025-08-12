# ⚡ Sentiric TTS Gateway Service

[![Status](https://img.shields.io/badge/status-vision-lightgrey.svg)]()
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)]()

**Sentiric TTS Gateway Service**, Sentiric platformunun **akıllı ses üretim santralidir.** `agent-service`'ten gelen ses sentezleme isteklerini alır ve bu istekleri en uygun "uzman" TTS motoruna (`edge-tts`, `coqui-tts`, `elevenlabs-tts` vb.) akıllıca yönlendirir.

Bu servis, platformu tek bir TTS teknolojisine bağımlı olmaktan kurtarır ve maliyet, hız, kalite arasında dinamik bir denge kurmayı sağlar.

## 🎯 Temel Sorumluluklar (Vizyon)

*   **Akıllı Yönlendirme (Routing):** Gelen isteğin içeriğine (dil, SSML etiketleri, kalite talebi) göre en uygun uzman TTS motorunu seçer.
*   **Merkezi Önbellekleme (Caching):** Sık sentezlenen cümleleri Redis'te önbelleğe alarak, tekrar eden isteklerde AI motorlarını hiç çağırmadan yanıt döner. Bu, maliyeti düşürür ve hızı artırır.
*   **SSML Ayrıştırma (Parsing):** `<speak>` ve `<break>` gibi SSML etiketlerini anlar. Uzun metinleri, duraklamalara göre parçalara ayırıp farklı motorlarda paralel olarak sentezleyerek ilk sesin duyulma süresini (Time to First Audio) kısaltır.
*   **Dayanıklılık (Resilience):** Bir uzman motor çöktüğünde veya yavaşladığında, isteği otomatik olarak bir sonraki uygun motora yönlendirir (fallback).

## 🛠️ Teknoloji Yığını (Planlanan)

*   **Dil:** Rust (Yüksek performanslı I/O ve eşzamanlılık için)
*   **Asenkron Runtime:** Tokio
*   **Servisler Arası İletişim:** gRPC (Tonic ile)
*   **Cache:** Redis

## 🔌 API Etkileşimleri

*   **Gelen (Sunucu):**
    *   `sentiric-agent-service` (gRPC): `SynthesizeSpeech` RPC'sini çağırır.
*   **Giden (İstemci):**
    *   `sentiric-edge-tts-service` (gRPC/REST)
    *   `sentiric-coqui-tts-service` (gRPC/REST)
    *   `sentiric-elevenlabs-tts-service` (gRPC/REST)
    *   `Redis`: Önbellek okuma/yazma.

## 🤝 Katkıda Bulunma

Bu servis henüz geliştirme aşamasında olmasa da, fikirlerinizi ve önerilerinizi `sentiric-governance` reposunda bir `Issue` açarak paylaşabilirsiniz.

---
## 🏛️ Anayasal Konum

Bu servis, [Sentiric Anayasası'nın (v11.0)](https://github.com/sentiric/sentiric-governance/blob/main/docs/blueprint/Architecture-Overview.md) **Zeka & Orkestrasyon Katmanı**'nda yer alan merkezi bir bileşendir.