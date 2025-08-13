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

#[derive(Serialize)]
struct CoquiTtsRequest<'a> {
    text: &'a str,
    language: &'a str,
    speaker_wav_url: Option<&'a str>,
}

pub struct MyTtsGatewayService {
    http_client: Client,
    coqui_tts_url: String,
}

#[tonic::async_trait]
impl TextToSpeechService for MyTtsGatewayService {
    #[instrument(skip(self), fields(text = %request.get_ref().text))]
    async fn synthesize(
        &self,
        request: Request<SynthesizeRequest>,
    ) -> Result<Response<SynthesizeResponse>, Status> {
        let req = request.into_inner();
        info!("Sentezleme isteği alındı, Coqui-TTS motoruna yönlendiriliyor.");
        
        let speaker_url_str = req.speaker_wav_url.as_deref();

        let payload = CoquiTtsRequest {
            text: &req.text,
            language: &req.language_code,
            speaker_wav_url: speaker_url_str,
        };

        let res = self.http_client
            .post(&self.coqui_tts_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Uzman Coqui TTS servisine bağlanılamadı.");
                Status::unavailable("Uzman Coqui TTS servisine ulaşılamıyor.")
            })?;

        let status = res.status();
        if !status.is_success() {
            let error_body = res.text().await.unwrap_or_else(|_| "Bilinmeyen hata".to_string());
            error!(status = %status, body = %error_body, "Uzman Coqui TTS servisi hata döndürdü.");
            return Err(Status::internal(format!("Uzman Coqui TTS servisi hata döndürdü: {}", error_body)));
        }

        let audio_bytes = res.bytes().await.map_err(|e| {
            error!(error = %e, "Uzman Coqui TTS servisinden gelen ses verisi okunamadı.");
            Status::internal("Ses verisi okunamadı.")
        })?;

        info!("Uzman motordan (Coqui-TTS) ses başarıyla alındı.");

        let response = SynthesizeResponse {
            audio_content: audio_bytes.to_vec(),
            engine_used: "sentiric-tts-coqui-service".to_string(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    
    let env = env::var("ENV").unwrap_or_else(|_| "production".to_string());
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber_builder = tracing_subscriber::fmt().with_env_filter(env_filter);
    
    if env == "development" {
        subscriber_builder.init();
    } else {
        subscriber_builder.json().init();
    }

    let port = env::var("TTS_GATEWAY_PORT").unwrap_or_else(|_| "50051".to_string());
    let addr: SocketAddr = format!("[::]:{}", port).parse()?;
    
    let coqui_tts_url = env::var("TTS_COQUI_URL")
        .context("TTS_COQUI_URL ortam değişkeni bulunamadı!")?;
    let full_coqui_url = format!("http://{}/api/v1/synthesize", coqui_tts_url);

    let tts_service = MyTtsGatewayService {
        http_client: Client::new(),
        coqui_tts_url: full_coqui_url,
    };

    info!(address = %addr, upstream_url = %tts_service.coqui_tts_url, "TTS Gateway gRPC sunucusu dinleniyor...");
    
    Server::builder()
        .add_service(TextToSpeechServiceServer::new(tts_service))
        .serve(addr)
        .await?;

    Ok(())
}