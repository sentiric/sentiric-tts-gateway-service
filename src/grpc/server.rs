// sentiric-tts-gateway-service/src/grpc/server.rs

use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayService;
use sentiric_contracts::sentiric::tts::v1::{
    CoquiSynthesizeStreamRequest, ListVoicesRequest, ListVoicesResponse,
    MmsSynthesizeStreamRequest, SynthesizeStreamRequest, SynthesizeStreamResponse, TuningParams,
};
use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument, Span};

pub struct TtsGateway {
    coqui_client: CoquiClient,
    mms_client: MmsClient, // [DÜZELTME]: '_' öneki kaldırıldı, artık aktif.
}

impl TtsGateway {
    pub fn new(coqui_client: CoquiClient, mms_client: MmsClient) -> Self {
        Self {
            coqui_client,
            mms_client,
        }
    }
}

#[tonic::async_trait]
impl TtsGatewayService for TtsGateway {
    type SynthesizeStreamStream = ReceiverStream<Result<SynthesizeStreamResponse, Status>>;

    async fn synthesize(
        &self,
        _request: Request<sentiric_contracts::sentiric::tts::v1::SynthesizeRequest>,
    ) -> Result<Response<sentiric_contracts::sentiric::tts::v1::SynthesizeResponse>, Status> {
        Err(Status::unimplemented(
            "Unary Synthesize is deprecated. Use SynthesizeStream for low latency.",
        ))
    }

    async fn list_voices(
        &self,
        _request: Request<ListVoicesRequest>,
    ) -> Result<Response<ListVoicesResponse>, Status> {
        Err(Status::unimplemented("ListVoices not implemented yet"))
    }

    #[instrument(skip_all, fields(trace_id, voice_id, sample_rate))]
    async fn synthesize_stream(
        &self,
        request: Request<SynthesizeStreamRequest>,
    ) -> Result<Response<Self::SynthesizeStreamStream>, Status> {
        let trace_id = request
            .metadata()
            .get("x-trace-id")
            .and_then(|m| m.to_str().ok())
            .map(|s| s.to_string());
        Span::current().record("trace_id", &trace_id.as_deref().unwrap_or("unknown"));

        let req = request.into_inner();
        let voice_selector = req.voice_id.clone();
        Span::current().record("voice_id", &voice_selector.as_str());

        // AudioConfig zorunlu
        let audio_config = req
            .audio_config
            .ok_or_else(|| Status::invalid_argument("AudioConfig is mandatory."))?;
        let requested_sample_rate = audio_config.sample_rate_hertz;
        Span::current().record("sample_rate", requested_sample_rate);

        info!("TTS Gateway: İstek Yönlendiriliyor...");

        let (tx, rx) = tokio::sync::mpsc::channel(4096);

        // Tuning parametreleri (Varsayılanlar)
        let tuning = req.tuning.unwrap_or_else(|| TuningParams {
            temperature: 0.75, speed: 1.0, top_p: 0.85, top_k: 50, repetition_penalty: 2.0,
        });

        // --- YÖNLENDİRME MANTIĞI (ROUTING LOGIC) ---
        if voice_selector.starts_with("mms:") {
            // --- MMS MOTORU ---
            // "mms:tur" -> "tur"
            let lang_code = voice_selector.strip_prefix("mms:").unwrap_or("tur").to_string();
            
            let mms_req = MmsSynthesizeStreamRequest {
                text: req.text,
                language_code: lang_code,
                speed: tuning.speed,
                sample_rate: requested_sample_rate,
            };

            match self.mms_client.synthesize_stream(mms_req, trace_id.clone()).await {
                Ok(mut stream) => {
                    tokio::spawn(async move {
                        while let Some(res) = stream.next().await {
                            match res {
                                Ok(chunk) => {
                                    if !chunk.audio_chunk.is_empty() {
                                        let content_type = format!("audio/L16;rate={}", requested_sample_rate);
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type,
                                            provider_used: "mms".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() { break; }
                                    }
                                }
                                Err(e) => { 
                                    error!(error = ?e, "MMS upstream error"); 
                                    let _ = tx.send(Err(e)).await; 
                                    break; 
                                }
                            }
                        }
                    });
                },
                Err(e) => {
                    error!(error = ?e, "Failed to connect to MMS upstream");
                    return Err(e);
                }
            }

        } else {
            // --- COQUI MOTORU (Varsayılan) ---
            let coqui_req = CoquiSynthesizeStreamRequest {
                text: req.text,
                language_code: "tr".to_string(),
                speaker_wav: req.cloning_audio_data,
                temperature: tuning.temperature,
                speed: tuning.speed,
                top_p: tuning.top_p,
                top_k: tuning.top_k as f32,
                repetition_penalty: tuning.repetition_penalty,
                output_format: "pcm".to_string(),
                sample_rate: requested_sample_rate,
            };

            match self.coqui_client.synthesize_stream(coqui_req, trace_id.clone()).await {
                Ok(mut stream) => {
                    tokio::spawn(async move {
                        while let Some(res) = stream.next().await {
                            match res {
                                Ok(chunk) => {
                                    if !chunk.audio_chunk.is_empty() {
                                        let content_type = format!("audio/L16;rate={}", requested_sample_rate);
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type,
                                            provider_used: "coqui".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() { break; }
                                    }
                                }
                                Err(e) => { 
                                    error!(error = ?e, "Coqui upstream error"); 
                                    let _ = tx.send(Err(e)).await; 
                                    break; 
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!(error = ?e, "Failed to connect to Coqui upstream");
                    return Err(e);
                }
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}