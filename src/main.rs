// sentiric-tts-gateway-service/src/main.rs
use anyhow::Result;
use sentiric_tts_gateway_service::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Uygulamanın başlatılması ve çalıştırılması sorumluluğu
    // tamamen 'app' modülüne devredildi.
    App::bootstrap().await?.run().await
}