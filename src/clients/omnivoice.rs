// Dosya: src/clients/omnivoice.rs
use crate::config::AppConfig;
use crate::tls::load_client_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_omnivoice_service_client::TtsOmnivoiceServiceClient;
use sentiric_contracts::sentiric::tts::v1::{
    OmnivoiceSynthesizeStreamRequest, OmnivoiceSynthesizeStreamResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use tracing::{error, info};

#[derive(Clone)]
pub struct OmnivoiceClient {
    client: TtsOmnivoiceServiceClient<Channel>,
}

impl OmnivoiceClient {
    pub async fn connect(config: &Arc<AppConfig>) -> anyhow::Result<Self> {
        let url = config.tts_omnivoice_service_url.clone();
        if url.starts_with("http://") {
            panic!("[ARCH-COMPLIANCE] Insecure HTTP channels are forbidden. Use mTLS.");
        }
        info!(event = "UPSTREAM_CONNECTING", url = %url, engine = "omnivoice", "OmniVoice Service istemcisi yapılandırılıyor...");
        let tls_config = load_client_tls_config(config).await?;
        let channel = Endpoint::from_shared(url)?
            .tls_config(tls_config)?
            .connect_lazy();
        Ok(Self {
            client: TtsOmnivoiceServiceClient::new(channel),
        })
    }

    pub async fn synthesize_stream(
        &self,
        request: OmnivoiceSynthesizeStreamRequest,
        trace_id: Option<String>,
        span_id: Option<String>,
        tenant_id: Option<String>,
    ) -> Result<tonic::Streaming<OmnivoiceSynthesizeStreamResponse>, tonic::Status> {
        let mut client = self.client.clone();
        let mut req = Request::new(request);

        if let Some(tid) = trace_id.as_ref() {
            if let Ok(meta_val) = MetadataValue::from_str(tid) {
                req.metadata_mut().insert("x-trace-id", meta_val);
            }
        }
        if let Some(sid) = span_id.as_ref() {
            if let Ok(meta_val) = MetadataValue::from_str(sid) {
                req.metadata_mut().insert("x-span-id", meta_val);
            }
        }
        if let Some(ten) = tenant_id.as_ref() {
            if let Ok(meta_val) = MetadataValue::from_str(ten) {
                req.metadata_mut().insert("x-tenant-id", meta_val);
            }
        }

        match client.omnivoice_synthesize_stream(req).await {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => {
                error!(event = "UPSTREAM_CALL_FAILED", engine = "omnivoice", error = %e, "OmniVoice gRPC bağlantısı başarısız.");
                Err(e)
            }
        }
    }
}
