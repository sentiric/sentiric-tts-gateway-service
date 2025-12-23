use config::{Config, File, Environment};
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub env: String,
    pub rust_log: String,
    pub host: String,
    pub grpc_port: u16,

    // Upstream Services
    pub tts_coqui_url: String, // http://tts-coqui-service:14031
    pub tts_mms_url: String,   // http://tts-mms-service:14061

    // Security
    pub grpc_tls_ca_path: String,
    pub tts_gateway_service_cert_path: String,
    pub tts_gateway_service_key_path: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name(".env").required(false))
            .add_source(Environment::default())
            .set_default("env", "development")?
            .set_default("rust_log", "info")?
            .set_default("host", "0.0.0.0")?
            .set_default("grpc_port", 14011)?
            .set_default("tts_coqui_url", "http://tts-coqui-service:14031")?
            .set_default("tts_mms_url", "http://tts-mms-service:14061")?;

        builder.build()?.try_deserialize().map_err(|e| e.into())
    }
}