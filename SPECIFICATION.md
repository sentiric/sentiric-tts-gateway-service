# 📋 Teknik Şartname (Specification)

## 1. Servis Kimliği
*   **Adı:** `sentiric-tts-gateway-service`
*   **Dil:** Rust
*   **Port Bloğu:** 1401X (Harmonik Mimari)

## 2. API Kontratı (gRPC)

Servis, `sentiric-contracts` reposundaki `sentiric.tts.v1` paketini implemente eder.

### Proto Tanımı (`tts.proto`)

```protobuf
service TextToSpeechService {
  rpc Synthesize(SynthesizeRequest) returns (stream SynthesizeResponse);
}

message SynthesizeRequest {
  string text = 1;              // Sentezlenecek metin (SSML olabilir)
  string language_code = 2;     // örn: "tr-TR", "en-US"
  string voice_selector = 3;    // örn: "coqui:ece", "edge:ahmet"
  float speed = 4;              // 1.0 normal, 0.5 yavaş, 2.0 hızlı
  float pitch = 5;              // 1.0 normal
  int32 volume_gain_db = 6;     // Desibel artışı/azalışı
}

message SynthesizeResponse {
  bytes audio_content = 1;      // Ham ses verisi (PCM/OPUS)
  bool is_final = 2;            // Stream bitti mi?
}
```

## 3. Ortam Değişkenleri (Environment Variables)

Bu servis çalışmak için aşağıdaki konfigürasyonları `.env` dosyasından veya Docker ortamından bekler:

| Değişken | Zorunlu | Açıklama |
| :--- | :--- | :--- |
| `TTS_GATEWAY_SERVICE_GRPC_PORT` | Evet | Dinlenecek gRPC portu (Genelde 14011). |
| `TTS_COQUI_SERVICE_URL` | Hayır | Coqui motorunun adresi (http://tts-coqui-service:14030). |
| `TTS_EDGE_SERVICE_URL` | Evet | Edge motorunun adresi (Fallback için zorunlu). |
| `TTS_ELEVENLABS_SERVICE_URL` | Hayır | ElevenLabs motorunun adresi. |
| `REDIS_URL` | Evet | Önbellekleme için Redis adresi. |

## 4. Performans Hedefleri

*   **Time-to-First-Byte (TTFB):** < 200ms (İstekten ilk ses paketinin çıkışına kadar geçen süre).
*   **Throughput:** Tek bir instance, saniyede en az 50 eş zamanlı stream'i yönetebilmelidir.
*   **Memory Footprint:** Yük altında < 100MB RAM (Rust avantajı).