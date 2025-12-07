# 🛠️ Kurulum Rehberi

## 1. Ön Gereksinimler

*   **Docker & Docker Compose** (Konteynerli çalışma için)
*   **Rust & Cargo** (Yerel geliştirme için)
*   **Protobuf Compiler** (`protoc`)

### Protobuf Kurulumu (Linux)
```bash
sudo apt update
sudo apt install -y protobuf-compiler libprotobuf-dev
```

## 2. Yerel Geliştirme Ortamı

Projeyi klonladıktan sonra:

```bash
# 1. Config dosyasını oluştur
cp .env.example .env

# 2. Redis ve Mock servisleri ayağa kaldır
docker compose -f docker-compose.dev.yml up -d redis mock-tts-engine

# 3. Servisi çalıştır
cargo run
```

## 3. Test Etme

Servis çalışırken başka bir terminalde:

```bash
# Sağlık kontrolü
curl http://localhost:14010/healthz

# gRPC Testi (grpcurl gerektirir)
grpcurl -plaintext -d '{"text": "Merhaba dünya"}' localhost:14011 sentiric.tts.v1.TextToSpeechService/Synthesize
```