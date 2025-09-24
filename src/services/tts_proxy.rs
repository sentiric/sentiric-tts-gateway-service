// sentiric-tts-gateway-service/src/services/tts_proxy.rs
use crate::config::AppConfig;
use crate::error::GatewayError;
use reqwest::Client;
use sentiric_contracts::sentiric::tts::v1::{SynthesizeRequest, SynthesizeResponse};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

#[derive(Serialize)]
struct EdgeTtsRequest<'a> {
    text: &'a str,
    voice: &'a str,
}

#[derive(Serialize)]
struct CoquiTtsRequest<'a> {
    text: &'a str,
    language: &'a str,
    speaker_wav_url: Option<&'a str>,
}

pub struct TtsProxyService {
    http_client: Client,
    config: Arc<AppConfig>,
}

impl TtsProxyService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { http_client: Client::new(), config }
    }

    pub async fn proxy_to_coqui(&self, req: SynthesizeRequest) -> Result<SynthesizeResponse, GatewayError> {
        let target_url = self.config.tts_coqui_service_url.as_ref()
            .ok_or(GatewayError::CloningNotConfigured)?;
        
        let payload = CoquiTtsRequest {
            text: &req.text,
            language: &req.language_code,
            speaker_wav_url: req.speaker_wav_url.as_deref(),
        };
        
        let res = self.http_client.post(target_url).json(&payload).send().await
            .map_err(|e| GatewayError::UpstreamUnavailable(format!("Coqui servisine ulaşılamıyor: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(GatewayError::UpstreamError(format!("Coqui servisi hata döndürdü ({}): {}", status, err_body)));
        }
        
        let audio_bytes = res.bytes().await.map_err(|e| GatewayError::UpstreamReadError(e.to_string()))?;
        info!("Uzman motordan (Coqui-TTS) ses başarıyla alındı.");
        
        Ok(SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-coqui-service".to_string(),
        })
    }

    pub async fn proxy_to_edge(&self, req: SynthesizeRequest) -> Result<SynthesizeResponse, GatewayError> {
        const DEFAULT_VOICE: &str = "tr-TR-EmelNeural";
        let voice = req.voice_selector.as_deref().filter(|v| !v.trim().is_empty()).unwrap_or(DEFAULT_VOICE);
        
        let payload = EdgeTtsRequest { text: &req.text, voice };

        let res = self.http_client.post(&self.config.tts_edge_service_url).json(&payload).send().await
            .map_err(|e| GatewayError::UpstreamUnavailable(format!("Edge servisine ulaşılamıyor: {}", e)))?;
        
        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_default();
            return Err(GatewayError::UpstreamError(format!("Edge servisi hata döndürdü ({}): {}", status, err_body)));
        }

        let audio_bytes = res.bytes().await.map_err(|e| GatewayError::UpstreamReadError(e.to_string()))?;
        info!("Uzman motordan (Edge-TTS) ses başarıyla alındı.");

        Ok(SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-edge-service".to_string(),
        })
    }
}