use crate::config::AppConfig;
use crate::tls::load_client_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_coqui_service_client::TtsCoquiServiceClient;
use sentiric_contracts::sentiric::tts::v1::{CoquiSynthesizeStreamRequest, CoquiSynthesizeStreamResponse};
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct CoquiClient {
    client: TtsCoquiServiceClient<Channel>,
}

impl CoquiClient {
    pub async fn connect(config: &Arc<AppConfig>) -> anyhow::Result<Self> {
        let url = config.tts_coqui_url.clone();
        info!("Connecting to Coqui Service at: {}", url);
        let tls_config = load_client_tls_config(config).await?;
        let channel = Endpoint::from_shared(url)?.tls_config(tls_config)?.connect().await?;
        Ok(Self { client: TtsCoquiServiceClient::new(channel) })
    }

    pub async fn synthesize_stream(
        &self,
        request: CoquiSynthesizeStreamRequest,
    ) -> Result<tonic::Streaming<CoquiSynthesizeStreamResponse>, tonic::Status> {
        let mut client = self.client.clone();
        let response = client.coqui_synthesize_stream(Request::new(request)).await?;
        Ok(response.into_inner())
    }
}