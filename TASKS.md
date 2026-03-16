# Dosya: TASKS.md
# ⚡ TTS Gateway Service - Görev Listesi

Bu liste, bu repoyu devralacak geliştirici için öncelikli işleri sıralar.

- [x] **[ARCH-COMPLIANCE]** Uygulama logları Sentiric `constraints.yaml` kurallarına uyarak JSON formatına (.json()) zorunlu çevrildi.
- [x] **[ARCH-COMPLIANCE]** Tüm İstemci (MMS/Coqui) ve Sunucu bağlantılarında Insecure (http://) fallback yapısı kurala aykırı olduğu için silindi. mTLS mimarisi kesin zorunluluk haline getirildi.