use config::{Config, File, Environment};
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub redis_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name(".env").required(false))
            .add_source(Environment::default().separator("__"))
            // Varsayılanlar
            .set_default("env", "development")?
            .set_default("rust_log", "info")?
            .set_default("service_version", "0.1.0")?
            .set_default("http_port", 14010)?
            .set_default("grpc_port", 14011)?;

        builder.build()?.try_deserialize().map_err(|e| e.into())
    }
}