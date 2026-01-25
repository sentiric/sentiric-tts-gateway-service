use config::{Config, File, Environment};
use serde::Deserialize;
use anyhow::Result;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[allow(dead_code)]
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    
    // Ağ Ayarları
    pub host: String,
    pub http_port: u16,    // EKLENDİ
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
            .add_source(Environment::default().separator("__"))
            
            // MANUAL OVERRIDES
            .set_override_option("host", env::var("TTS_GATEWAY_SERVICE_LISTEN_ADDRESS").ok())?
            .set_override_option("http_port", env::var("TTS_GATEWAY_SERVICE_HTTP_PORT").ok())? // EKLENDİ
            .set_override_option("grpc_port", env::var("TTS_GATEWAY_SERVICE_GRPC_PORT").ok())?
            
            .set_override_option("tts_coqui_service_url", env::var("TTS_COQUI_SERVICE_URL").ok())?
            .set_override_option("tts_mms_service_url", env::var("TTS_MMS_SERVICE_URL").ok())?
            
            // DEFAULTS
            .set_default("env", "production")?
            .set_default("rust_log", "info,sentiric_tts_gateway=debug")?
            .set_default("service_version", "1.2.2")?
            
            .set_default("host", "0.0.0.0")?
            .set_default("http_port", 14010)? // EKLENDİ
            .set_default("grpc_port", 14011)?
            
            .set_default("tts_coqui_service_url", "https://tts-coqui-service:14031")?
            .set_default("tts_mms_service_url", "https://tts-mms-service:14061")?

            .set_default("grpc_tls_ca_path", "/sentiric-certificates/certs/ca.crt")?
            .set_default("tts_gateway_service_cert_path", "/sentiric-certificates/certs/tts-gateway-service.crt")?
            .set_default("tts_gateway_service_key_path", "/sentiric-certificates/certs/tts-gateway-service.key")?;

        builder.build()?.try_deserialize().map_err(|e| e.into())
    }
}