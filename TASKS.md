# ⚡ Sentiric TTS Gateway Service - Görev Listesi

Bu belge, `tts-gateway-service`'in geliştirme yol haritasını ve önceliklerini tanımlar.

---
### **FAZ 1: Temel Yönlendirme ve Proxy (Mevcut Odak)**

-   **Görev ID: TTS-GW-BUG-02 - fix(network): Uzman Motorlara Yönlendirmede URL Ayrıştırma Hatasını Gider**
    -   **Durum:** ⬜ **Yapılacak (Öncelik 1 - KRİTİK)**
    -   **Bağımlılık:** `sentiric-infrastructure`'daki `INFRA-STAB-02`.
    -   **Açıklama:** Canlı test logları, servisin `tts-edge-service`'e istek atarken `builder error for url` hatası aldığını göstermektedir. Bunun kök nedeni, `.env`'den gelen servis adresinin `http://` şemasını içermemesidir. Bu görev, hem bu hatayı gidermeyi hem de gelecekteki benzer sorunları önlemeyi hedefler.
    -   **Kabul Kriterleri:**
        -   [ ] `main.rs` içindeki `MyTtsGatewayService`'in oluşturulduğu yerde, ortam değişkenlerinden okunan URL'lerin geçerli bir şemaya sahip olup olmadığı kontrol edilmeli ve gerekirse `http://` otomatik olarak eklenmelidir.
        -   [ ] `sentiric-config` reposundaki `service.env` dosyalarında URL'lerin `http://` şemasıyla tanımlanması sağlanmalıdır.
        -   [ ] Düzeltme sonrası, `agent-service`'ten gelen bir isteğin `tts-edge-service`'e başarıyla ulaştığı loglardan doğrulanmalıdır.

-   **Görev ID: TTS-GW-BUG-01 - fix(routing): `voice_selector` Alanını Doğru Kullan**
    -   **Durum:** ✅ **Tamamlandı**
    -   **Çözüm Notu:** `proxy_to_edge` fonksiyonu, gelen `voice_selector` alanını doğru bir şekilde işleyecek ve boş olduğunda loglama yaparak güvenli bir varsayılana yönelecek şekilde güncellenmiştir.

---

### **FAZ 2: Optimizasyon ve Dayanıklılık (Planlandı)**
-   [ ] **Görev ID: TTS-GW-004 - Redis Önbellekleme**
-   [ ] **Görev ID: TTS-GW-005 - Uzman Motor Fallback Mantığı**
-   [ ] **Görev ID: TTS-GW-006 - Temel SSML Desteği**