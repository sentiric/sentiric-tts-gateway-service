// Dosya: src/clients/coqui.rs
use crate::config::AppConfig;
use crate::tls::load_client_tls_config;
use sentiric_contracts::sentiric::tts::v1::tts_coqui_service_client::TtsCoquiServiceClient;
use sentiric_contracts::sentiric::tts::v1::{CoquiSynthesizeStreamRequest, CoquiSynthesizeStreamResponse};
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use std::sync::Arc;
use tracing::{info, error};
use tonic::metadata::MetadataValue;
use std::str::FromStr;

#[derive(Clone)]
pub struct CoquiClient {
    client: TtsCoquiServiceClient<Channel>,
}

impl CoquiClient {
    pub async fn connect(config: &Arc<AppConfig>) -> anyhow::Result<Self> {
        let url = config.tts_coqui_service_url.clone();
        
        if url.starts_with("http://") {
            panic!("Architectural Violation: Insecure HTTP channels are strictly forbidden for gRPC communication. Use https:// and mTLS.");
        }
        
        info!(
            event = "UPSTREAM_CONNECTING",
            url = %url,
            engine = "coqui",
            "Coqui Service istemcisi yapılandırılıyor..."
        );

        let tls_config = load_client_tls_config(config).await?;
        let channel = Endpoint::from_shared(url)?.tls_config(tls_config)?.connect_lazy();
            
        Ok(Self { client: TtsCoquiServiceClient::new(channel) })
    }

    pub async fn synthesize_stream(
        &self,
        request: CoquiSynthesizeStreamRequest,
        trace_id: Option<String>,
        span_id: Option<String>,
        tenant_id: Option<String>,
    ) -> Result<tonic::Streaming<CoquiSynthesizeStreamResponse>, tonic::Status> {
        let mut client = self.client.clone();
        let mut req = Request::new(request);

        // [ARCH-COMPLIANCE] Strict Context Propagation
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

        match client.coqui_synthesize_stream(req).await {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => {
                error!(
                    event = "UPSTREAM_CALL_FAILED",
                    trace_id = ?trace_id,
                    span_id = ?span_id,
                    tenant_id = ?tenant_id,
                    engine = "coqui",
                    error = %e,
                    "Coqui motoruna gRPC bağlantısı başarısız oldu."
                );
                Err(e)
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        true
    }
}