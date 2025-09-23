// sentiric-tts-gateway-service/src/app.rs
use crate::config::AppConfig;
use crate::grpc::service::MyTtsGatewayService;
use crate::services::tts_proxy::TtsProxyService;
use crate::tls::load_server_tls_config;
use anyhow::{Context, Result};
use sentiric_contracts::sentiric::tts::v1::text_to_speech_service_server::TextToSpeechServiceServer;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::transport::Server;
use tracing::{error, info, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter, Registry,
};

pub struct App {
    config: Arc<AppConfig>,
}

impl App {
    pub async fn bootstrap() -> Result<Self> {
        dotenvy::dotenv().ok();
        let config = Arc::new(AppConfig::load_from_env().context("Konfigürasyon dosyası yüklenemedi")?);

        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&config.rust_log))?;
        let subscriber = Registry::default().with(env_filter);
        
        if config.env == "development" {
            subscriber.with(fmt::layer().with_target(true).with_line_number(true).with_span_events(FmtSpan::NONE)).init();
        } else {
            subscriber.with(fmt::layer().json().with_current_span(true).with_span_list(true).with_span_events(FmtSpan::NONE)).init();
        }

        info!(
            service_name = "sentiric-tts-gateway-service",
            version = %config.service_version,
            commit = %config.git_commit,
            build_date = %config.build_date,
            profile = %config.env,
            "🚀 Servis başlatılıyor..."
        );
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        let server_handle = tokio::spawn(async move {
            let tls_config = load_server_tls_config(&self.config).await?;
            
            let proxy_service = Arc::new(TtsProxyService::new(self.config.clone()));

            let grpc_service = MyTtsGatewayService::new(proxy_service);
            
            info!(address = %self.config.grpc_listen_addr, "Güvenli gRPC sunucusu dinlemeye başlıyor...");
            
            Server::builder()
                .tls_config(tls_config)?
                .add_service(TextToSpeechServiceServer::new(grpc_service))
                .serve_with_shutdown(self.config.grpc_listen_addr, async {
                    shutdown_rx.recv().await;
                    info!("gRPC sunucusu için kapatma sinyali alındı.");
                })
                .await
                .context("gRPC sunucusu hatayla sonlandı")
        });

        let ctrl_c = async { tokio::signal::ctrl_c().await.expect("Ctrl+C dinleyicisi kurulamadı"); };
        
        tokio::select! {
            res = server_handle => {
                error!("Sunucu beklenmedik şekilde sonlandı!");
                return res?;
            },
            _ = ctrl_c => {},
        }

        warn!("Kapatma sinyali alındı. Graceful shutdown başlatılıyor...");
        let _ = shutdown_tx.send(()).await;
        
        info!("Servis başarıyla durduruldu.");
        Ok(())
    }
}