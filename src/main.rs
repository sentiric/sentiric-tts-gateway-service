use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info, instrument};
use tracing_subscriber::EnvFilter;

use sentiric_contracts::sentiric::tts::v1::{
    text_to_speech_service_server::{TextToSpeechService, TextToSpeechServiceServer},
    SynthesizeRequest, SynthesizeResponse,
};

use reqwest::Client;
use serde::Serialize;

// Uzman motorlara gönderilecek request body'leri için struct'lar
#[derive(Serialize)]
struct CoquiTtsRequest<'a> {
    text: &'a str,
    language: &'a str,
    speaker_wav_url: Option<&'a str>,
}

#[derive(Serialize)]
struct EdgeTtsRequest<'a> {
    text: &'a str,
    voice: &'a str,
}

// Servis state'i
pub struct MyTtsGatewayService {
    http_client: Client,
    tts_coqui_service_url: String,
    tts_edge_service_url: String,
    tts_elevenlabs_service_url: String,
    tts_styletts2_service_url: String,
}

#[tonic::async_trait]
impl TextToSpeechService for MyTtsGatewayService {
    #[instrument(skip(self), fields(text = %request.get_ref().text))]
    async fn synthesize(
        &self,
        request: Request<SynthesizeRequest>,
    ) -> Result<Response<SynthesizeResponse>, Status> {
        let req = request.into_inner();
        
        // --- AKILLI YÖNLENDİRME MANTIĞI ---
        // Eğer ses klonlama isteniyorsa Coqui'ye git, yoksa Edge'e git.
        if req.speaker_wav_url.is_some() {
            info!("Ses klonlama isteği, Coqui-TTS motoruna yönlendiriliyor.");
            self.proxy_to_coqui(req).await
        } else {
            info!("Standart sentezleme isteği, Edge-TTS motoruna yönlendiriliyor.");
            self.proxy_to_edge(req).await
        }
    }
}

impl MyTtsGatewayService {
    async fn proxy_to_coqui(&self, req: SynthesizeRequest) -> Result<Response<SynthesizeResponse>, Status> {
        let speaker_url_str = req.speaker_wav_url.as_deref();
        let payload = CoquiTtsRequest {
            text: &req.text,
            language: &req.language_code,
            speaker_wav_url: speaker_url_str,
        };

        let res = self.http_client.post(&self.tts_coqui_service_url).json(&payload).send().await
            .map_err(|e| { error!(error = %e, "Uzman Coqui TTS servisine bağlanılamadı."); Status::unavailable("Coqui servisine ulaşılamıyor.") })?;
        
        if !res.status().is_success() { /* ... Hata yönetimi ... */ return Err(Status::internal("Coqui servisi hata döndürdü.")); }
        
        let audio_bytes = res.bytes().await.map_err(|_| Status::internal("Coqui'den ses verisi okunamadı."))?;
        info!("Uzman motordan (Coqui-TTS) ses başarıyla alındı.");

        Ok(Response::new(SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-coqui-service".to_string(),
        }))
    }

    async fn proxy_to_edge(&self, req: SynthesizeRequest) -> Result<Response<SynthesizeResponse>, Status> {
        // voice_selector'da bir ses belirtilmiş mi kontrol et, yoksa varsayılanı kullan
        let voice = req.voice_selector.unwrap_or_else(|| "tr-TR-AhmetNeural".to_string());
        
        let payload = EdgeTtsRequest { text: &req.text, voice: &voice };

        let res = self.http_client.post(&self.tts_edge_service_url).json(&payload).send().await
            .map_err(|e| { error!(error = %e, "Uzman Edge TTS servisine bağlanılamadı."); Status::unavailable("Edge servisine ulaşılamıyor.") })?;
        
        if !res.status().is_success() { /* ... Hata yönetimi ... */ return Err(Status::internal("Edge servisi hata döndürdü.")); }
        
        let audio_bytes = res.bytes().await.map_err(|_| Status::internal("Edge'den ses verisi okunamadı."))?;
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
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(env_filter);
    
    if env == "development" { subscriber_builder.init(); } else { subscriber_builder.json().init(); }

    let port = env::var("TTS_GATEWAY_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr: SocketAddr = format!("[::]:{}", port).parse()?;
    
    let tts_coqui_service_url = env::var("TTS_COQUI_SERVICE_URL").context("TTS_COQUI_SERVICE_URL ortam değişkeni bulunamadı!")?;
    let tts_edge_service_url = env::var("TTS_EDGE_SERVICE_URL").context("TTS_EDGE_SERVICE_URL ortam değişkeni bulunamadı!")?;
    let tts_elevenlabs_service_url = env::var("TTS_ELEVENLABS_SERVICE_URL").context("TTS_ELEVENLABS_SERVICE_URL ortam değişkeni bulunamadı!")?;
    let tts_styletts2_service_url = env::var("TTS_STYLETTS2_SERVICE_URL").context("TTS_STYLETTS2_SERVICE_URL ortam değişkeni bulunamadı!")?;

    let tts_service = MyTtsGatewayService {
        http_client: Client::new(),
        tts_coqui_service_url: format!("//{}/api/v1/synthesize", tts_coqui_service_url),
        tts_edge_service_url: format!("//{}/api/v1/synthesize", tts_edge_service_url),
        tts_elevenlabs_service_url: format!("//{}/api/v1/synthesize", tts_elevenlabs_service_url),
        tts_styletts2_service_url: format!("//{}/api/v1/synthesize", tts_styletts2_service_url),
    };

    info!(address = %addr, "TTS Gateway gRPC sunucusu dinleniyor...");
    
    Server::builder()
        .add_service(TextToSpeechServiceServer::new(tts_service))
        .serve(addr)
        .await?;

    Ok(())
}