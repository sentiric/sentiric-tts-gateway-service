use crate::config::AppConfig;
use crate::tls::load_client_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_mms_service_client::TtsMmsServiceClient;
use sentiric_contracts::sentiric::tts::v1::{MmsSynthesizeStreamRequest, MmsSynthesizeStreamResponse};
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct MmsClient {
    client: TtsMmsServiceClient<Channel>,
}

impl MmsClient {
    pub async fn connect(config: &Arc<AppConfig>) -> anyhow::Result<Self> {
        let url = config.tts_mms_url.clone();
        info!("Connecting to MMS Service at: {}", url);
        let tls_config = load_client_tls_config(config).await?;
        let channel = Endpoint::from_shared(url)?.tls_config(tls_config)?.connect().await?;
        Ok(Self { client: TtsMmsServiceClient::new(channel) })
    }

    pub async fn synthesize_stream(
        &self,
        request: MmsSynthesizeStreamRequest,
    ) -> Result<tonic::Streaming<MmsSynthesizeStreamResponse>, tonic::Status> {
        let mut client = self.client.clone();
        let response = client.mms_synthesize_stream(Request::new(request)).await?;
        Ok(response.into_inner())
    }
}