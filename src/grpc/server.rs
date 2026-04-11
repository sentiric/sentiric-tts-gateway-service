// Dosya: src/grpc/server.rs
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;
use crate::clients::omnivoice::OmnivoiceClient;
use futures::StreamExt;
use sentiric_contracts::sentiric::tts::v1::tts_gateway_service_server::TtsGatewayService;
use sentiric_contracts::sentiric::tts::v1::{
    CoquiSynthesizeStreamRequest, ListVoicesRequest, ListVoicesResponse,
    MmsSynthesizeStreamRequest, OmnivoiceSynthesizeStreamRequest, SynthesizeStreamRequest,
    SynthesizeStreamResponse, TuningParams,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument, warn, Span};

pub struct TtsGateway {
    coqui_client: CoquiClient,
    mms_client: MmsClient,
    omnivoice_client: OmnivoiceClient,
}

impl TtsGateway {
    pub fn new(
        coqui_client: CoquiClient,
        mms_client: MmsClient,
        omnivoice_client: OmnivoiceClient,
    ) -> Self {
        Self {
            coqui_client,
            mms_client,
            omnivoice_client,
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
            "Unary Synthesize is deprecated. Use SynthesizeStream.",
        ))
    }

    async fn list_voices(
        &self,
        _req: Request<ListVoicesRequest>,
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

        if tenant_id_opt.is_none() || ten_log.is_empty() {
            error!(event = "MISSING_TENANT_ID", trace_id = %tid_log, span_id = %sid_log, "Tenant ID missing.");
            return Err(Status::invalid_argument("tenant_id is strictly required"));
        }

        let req = request.into_inner();
        let voice_selector = req.voice_id.clone();
        Span::current().record("voice_id", voice_selector.as_str());

        let audio_config = req
            .audio_config
            .clone()
            .ok_or_else(|| Status::invalid_argument("AudioConfig is mandatory."))?;

        let requested_sample_rate = audio_config.sample_rate_hertz;
        Span::current().record("sample_rate", requested_sample_rate);

        let tuning = req.tuning.clone().unwrap_or(TuningParams {
            temperature: 0.75,
            speed: 1.0,
            top_p: 0.85,
            top_k: 50,
            repetition_penalty: 2.0,
        });

        info!(event = "TTS_STREAM_REQUESTED", trace_id = %tid_log, voice_id = %voice_selector, "TTS Gateway: Yönlendirme ve Fallback Zinciri Başlatılıyor...");

        // ---------------------------------------------------------------------
        // 🛡️ AKILLI ŞELALE STRATEJİSİ (SMART FALLBACK CHAIN)
        // ---------------------------------------------------------------------
        let engine_chain = if voice_selector.starts_with("omnivoice:") {
            vec!["omnivoice", "coqui", "mms"]
        } else if voice_selector.starts_with("mms:") {
            vec!["mms", "omnivoice", "coqui"]
        } else {
            // Varsayılan (veya coqui:)
            vec!["coqui", "omnivoice", "mms"]
        };

        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let mut stream_established = false;

        for engine in engine_chain {
            info!(event = "TTS_ENGINE_TRY", trace_id = %tid_log, engine = %engine, "Motor deneniyor...");

            match engine {
                "omnivoice" => {
                    let omni_req = OmnivoiceSynthesizeStreamRequest {
                        text: req.text.clone(),
                        language_code: "tr".to_string(),
                        voice_guidance_prompt: Some(
                            voice_selector
                                .replace("omnivoice:", "")
                                .replace("coqui:", ""),
                        ),
                        reference_audio: req.cloning_audio_data.clone(),
                        temperature: tuning.temperature,
                        speed: tuning.speed,
                        output_format: "pcm".to_string(),
                        sample_rate: requested_sample_rate,
                    };
                    match self
                        .omnivoice_client
                        .synthesize_stream(
                            omni_req,
                            trace_id_opt.clone(),
                            span_id_opt.clone(),
                            tenant_id_opt.clone(),
                        )
                        .await
                    {
                        Ok(mut stream) => {
                            info!(event = "TTS_ENGINE_SUCCESS", trace_id = %tid_log, engine = "omnivoice", "✅ Motor bağlantısı başarılı!");
                            stream_established = true;
                            tokio::spawn(async move {
                                while let Some(Ok(chunk)) = stream.next().await {
                                    if !chunk.audio_chunk.is_empty() {
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type: format!(
                                                "audio/L16;rate={}",
                                                requested_sample_rate
                                            ),
                                            provider_used: "omnivoice".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            });
                            break;
                        }
                        Err(e) => {
                            warn!(event = "TTS_ENGINE_UNAVAILABLE", trace_id = %tid_log, engine = "omnivoice", error = %e, "⚠️ Motor ulaşılamaz durumda. Fallback zincirinde bir sonrakine geçiliyor.");
                        }
                    }
                }
                "coqui" => {
                    let coqui_req = CoquiSynthesizeStreamRequest {
                        text: req.text.clone(),
                        language_code: "tr".to_string(),
                        speaker_wav: req.cloning_audio_data.clone(),
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
                            info!(event = "TTS_ENGINE_SUCCESS", trace_id = %tid_log, engine = "coqui", "✅ Motor bağlantısı başarılı!");
                            stream_established = true;
                            tokio::spawn(async move {
                                while let Some(Ok(chunk)) = stream.next().await {
                                    if !chunk.audio_chunk.is_empty() {
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type: format!(
                                                "audio/L16;rate={}",
                                                requested_sample_rate
                                            ),
                                            provider_used: "coqui".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            });
                            break;
                        }
                        Err(e) => {
                            warn!(event = "TTS_ENGINE_UNAVAILABLE", trace_id = %tid_log, engine = "coqui", error = %e, "⚠️ Motor ulaşılamaz durumda. Fallback zincirinde bir sonrakine geçiliyor.");
                        }
                    }
                }
                "mms" => {
                    let mms_req = MmsSynthesizeStreamRequest {
                        text: req.text.clone(),
                        language_code: "tur".to_string(),
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
                            info!(event = "TTS_ENGINE_SUCCESS", trace_id = %tid_log, engine = "mms", "✅ Motor bağlantısı başarılı!");
                            stream_established = true;
                            tokio::spawn(async move {
                                while let Some(Ok(chunk)) = stream.next().await {
                                    if !chunk.audio_chunk.is_empty() {
                                        let resp = SynthesizeStreamResponse {
                                            audio_content: chunk.audio_chunk,
                                            content_type: format!(
                                                "audio/L16;rate={}",
                                                requested_sample_rate
                                            ),
                                            provider_used: "mms".to_string(),
                                        };
                                        if tx.send(Ok(resp)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            });
                            break;
                        }
                        Err(e) => {
                            warn!(event = "TTS_ENGINE_UNAVAILABLE", trace_id = %tid_log, engine = "mms", error = %e, "⚠️ Motor ulaşılamaz durumda. Fallback zincirinde bir sonrakine geçiliyor.");
                        }
                    }
                }
                _ => {}
            }
        }

        if !stream_established {
            error!(event = "TTS_ALL_ENGINES_FAILED", trace_id = %tid_log, "🚨 FATAL: Hiçbir TTS motoru yanıt vermedi!");
            return Err(Status::unavailable(
                "Tüm uzman TTS motorları devre dışı. Lütfen sistemi kontrol edin.",
            ));
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
