# 📋 Teknik Şartname

## 1. Servis Kimliği
*   **Adı:** `sentiric-tts-gateway-service`
*   **Dil:** Rust (Tokio / Tonic)
*   **Port Bloğu:** 1401X (Harmonik Mimari)

## 2. Kaynak Tüketimi
*   **CPU:** Idle durumda < %1, Yük altında (500 stream/s) < %15 (Tek Çekirdek)
*   **RAM:** < 50 MB (Stateless olduğu için)

## 3. API Kontratı
Servis, `sentiric-contracts` v1.12.3 sürümünü kullanır.

### Ana RPC: `SynthesizeStream`
*   **Girdi:** `SynthesizeStreamRequest`
    *   `text`: Sentezlenecek metin.
    *   `voice_id`: Yönlendirme anahtarı (örn: `coqui:ece`).
    *   `prosody`: Hız, tonlama ayarları.
*   **Çıktı:** `SynthesizeStreamResponse` (Stream)
    *   `audio_content`: Ham PCM ses verisi.
    *   `provider_used`: Hangi motorun kullanıldığı (`coqui` veya `mms`).

## 4. Hata Yönetimi
*   **Upstream Unavailable:** Hedef motor (örn: Coqui) kapalıysa, anında `Status::UNAVAILABLE` döner ve akış kapatılır.
*   **Unknown Provider:** Tanımsız bir `voice_id` gelirse varsayılan olarak Coqui denenir.