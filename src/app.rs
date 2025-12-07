use crate::config::AppConfig;
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct App {
    config: AppConfig,
}

impl App {
    pub async fn bootstrap() -> Result<Self> {
        // 1. Env Yükle
        dotenvy::dotenv().ok();
        
        // 2. Config Yükle
        let config = AppConfig::load()?;

        // 3. Logger Başlat
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
            .with(fmt::layer())
            .init();

        info!("🚀 TTS Gateway Service v{} başlatılıyor...", config.service_version);
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        info!("Servisler ayağa kaldırılıyor (HTTP: {}, gRPC: {})...", 
              self.config.http_port, self.config.grpc_port);
        
        // Burada Tokio spawn ile gRPC ve HTTP sunucuları başlatılacak
        // Şimdilik sadece bekletiyoruz
        tokio::signal::ctrl_c().await?;
        info!("🛑 Kapatılıyor...");
        Ok(())
    }
}