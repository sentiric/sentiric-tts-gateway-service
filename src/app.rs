// Dosya: src/app.rs
use crate::config::AppConfig;
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use crate::grpc::server::TtsGateway;
use crate::tls::load_server_tls_config;
use crate::metrics::start_metrics_server; 
use crate::logger::SutsV4Formatter; // [YENİ]
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayServiceServer;
use tonic::transport::Server;
use std::net::SocketAddr;
use tracing::{info, error};
use anyhow::Result;
use std::sync::Arc;

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let config = Arc::new(AppConfig::load()?);

        // [ARCH-COMPLIANCE] Özel SUTS v4.0 Formatter ayağa kaldırılır
        let formatter = SutsV4Formatter {
            service_name: "tts-gateway-service".to_string(),
            service_version: config.service_version.clone(),
            service_env: config.env.clone(),
        };

        tracing_subscriber::fmt()
            .with_env_filter(&config.rust_log)
            .event_format(formatter)
            .init();

        info!(
            event = "SERVICE_START",
            schema_v = "1.0.0",
            service_version = %config.service_version,
            "🚀 TTS Gateway Service starting..."
        );

        let coqui_client = CoquiClient::connect(&config).await?;
        let mms_client = MmsClient::connect(&config).await?;

        let metrics_addr: SocketAddr = format!("{}:{}", config.host, config.http_port).parse()?;
        start_metrics_server(metrics_addr, coqui_client.clone(), mms_client.clone());

        let addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port).parse()?;
        let gateway_service = TtsGateway::new(coqui_client, mms_client);
        
        let mut builder = Server::builder();

        if config.tts_gateway_service_cert_path.is_empty() 
            || config.tts_gateway_service_key_path.is_empty() 
            || config.grpc_tls_ca_path.is_empty() 
        {
            panic!("Architectural Violation: mTLS certificates are strictly required. INSECURE mode is forbidden.");
        }

        let tls_config = load_server_tls_config(&config).await.expect("Architectural Violation: Failed to load required TLS Configuration");
        builder = builder.tls_config(tls_config)?;
        
        info!(
            event = "GRPC_SERVER_READY",
            address = %addr,
            "🎧 gRPC Server listening (mTLS Enabled)"
        );

        if let Err(e) = builder
            .add_service(TtsGatewayServiceServer::new(gateway_service))
            .serve(addr)
            .await 
        {
            error!(event = "GRPC_SERVER_CRASH", error = %e, "gRPC Server stopped unexpectedly.");
            return Err(e.into());
        }

        Ok(())
    }
}