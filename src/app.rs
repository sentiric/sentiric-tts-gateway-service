use crate::config::AppConfig;
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use crate::grpc::server::TtsGateway;
use crate::tls::load_server_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayServiceServer;
use tonic::transport::Server;
use std::net::SocketAddr;
use tracing::{info, warn}; // warn eklendi
use anyhow::Result;
use std::sync::Arc;

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        // 1. Config
        let config = Arc::new(AppConfig::load()?);

        // 2. Logging
        tracing_subscriber::fmt()
            .with_env_filter(&config.rust_log)
            .init();

        info!("🚀 TTS Gateway Service v{} starting...", config.service_version);

        // 3. Connect Upstreams
        let coqui_client = CoquiClient::connect(&config).await?;
        let mms_client = MmsClient::connect(&config).await?;

        // 4. Setup Server Logic
        let addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port).parse()?;
        let gateway_service = TtsGateway::new(coqui_client, mms_client);
        
        // 5. Configure Server (TLS vs Insecure)
        let mut builder = Server::builder();

        // [FIX] Sertifika yolları doluysa TLS yükle, boşsa Insecure başlat
        if !config.tts_gateway_service_cert_path.is_empty() 
            && !config.tts_gateway_service_key_path.is_empty() 
            && !config.grpc_tls_ca_path.is_empty() 
        {
            info!("🔐 Loading TLS configuration...");
            // Dosyaların varlığını kontrol etmeden yüklemeye çalışma
            match load_server_tls_config(&config).await {
                Ok(tls_config) => {
                    builder = builder.tls_config(tls_config)?;
                    info!("🎧 gRPC Server listening on {} (mTLS Enabled)", addr);
                },
                Err(e) => {
                    // Eğer sertifika yolu var ama dosya bozuksa/yoksa hata verip çıkmalıyız
                    return Err(anyhow::anyhow!("Failed to load TLS config: {}", e));
                }
            }
        } else {
            warn!("⚠️  TLS Certificate paths are empty. Starting gRPC server in INSECURE mode.");
            info!("🎧 gRPC Server listening on {} (INSECURE)", addr);
        }

        // 6. Start Serving
        builder
            .add_service(TtsGatewayServiceServer::new(gateway_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}