# ⚡ TTS Gateway Service - Görev Listesi

Bu liste, bu repoyu devralacak geliştirici için öncelikli işleri sıralar.

## 🔴 Faz 1: İskelet ve Bağlantılar
- [ ] **Protobuf Entegrasyonu:** `sentiric-contracts` reposunu `Cargo.toml`'a git bağımlılığı olarak ekle ve `build.rs` ile derle.
- [ ] **Upstream Client (Edge):** `reqwest` kullanarak en basit motor olan `tts-coqui-service`'e HTTP POST isteği atan ve dönen stream'i yakalayan bir istemci yaz.
- [ ] **gRPC Server:** `tonic` kullanarak `Synthesize` metodunu implemente et. Gelen isteği alıp Edge Client'a ilet.

## 🟡 Faz 2: Akıllı Yönlendirme ve Coqui
- [ ] **Routing Logic:** `LOGIC.md`'deki tabloya göre `voice_selector` parse eden bir `Router` struct'ı yaz.
- [ ] **Upstream Client (Coqui):** Coqui servisine istek atan istemciyi yaz.
- [ ] **Fallback Mekanizması:** Eğer Coqui hata dönerse otomatik olarak Edge client'ı çağıran `retry` mantığını ekle.

## 🟢 Faz 3: Performans ve Caching
- [ ] **Redis Cache:** Gelen metnin hash'ini alıp Redis'te var mı diye sor. Varsa direkt sesi dön.
- [ ] **Concurrency:** `Tokio` task'leri ile her isteği non-blocking olarak işle.

## 🔵 Faz 4: Güvenlik
- [ ] **mTLS:** `tonic` TLS konfigürasyonunu `.env`'den gelen sertifikalarla aktif et.