// sentiric-tts-gateway-service/src/config.rs
use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct AppConfig {
    pub grpc_listen_addr: SocketAddr,
    pub http_listen_addr: SocketAddr, // YENİ ALAN
    pub tts_edge_service_url: String,
    pub tts_coqui_service_url: Option<String>,
    pub env: String,
    pub rust_log: String,
    pub service_version: String,
    pub git_commit: String,
    pub build_date: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
}

impl AppConfig {
    pub fn load_from_env() -> Result<Self> {
        let grpc_port = env::var("TTS_GATEWAY_SERVICE_GRPC_PORT").unwrap_or_else(|_| "14011".to_string());
        let grpc_addr: SocketAddr = format!("[::]:{}", grpc_port).parse()?;

        // YENİ: HTTP portunu da env'den oku
        let http_port = env::var("TTS_GATEWAY_SERVICE_HTTP_PORT").unwrap_or_else(|_| "14010".to_string());
        let http_addr: SocketAddr = format!("[::]:{}", http_port).parse()?;
            
        let tts_edge_service_url = env::var("TTS_EDGE_SERVICE_TARGET_HTTP_URL")
            .context("ZORUNLU: TTS_EDGE_SERVICE_TARGET_HTTP_URL eksik")?;
        let tts_coqui_service_url = env::var("TTS_COQUI_SERVICE_TARGET_HTTP_URL")
            .ok()
            .filter(|url| !url.is_empty())
            .map(|url| format!("{}/api/v1/synthesize", url));
            
        Ok(AppConfig {
            grpc_listen_addr: grpc_addr,
            http_listen_addr: http_addr, // YENİ ATAMA
            tts_edge_service_url: format!("{}/api/v1/synthesize", tts_edge_service_url),
            tts_coqui_service_url,
            env: env::var("ENV").unwrap_or_else(|_| "production".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            service_version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            git_commit: env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
            build_date: env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string()),
            cert_path: env::var("TTS_GATEWAY_SERVICE_CERT_PATH").context("ZORUNLU: TTS_GATEWAY_SERVICE_CERT_PATH eksik")?,
            key_path: env::var("TTS_GATEWAY_SERVICE_KEY_PATH").context("ZORUNLU: TTS_GATEWAY_SERVICE_KEY_PATH eksik")?,
            ca_path: env::var("GRPC_TLS_CA_PATH").context("ZORUNLU: GRPC_TLS_CA_PATH eksik")?,
        })
    }
}