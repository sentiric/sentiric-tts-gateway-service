// Dosya: src/grpc/server.rs
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use futures::StreamExt;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayService;
use sentiric_contracts::sentiric::tts::v1::{
    CoquiSynthesizeStreamRequest, ListVoicesRequest, ListVoicesResponse,
    MmsSynthesizeStreamRequest, SynthesizeStreamRequest, SynthesizeStreamResponse, TuningParams,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument, warn, Span};

pub struct TtsGateway {
    coqui_client: CoquiClient,
    mms_client: MmsClient,
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
        let trace_id_opt = request
            .metadata()
            .get("x-trace-id")
            .and_then(|m| m.to_str().ok())
            .map(|s| s.to_string());
        let span_id_opt = request
            .metadata()
            .get("x-span-id")
            .and_then(|m| m.to_str().ok())
            .map(|s| s.to_string());
        let tenant_id_opt = request
            .metadata()
            .get("x-tenant-id")
            .and_then(|m| m.to_str().ok())
            .map(|s| s.to_string());

        let tid_log = trace_id_opt.as_deref().unwrap_or("unknown");
        let sid_log = span_id_opt.as_deref().unwrap_or("unknown");
        let ten_log = tenant_id_opt.as_deref().unwrap_or("unknown");

        Span::current().record("trace_id", tid_log);

        // [ARCH-COMPLIANCE] Strict Tenant Isolation Check
        if tenant_id_opt.is_none() || ten_log.is_empty() {
            error!(
                event = "MISSING_TENANT_ID",
                trace_id = %tid_log,
                span_id = %sid_log,
                "Tenant ID is missing in request. Request rejected."
            );
            return Err(Status::invalid_argument(
                "tenant_id is strictly required for isolation",
            ));
        }

        let req = request.into_inner();
        let voice_selector = req.voice_id.clone();
        Span::current().record("voice_id", voice_selector.as_str());

        let audio_config = req.audio_config.ok_or_else(|| {
            error!(
                event = "INVALID_REQUEST",
                trace_id = %tid_log,
                span_id = %sid_log,
                tenant_id = %ten_log,
                "AudioConfig is mandatory."
            );
            Status::invalid_argument("AudioConfig is mandatory.")
        })?;

        let requested_sample_rate = audio_config.sample_rate_hertz;
        Span::current().record("sample_rate", requested_sample_rate);

        info!(
            event = "TTS_STREAM_REQUESTED",
            trace_id = %tid_log,
            span_id = %sid_log,
            tenant_id = %ten_log,
            voice_id = %voice_selector,
            sample_rate = %requested_sample_rate,
            "TTS Gateway: İstek Yönlendiriliyor..."
        );

        let (tx, rx) = tokio::sync::mpsc::channel(4096);

        let tuning = req.tuning.unwrap_or(TuningParams {
            temperature: 0.75,
            speed: 1.0,
            top_p: 0.85,
            top_k: 50,
            repetition_penalty: 2.0,
        });

        let tid_thread = tid_log.to_string();
        let sid_thread = sid_log.to_string();
        let ten_thread = ten_log.to_string();

        // --- ROUTING LOGIC ---
        if voice_selector.starts_with("mms:") {
            let lang_code = voice_selector
                .strip_prefix("mms:")
                .unwrap_or("tur")
                .to_string();
            let mms_req = MmsSynthesizeStreamRequest {
                text: req.text,
                language_code: lang_code,
                speed: tuning.speed,
                sample_rate: requested_sample_rate,
            };

            match self
                .mms_client
                .synthesize_stream(
                    mms_req,
                    trace_id_opt.clone(),
                    span_id_opt.clone(),
                    tenant_id_opt.clone(),
                )
                .await
            {
                Ok(mut stream) => {
                    tokio::spawn(async move {
                        while let Some(res) = stream.next().await {
                            match res {
                                Ok(chunk) => {
                                    if !chunk.audio_chunk.is_empty() {
                                        let content_type =
                                            format!("audio/L16;rate={}", requested_sample_rate);
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type,
                                            provider_used: "mms".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() {
                                            warn!(event = "CLIENT_DISCONNECTED", trace_id = %tid_thread, span_id = %sid_thread, tenant_id = %ten_thread, "Client disconnected.");
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(event = "TTS_ENGINE_STREAM_ERROR", trace_id = %tid_thread, span_id = %sid_thread, tenant_id = %ten_thread, error = %e, "MMS upstream error");
                                    let _ = tx.send(Err(e)).await;
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
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

            match self
                .coqui_client
                .synthesize_stream(
                    coqui_req,
                    trace_id_opt.clone(),
                    span_id_opt.clone(),
                    tenant_id_opt.clone(),
                )
                .await
            {
                Ok(mut stream) => {
                    tokio::spawn(async move {
                        while let Some(res) = stream.next().await {
                            match res {
                                Ok(chunk) => {
                                    if !chunk.audio_chunk.is_empty() {
                                        let content_type =
                                            format!("audio/L16;rate={}", requested_sample_rate);
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type,
                                            provider_used: "coqui".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() {
                                            warn!(event = "CLIENT_DISCONNECTED", trace_id = %tid_thread, span_id = %sid_thread, tenant_id = %ten_thread, "Client disconnected.");
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(event = "TTS_ENGINE_STREAM_ERROR", trace_id = %tid_thread, span_id = %sid_thread, tenant_id = %ten_thread, error = %e, "Coqui upstream error");
                                    let _ = tx.send(Err(e)).await;
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
