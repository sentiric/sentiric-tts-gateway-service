use crate::config::AppConfig;
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use crate::grpc::server::TtsGateway;
use crate::tls::load_server_tls_config;
use crate::metrics::start_metrics_server; // EKLENDİ
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayServiceServer;
use tonic::transport::Server;
use std::net::SocketAddr;
use tracing::{info, warn};
use anyhow::Result;
use std::sync::Arc;

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let config = Arc::new(AppConfig::load()?);

        tracing_subscriber::fmt()
            .with_env_filter(&config.rust_log)
            .init();

        info!("🚀 TTS Gateway Service v{} starting...", config.service_version);

        let coqui_client = CoquiClient::connect(&config).await?;
        let mms_client = MmsClient::connect(&config).await?;

        // METRICS & HEALTH SERVER
        let metrics_addr: SocketAddr = format!("{}:{}", config.host, config.http_port).parse()?;
        start_metrics_server(metrics_addr, coqui_client.clone(), mms_client.clone());

        // GRPC SERVER
        let addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port).parse()?;
        let gateway_service = TtsGateway::new(coqui_client, mms_client);
        
        let mut builder = Server::builder();

        if !config.tts_gateway_service_cert_path.is_empty() 
            && !config.tts_gateway_service_key_path.is_empty() 
            && !config.grpc_tls_ca_path.is_empty() 
        {
            match load_server_tls_config(&config).await {
                Ok(tls_config) => {
                    builder = builder.tls_config(tls_config)?;
                    info!("🎧 gRPC Server listening on {} (mTLS Enabled)", addr);
                },
                Err(e) => {
                    warn!("⚠️ Failed to load TLS config: {}. Falling back to INSECURE.", e);
                    info!("🎧 gRPC Server listening on {} (INSECURE)", addr);
                }
            }
        } else {
            warn!("⚠️  TLS Certificate paths are empty. Starting gRPC server in INSECURE mode.");
            info!("🎧 gRPC Server listening on {} (INSECURE)", addr);
        }

        builder
            .add_service(TtsGatewayServiceServer::new(gateway_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}