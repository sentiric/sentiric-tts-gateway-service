# 📋 Teknik Şartname

## 1. Servis Kimliği
*   **Adı:** `sentiric-tts-gateway-service`
*   **Dil:** Rust (Edition 2021)
*   **Framework:** Tonic (gRPC), Axum (HTTP), Tokio (Runtime)

## 2. Portlar ve Protokoller
| Port | Protokol | Amaç |
|---|---|---|
| `14010` | HTTP | Sağlık kontrolü (`/healthz`) ve Metrikler (`/metrics`) |
| `14011` | gRPC | Ana servis iletişimi (`Synthesize`) |
| `14012` | HTTP | (Opsiyonel) Debugging |

## 3. Bağımlılıklar
*   **Redis:** Önbellekleme için zorunlu.
*   **Sentiric Contracts:** `.proto` dosyaları için git bağımlılığı.
*   **mTLS Sertifikaları:** Üretim ortamında zorunlu.

## 4. Hata Yönetimi
*   **Upstream Unavailable:** Uzman motor kapalıysa `UNAVAILABLE` (gRPC 14) döner.
*   **Invalid Argument:** Parametreler hatalıysa `INVALID_ARGUMENT` (gRPC 3) döner.