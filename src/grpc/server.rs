use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayService;
use sentiric_contracts::sentiric::tts::v1::{
    SynthesizeStreamRequest, SynthesizeStreamResponse, 
    CoquiSynthesizeStreamRequest, MmsSynthesizeStreamRequest,
    ListVoicesRequest, ListVoicesResponse
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
    
    async fn list_voices(&self, _request: Request<ListVoicesRequest>) -> Result<Response<ListVoicesResponse>, Status> {
         Err(Status::unimplemented("ListVoices not implemented yet"))
    }

    #[instrument(skip(self, request))]
    async fn synthesize_stream(
        &self,
        request: Request<SynthesizeStreamRequest>,
    ) -> Result<Response<Self::SynthesizeStreamStream>, Status> {
        
        // 1. Metadata / Trace ID Extraction
        let trace_id = request.metadata()
            .get("x-trace-id")
            .and_then(|m| m.to_str().ok())
            .map(|s| s.to_string());

        let req = request.into_inner();
        let voice_selector = req.voice_id.clone(); 
        
        info!(
            "TTS Request received. Selector: {}, TraceID: {}", 
            voice_selector, 
            trace_id.as_deref().unwrap_or("none")
        );

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // 2. Routing Logic
        if voice_selector.starts_with("mms:") {
            // --- MMS ROUTE ---
            let lang_code = if req.voice_id.contains(':') {
                req.voice_id.split(':').nth(1).unwrap_or("tur").to_string()
            } else {
                "tur".to_string()
            };

            let mms_req = MmsSynthesizeStreamRequest {
                text: req.text,
                language_code: lang_code,
                speed: req.prosody.map(|p| p.rate as f32).unwrap_or(1.0),
                sample_rate: 16000, // MMS Default
            };

            // Hata yakalama eklendi
            match self.mms_client.synthesize_stream(mms_req, trace_id.clone()).await {
                Ok(mut stream) => {
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
                                Err(e) => { 
                                    error!("Upstream (MMS) stream error: {}", e);
                                    let _ = tx.send(Err(e)).await; 
                                    break; 
                                }
                            }
                        }
                    });
                },
                Err(e) => {
                    return Err(Status::unavailable(format!("MMS Service Unavailable: {}", e)));
                }
            }

        } else {
            // --- COQUI ROUTE (Default) ---
            let _speaker = if voice_selector.starts_with("coqui:") {
                voice_selector.strip_prefix("coqui:").unwrap_or("").to_string()
            } else {
                voice_selector
            };

            let coqui_req = CoquiSynthesizeStreamRequest {
                text: req.text,
                language_code: "tr".to_string(),
                speaker_wav: vec![], 
                speed: req.prosody.as_ref().map(|p| p.rate as f32).unwrap_or(1.0),
                temperature: 0.7,
                top_p: 0.9,
                top_k: 50.0,
                repetition_penalty: 1.1,
                output_format: "pcm".to_string(),
            };

            // Hata yakalama eklendi
            match self.coqui_client.synthesize_stream(coqui_req, trace_id.clone()).await {
                Ok(mut stream) => {
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
                                Err(e) => { 
                                    error!("Upstream (Coqui) stream error: {}", e);
                                    let _ = tx.send(Err(e)).await; 
                                    break; 
                                }
                            }
                        }
                    });
                },
                Err(e) => {
                    return Err(Status::unavailable(format!("Coqui Service Unavailable: {}", e)));
                }
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}