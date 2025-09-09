# ⚡ Sentiric TTS Gateway Service - Görev Listesi

Bu belge, `tts-gateway-service`'in geliştirme yol haritasını ve önceliklerini tanımlar.


---
*   **Görev ID:** `TTS-GW-BUG-01`
    *   **Başlık:** fix(routing): Gelen `SynthesizeRequest` içindeki `voice_selector` alanını doğru kullan
    *   **Durum:** `[ ✅ ] Tamamlandı`
    *   **Öncelik:** **KRİTİK**
    *   **Gerekçe:** Platformun dinamik ses seçme yeteneği çalışmıyordu. `dialplan`'dan `EmelNeural` sesi istenmesine rağmen, gateway varsayılan olarak `AhmetNeural` sesini kullanıyordu. `proxy_to_edge` fonksiyonu, gelen isteğin `voice_selector` alanını okumalı ve bunu `EdgeTtsRequest` payload'ına doğru şekilde aktarmalıydı.
    *   **Kabul Kriterleri:**
        1.  `dialplan`'da `tr-TR-EmelNeural` sesi tanımlandığında, arama sırasında duyulan sesin kadın sesi olduğu doğrulanmalıdır.
        2.  `tts-edge-service` logları, `voice` parametresi olarak `tr-TR-EmelNeural` aldığını göstermelidir.
    *   **Çözüm Notu:** `proxy_to_edge` fonksiyonundaki mantık güncellendi. Artık gelen `voice_selector` alanı dolu ise öncelikli olarak o kullanılıyor. Eğer alan boş veya tanımsız ise, sistem bir uyarı log'u basarak varsayılan ses olan daha kaliteli `tr-TR-EmelNeural` kadın sesine yönleniyor.

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

### **FAZ 2: Optimizasyon ve Dayanıklılık**
-   [ ] **Görev ID: TTS-GW-004 - Redis Önbellekleme**
    -   **Açıklama:** Sık sentezlenen cümleler için Redis tabanlı bir önbellekleme mekanizması ekle.
    -   **Kabul Kriterleri:**
        -   [ ] Bir `Synthesize` isteği geldiğinde, önce Redis'te bu metin için bir anahtar olup olmadığı kontrol edilmeli.
        -   [ ] Anahtar varsa, ses verisi doğrudan Redis'ten dönülmeli ve loglarda "CACHE HIT" mesajı görünmeli.
        -   [ ] Anahtar yoksa, uzman motordan gelen ses verisi Redis'e kaydedilmeli ve loglarda "CACHE MISS" mesajı görünmeli.

-   [ ] **Görev ID: TTS-GW-005 - Uzman Motor Fallback Mantığı**
    -   **Açıklama:** Birincil uzman motor (örn: `coqui-tts`) hata verdiğinde veya zaman aşımına uğradığında, isteği otomatik olarak ikincil motora (`edge-tts`) yönlendir.
    -   **Durum:** ⬜ Planlandı.

-   [ ] **Görev ID: TTS-GW-006 - Temel SSML Desteği**
    -   **Açıklama:** Gelen metindeki `<break time="500ms"/>` gibi etiketleri anla ve sentezlenmiş ses parçaları arasına uygun sürede sessizlik ekle.
    -   **Durum:** ⬜ Planlandı.