use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayService;
use sentiric_contracts::sentiric::tts::v1::{
    SynthesizeStreamRequest, SynthesizeStreamResponse, 
    CoquiSynthesizeStreamRequest, MmsSynthesizeStreamRequest
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{info, error, instrument};
use futures::StreamExt;

pub struct TtsGateway {
    coqui_client: CoquiClient,
    mms_client: MmsClient,
}

impl TtsGateway {
    pub fn new(coqui_client: CoquiClient, mms_client: MmsClient) -> Self {
        Self { coqui_client, mms_client }
    }
}

#[tonic::async_trait]
impl TtsGatewayService for TtsGateway {
    type SynthesizeStreamStream = ReceiverStream<Result<SynthesizeStreamResponse, Status>>;

    async fn synthesize(
        &self,
        _request: Request<sentiric_contracts::sentiric::tts::v1::SynthesizeRequest>,
    ) -> Result<Response<sentiric_contracts::sentiric::tts::v1::SynthesizeResponse>, Status> {
         Err(Status::unimplemented("Use SynthesizeStream for better performance"))
    }
    
    async fn list_voices(&self, _request: Request<sentiric_contracts::sentiric::tts::v1::ListVoicesRequest>) -> Result<Response<sentiric_contracts::sentiric::tts::v1::ListVoicesResponse>, Status> {
         Err(Status::unimplemented("ListVoices not implemented yet"))
    }

    #[instrument(skip(self, request))]
    async fn synthesize_stream(
        &self,
        request: Request<SynthesizeStreamRequest>,
    ) -> Result<Response<Self::SynthesizeStreamStream>, Status> {
        let req = request.into_inner();
        let voice_selector = req.voice_id; // "coqui:...", "mms:..."
        
        // Hangi provider kullanıldı bilgisini response'a eklemek için
        let mut provider_used = "unknown".to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // --- ROUTING ---
        if voice_selector.starts_with("mms:") {
            provider_used = "mms".to_string();
            info!("Routing to MMS: {}", voice_selector);
            
            let mms_req = MmsSynthesizeStreamRequest {
                text: req.text,
                language_code: "tur".to_string(), // MMS dil kodu
                speed: req.prosody.map(|p| p.rate as f32).unwrap_or(1.0),
                sample_rate: 16000,
            };

            let mut stream = self.mms_client.synthesize_stream(mms_req).await
                .map_err(|e| Status::unavailable(format!("MMS Error: {}", e)))?;
                
            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    match res {
                        Ok(chunk) => {
                            let resp = SynthesizeStreamResponse {
                                audio_content: chunk.audio_chunk,
                                content_type: "audio/pcm".to_string(),
                                provider_used: "mms".to_string(),
                            };
                            if tx.send(Ok(resp)).await.is_err() { break; }
                        }
                        Err(e) => { let _ = tx.send(Err(e)).await; break; }
                    }
                }
            });

        } else {
            // Default: COQUI
            provider_used = "coqui".to_string();
            info!("Routing to Coqui: {}", voice_selector);
            
            let coqui_req = CoquiSynthesizeStreamRequest {
                text: req.text,
                language_code: "tr".to_string(),
                speaker_wav: vec![], // Coqui servisi default'u kullanacak
                speed: req.prosody.map(|p| p.rate as f32).unwrap_or(1.0),
                temperature: 0.7,
                top_p: 0.9,
                top_k: 50.0,
                repetition_penalty: 1.1,
                output_format: "pcm".to_string(),
            };

            let mut stream = self.coqui_client.synthesize_stream(coqui_req).await
                .map_err(|e| Status::unavailable(format!("Coqui Error: {}", e)))?;

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    match res {
                        Ok(chunk) => {
                            let resp = SynthesizeStreamResponse {
                                audio_content: chunk.audio_chunk,
                                content_type: "audio/pcm".to_string(),
                                provider_used: "coqui".to_string(),
                            };
                            if tx.send(Ok(resp)).await.is_err() { break; }
                        }
                        Err(e) => { let _ = tx.send(Err(e)).await; break; }
                    }
                }
            });
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}