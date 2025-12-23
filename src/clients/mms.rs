use crate::config::AppConfig;
use crate::tls::load_client_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_mms_service_client::TtsMmsServiceClient;
use sentiric_contracts::sentiric::tts::v1::{MmsSynthesizeStreamRequest, MmsSynthesizeStreamResponse};
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use std::sync::Arc;
use tracing::{info, error, warn};
use tonic::metadata::MetadataValue;
use std::str::FromStr;

#[derive(Clone)]
pub struct MmsClient {
    client: TtsMmsServiceClient<Channel>,
}

impl MmsClient {
    pub async fn connect(config: &Arc<AppConfig>) -> anyhow::Result<Self> {
        let url = config.tts_mms_service_url.clone();
        info!("Configuring MMS Service Endpoint: {}", url);
        
        let tls_config = load_client_tls_config(config).await?;
        
        // KRİTİK: Lazy Connection
        let channel = Endpoint::from_shared(url)?
            .tls_config(tls_config)?
            .connect_lazy();
            
        Ok(Self { client: TtsMmsServiceClient::new(channel) })
    }

    pub async fn synthesize_stream(
        &self,
        request: MmsSynthesizeStreamRequest,
        trace_id: Option<String>,
    ) -> Result<tonic::Streaming<MmsSynthesizeStreamResponse>, tonic::Status> {
        let mut client = self.client.clone();
        let mut req = Request::new(request);

        if let Some(tid) = trace_id {
            if let Ok(meta_val) = MetadataValue::from_str(&tid) {
                req.metadata_mut().insert("x-trace-id", meta_val);
            }
        }

        match client.mms_synthesize_stream(req).await {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => {
                error!("MMS Engine gRPC connection failed: {}", e);
                Err(e)
            }
        }
    }
}