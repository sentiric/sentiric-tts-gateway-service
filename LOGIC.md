# 🧠 Mantık ve Yönlendirme Mimarisi (v2.0)

Bu belge, `tts-gateway-service`in bir isteği nasıl işlediğini ve hangi motora yönlendireceğine nasıl karar verdiğini açıklar.

## 1. Yönlendirme Algoritması (Routing Logic)

Gateway, gelen `SynthesizeStreamRequest` içindeki `voice_id` alanını analiz eder.

| Ön Ek (Prefix) | Hedef Servis | Protokol | Örnek `voice_id` |
| :--- | :--- | :--- | :--- |
| `mms:` | **MMS TTS** | gRPC Stream (mTLS) | `mms:tr`, `mms:tur` |
| `coqui:` | **Coqui TTS** | gRPC Stream (mTLS) | `coqui:default`, `coqui:F_TR_Genc_Selin` |
| *(Diğer)* | **Coqui TTS** | gRPC Stream (mTLS) | *Varsayılan Fallback* |

## 2. Akış (Flow)

1.  **Request:** İstemci `SynthesizeStream` çağırır.
2.  **Metadata:** `x-trace-id` header'ı okunur ve loglara eklenir.
3.  **Routing:** `voice_id` parse edilir ve istemci (MmsClient veya CoquiClient) seçilir.
4.  **Connection:** Seçilen istemci, Upstream servise **mTLS** ile bağlanır (Lazy Connection).
5.  **Streaming:** Upstream'den gelen ses paketleri (`audio_chunk`), `SynthesizeStreamResponse` formatına sarılarak (Map) istemciye iletilir.
6.  **Error Handling:** Upstream kapalıysa veya hata dönerse, Gateway `Status::Unavailable` döner.
