use config::{Config, File, Environment};
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    
    pub host: String,
    pub grpc_port: u16,

    // Upstream Services
    pub tts_coqui_service_url: String, 
    pub tts_mms_service_url: String,

    // Security
    pub grpc_tls_ca_path: String,
    pub tts_gateway_service_cert_path: String,
    pub tts_gateway_service_key_path: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name(".env").required(false))
            // Environment değişkenlerini separator ile otomatik eşleştir
            .add_source(Environment::default().separator("__"))
            
            // Manuel Override (Docker Compose uyumluluğu için)
            .set_override_option("tts_coqui_service_url", std::env::var("TTS_COQUI_SERVICE_URL").ok())?
            .set_override_option("tts_mms_service_url", std::env::var("TTS_MMS_SERVICE_URL").ok())?
            
            // Varsayılan Değerler
            .set_default("env", "development")?
            .set_default("rust_log", "info")?
            .set_default("service_version", "1.2.0")?
            .set_default("host", "0.0.0.0")?
            .set_default("grpc_port", 14011)?
            
            // Fallback URL'ler
            .set_default("tts_coqui_service_url", "https://tts-coqui-service:14031")?
            .set_default("tts_mms_service_url", "https://tts-mms-service:14061")?;

        builder.build()?.try_deserialize().map_err(|e| e.into())
    }
}