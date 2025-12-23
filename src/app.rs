use crate::config::AppConfig;
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use crate::grpc::server::TtsGateway;
use crate::tls::load_server_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayServiceServer;
use tonic::transport::Server;
use std::net::SocketAddr;
use tracing::info;
use anyhow::Result;
use std::sync::Arc;

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let config = Arc::new(AppConfig::load()?);

        tracing_subscriber::fmt()
            .with_env_filter(&config.rust_log)
            .init();

        info!("🚀 TTS Gateway Service starting on {}:{}", config.host, config.grpc_port);

        let coqui_client = CoquiClient::connect(&config).await?;
        let mms_client = MmsClient::connect(&config).await?;

        let addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port).parse()?;
        let gateway_service = TtsGateway::new(coqui_client, mms_client);
        let tls_config = load_server_tls_config(&config).await?;

        Server::builder()
            .tls_config(tls_config)?
            .add_service(TtsGatewayServiceServer::new(gateway_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}