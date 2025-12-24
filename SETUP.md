### 🧪 FİNAL ENTEGRASYON TESTİ

Artık "Connection Refused" veya "InvalidContentType" hatası almadan gerçek veri akışını görmeliyiz. Aşağıdaki komutları sırasıyla çalıştırın.

#### 1. MMS Testi (Hızlı & Türkçe)

```bash
docker run --rm --network sentiric-net \
  -v $(pwd)/../sentiric-certificates/certs:/certs:ro \
  -v $(pwd)/../sentiric-contracts/proto:/proto:ro \
  fullstorydev/grpcurl \
  -cacert /certs/ca.crt \
  -cert /certs/tts-gateway-service.crt \
  -key /certs/tts-gateway-service.key \
  -import-path /proto \
  -proto sentiric/tts/v1/gateway.proto \
  -d '{"voice_id": "mms:tur", "text": "MMS motoru şu an sertifikalı ve güvenli çalışıyor."}' \
  tts-gateway-service:14011 \
  sentiric.tts.v1.TtsGatewayService/SynthesizeStream
```

#### 2. Coqui Testi (Yüksek Kalite)

```bash
docker run --rm --network sentiric-net \
  -v $(pwd)/../sentiric-certificates/certs:/certs:ro \
  -v $(pwd)/../sentiric-contracts/proto:/proto:ro \
  fullstorydev/grpcurl \
  -cacert /certs/ca.crt \
  -cert /certs/tts-gateway-service.crt \
  -key /certs/tts-gateway-service.key \
  -import-path /proto \
  -proto sentiric/tts/v1/gateway.proto \
  -d '{"voice_id": "coqui:default", "text": "Coqui motoru da mTLS ile korunmaktadır."}' \
  tts-gateway-service:14011 \
  sentiric.tts.v1.TtsGatewayService/SynthesizeStream
```

---
