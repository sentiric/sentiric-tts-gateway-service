# ⚡ Sentiric TTS Gateway Service - Görev Listesi

Bu belge, `tts-gateway-service`'in geliştirme yol haritasını ve önceliklerini tanımlar.

---

### Faz 1: Temel Yönlendirme ve Proxy (Sıradaki Öncelik)

Bu faz, servisin temel bir yönlendirici olarak çalışmasını hedefler.

-   [ ] **Görev ID: TTS-GW-001 - Proje İskeleti (Rust/Tonic)**
    -   **Açıklama:** `SynthesizeSpeech` RPC'sini tanımlayan bir Protobuf ile temel bir Tonic gRPC sunucusu oluştur.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: TTS-GW-002 - Statik Yönlendirme**
    -   **Açıklama:** Gelen tüm istekleri, varsayılan olarak `edge-tts-service`'e yönlendiren basit bir proxy mantığı implemente et.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: TTS-GW-003 - `agent-service` Entegrasyonu**
    -   **Açıklama:** `agent-service`'in artık doğrudan uzman motorlara değil, bu gateway'e istek atmasını sağla.
    -   **Durum:** ⬜ Planlandı.

---

### Faz 2: Akıllı Özellikler

Bu faz, servise "akıllı" yeteneklerini kazandırmayı hedefler.

-   [ ] **Görev ID: TTS-GW-004 - Redis Önbellekleme**
    -   **Açıklama:** Bir sentezleme isteği geldiğinde önce Redis'te bu metin için bir önbellek kaydı olup olmadığını kontrol et. Varsa, sesi doğrudan Redis'ten dön. Yoksa, uzman motordan gelen sesi Redis'e kaydet.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: TTS-GW-005 - Akıllı Yönlendirme Mantığı**
    -   **Açıklama:** İstekte `speaker_wav_url` varsa `coqui-tts-service`'e, `quality="premium"` ise `elevenlabs-tts-service`'e, diğer durumlarda `edge-tts-service`'e yönlendiren bir mantık ekle.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: TTS-GW-006 - Temel SSML Desteği**
    -   **Açıklama:** Gelen metindeki `<break time="500ms"/>` gibi etiketleri anla ve sentezlenmiş ses parçaları arasına uygun sürede sessizlik ekle.
    -   **Durum:** ⬜ Planlandı.