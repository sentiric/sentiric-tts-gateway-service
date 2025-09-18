use anyhow::{Context, Result};
use reqwest::Client;
use sentiric_contracts::sentiric::tts::v1::{
    text_to_speech_service_server::{TextToSpeechService, TextToSpeechServiceServer},
    SynthesizeRequest, SynthesizeResponse,
};
use serde::Serialize;
use std::env;
use std::net::SocketAddr;
use tonic::{
    transport::{Certificate, Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};
use tracing::{error, info, instrument, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

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

pub struct MyTtsGatewayService {
    http_client: Client,
    tts_edge_service_url: String,
    tts_coqui_service_url: Option<String>,
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
        let use_coqui_for_cloning = req.speaker_wav_url.is_some();

        if use_coqui_for_cloning {
            if self.tts_coqui_service_url.is_some() {
                info!("Ses klonlama isteği, Coqui-TTS motoruna yönlendiriliyor.");
                self.proxy_to_coqui(req).await
            } else {
                warn!("Ses klonlama isteği alındı ancak Coqui-TTS servisi yapılandırılmamış.");
                Err(Status::failed_precondition(
                    "Bu sunucu ses klonlama (voice cloning) için yapılandırılmamıştır.",
                ))
            }
        } else {
            info!("Standart sentezleme isteği, Edge-TTS motoruna yönlendiriliyor.");
            self.proxy_to_edge(req).await
        }
    }
}

impl MyTtsGatewayService {
    async fn proxy_to_coqui(
        &self,
        req: SynthesizeRequest,
    ) -> Result<Response<SynthesizeResponse>, Status> {
        let target_url = self.tts_coqui_service_url.as_ref().unwrap();
        let payload = CoquiTtsRequest {
            text: &req.text,
            language: &req.language_code,
            speaker_wav_url: req.speaker_wav_url.as_deref(),
        };
        info!(target_url = %target_url, "Coqui-TTS'e POST isteği gönderiliyor.");
        let res = self
            .http_client
            .post(target_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Uzman Coqui TTS servisine bağlanılamadı.");
                Status::unavailable("Coqui servisine ulaşılamıyor.")
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_else(|_| "No error body".to_string());
            error!(status = %status, body = %err_body, "Coqui servisi hata döndürdü.");
            return Err(Status::internal("Coqui servisi hata döndürdü."));
        }

        let audio_bytes = res
            .bytes()
            .await
            .map_err(|_| Status::internal("Coqui'den ses verisi okunamadı."))?;
        info!("Uzman motordan (Coqui-TTS) ses başarıyla alındı.");

        Ok(Response::new(SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-coqui-service".to_string(),
        }))
    }

    async fn proxy_to_edge(
        &self,
        req: SynthesizeRequest,
    ) -> Result<Response<SynthesizeResponse>, Status> {
        const DEFAULT_VOICE: &str = "tr-TR-EmelNeural";
        let voice = match req.voice_selector.as_deref() {
            Some(v) if !v.trim().is_empty() => {
                info!(voice = %v, "Gelen istekten ses seçici kullanılıyor.");
                v.to_string()
            }
            _ => {
                warn!(
                    "Gelen istekte 'voice_selector' alanı boş veya tanımsız. Varsayılan ses kullanılıyor: {}",
                    DEFAULT_VOICE
                );
                DEFAULT_VOICE.to_string()
            }
        };

        let payload = EdgeTtsRequest {
            text: &req.text,
            voice: &voice,
        };
        info!(target_url = %self.tts_edge_service_url, "Edge-TTS'e POST isteği gönderiliyor.");
        let res = self
            .http_client
            .post(&self.tts_edge_service_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Uzman Edge TTS servisine bağlanılamadı.");
                Status::unavailable("Edge servisine ulaşılamıyor.")
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_else(|_| "No error body".to_string());
            error!(status = %status, body = %err_body, "Edge servisi hata döndürdü.");
            return Err(Status::internal("Edge servisi hata döndürdü."));
        }

        let audio_bytes = res
            .bytes()
            .await
            .map_err(|_| Status::internal("Edge'den ses verisi okunamadı."))?;
        info!("Uzman motordan (Edge-TTS) ses başarıyla alındı.");

        Ok(Response::new(SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-edge-service".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let env = env::var("ENV").unwrap_or_else(|_| "production".to_string());
    
    // Standart Loglama Yapılandırması
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(env_filter);
    
    if env == "development" {
        subscriber.with(fmt::layer().with_target(true).with_line_number(true).with_span_events(FmtSpan::NONE)).init();
    } else {
        subscriber.with(fmt::layer().json().with_current_span(true).with_span_list(true).with_span_events(FmtSpan::NONE)).init();
    }

    let service_version = env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let git_commit = env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build_date = env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".to_string());

    info!(
        service_name = "sentiric-tts-gateway-service",
        version = %service_version,
        commit = %git_commit,
        build_date = %build_date,
        profile = %env,
        "🚀 Servis başlatılıyor..."
    );

    let port = env::var("TTS_GATEWAY_GRPC_PORT").unwrap_or_else(|_| "14011".to_string());
    let addr: SocketAddr = format!("[::]:{}", port).parse()?;
    
    let tts_edge_service_url = env::var("TTS_EDGE_SERVICE_HTTP_URL").context("TTS_EDGE_SERVICE_HTTP_URL ortam değişkeni bulunamadı!")?;
    let tts_coqui_service_url = env::var("TTS_COQUI_SERVICE_HTTP_URL")
        .ok()
        .map(|url| format!("{}/api/v1/synthesize", url));
    
    if let Some(url) = &tts_coqui_service_url {
        info!(coqui_url = %url, "Coqui-TTS entegrasyonu aktif.");
    } else {
        warn!("TTS_COQUI_SERVICE_HTTP_URL ortam değişkeni ayarlanmamış. Coqui-TTS (ses klonlama) özelliği devre dışı.");
    }

    let tts_service = MyTtsGatewayService {
        http_client: Client::new(),
        tts_edge_service_url: format!("{}/api/v1/synthesize", tts_edge_service_url),
        tts_coqui_service_url,
    };
    
    let cert_path = env::var("TTS_GATEWAY_CERT_PATH").context("TTS_GATEWAY_CERT_PATH eksik")?;
    let key_path = env::var("TTS_GATEWAY_KEY_PATH").context("TTS_GATEWAY_KEY_PATH eksik")?;
    let ca_path = env::var("GRPC_TLS_CA_PATH").context("GRPC_TLS_CA_PATH eksik")?;

    let identity = {
        let cert = tokio::fs::read(&cert_path).await.context("Sunucu sertifikası okunamadı")?;
        let key = tokio::fs::read(&key_path).await.context("Sunucu anahtarı okunamadı")?;
        Identity::from_pem(cert, key)
    };
    
    let client_ca_cert = {
        let ca = tokio::fs::read(&ca_path).await.context("CA sertifikası okunamadı")?;
        Certificate::from_pem(ca)
    };

    let tls_config = ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_cert);

    info!(address = %addr, "TTS Gateway gRPC sunucusu (mTLS ile) dinleniyor...");
    
    Server::builder()
        .tls_config(tls_config)?
        .add_service(TextToSpeechServiceServer::new(tts_service))
        .serve(addr)
        .await?;

    Ok(())
}