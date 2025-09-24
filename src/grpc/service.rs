// sentiric-tts-gateway-service/src/grpc/service.rs
use crate::services::tts_proxy::TtsProxyService;
use sentiric_contracts::sentiric::tts::v1::{
    text_to_speech_service_server::TextToSpeechService, SynthesizeRequest, SynthesizeResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, instrument};

pub struct MyTtsGatewayService {
    proxy_service: Arc<TtsProxyService>,
}

impl MyTtsGatewayService {
    pub fn new(proxy_service: Arc<TtsProxyService>) -> Self {
        Self { proxy_service }
    }
}

#[tonic::async_trait]
impl TextToSpeechService for MyTtsGatewayService {
    #[instrument(skip(self, request), fields(
        text = %request.get_ref().text,
        lang = %request.get_ref().language_code,
        has_speaker_url = request.get_ref().speaker_wav_url.is_some(),
        voice_selector = ?request.get_ref().voice_selector,
    ))]
    async fn synthesize(
        &self,
        request: Request<SynthesizeRequest>,
    ) -> Result<Response<SynthesizeResponse>, Status> {
        let req = request.into_inner();
        let use_cloning = req.speaker_wav_url.is_some();

        let result = if use_cloning {
            info!("Ses klonlama isteği, Coqui-TTS motoruna yönlendiriliyor.");
            self.proxy_service.proxy_to_coqui(req).await
        } else {
            info!("Standart sentezleme isteği, Edge-TTS motoruna yönlendiriliyor.");
            self.proxy_service.proxy_to_edge(req).await
        };

        result.map(Response::new).map_err(Status::from)
    }
}