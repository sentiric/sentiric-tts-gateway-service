// sentiric-tts-gateway-service/src/error.rs
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Uzman TTS servisine ulaşılamıyor: {0}")]
    UpstreamUnavailable(String),
    #[error("Uzman TTS servisi hata döndürdü: {0}")]
    UpstreamError(String),
    #[error("Uzman TTS servisinden ses verisi okunamadı: {0}")]
    UpstreamReadError(String),
    #[error("Bu sunucu ses klonlama için yapılandırılmamıştır.")]
    CloningNotConfigured,
}

impl From<GatewayError> for Status {
    fn from(err: GatewayError) -> Self {
        match err {
            GatewayError::UpstreamUnavailable(msg) => Status::unavailable(msg),
            GatewayError::CloningNotConfigured => Status::failed_precondition(err.to_string()),
            _ => Status::internal(err.to_string()),
        }
    }
}