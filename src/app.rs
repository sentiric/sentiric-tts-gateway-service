// sentiric-tts-gateway-service/src/app.rs
use crate::config::AppConfig;
use crate::grpc::service::MyTtsGatewayService;
use crate::services::tts_proxy::TtsProxyService;
use crate::tls::load_server_tls_config;
use anyhow::{Context, Result};
use sentiric_contracts::sentiric::tts::v1::text_to_speech_service_server::TextToSpeechServiceServer;
use std::convert::Infallible; // YENİ
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::transport::Server as GrpcServer; // YENİ: İsim çakışmasını önlemek için
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
// YENİ: Hyper kütüphanesini ekliyoruz
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server as HttpServer, StatusCode,
};


pub struct App {
    config: Arc<AppConfig>,
}

// YENİ: Basit HTTP isteklerini işleyecek fonksiyon
async fn handle_http_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/healthz") => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"status":"ok"}"#))
            .unwrap()),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

impl App {
    pub async fn bootstrap() -> Result<Self> {
        dotenvy::dotenv().ok();
        let config = Arc::new(AppConfig::load_from_env().context("Konfigürasyon dosyası yüklenemedi")?);

        let rust_log_env = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,h2=warn,hyper=warn,tower=warn,rustls=warn".to_string());
        
        let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(&rust_log_env))?;
        let subscriber = Registry::default().with(env_filter);
        
        if config.env == "development" {
            subscriber.with(fmt::layer().with_target(true).with_line_number(true)).init();
        } else {
            subscriber.with(fmt::layer().json().with_current_span(true).with_span_list(true)).init();
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

        // --- gRPC Sunucusunu Başlat ---
        let grpc_config = self.config.clone();
        let grpc_server_handle = tokio::spawn(async move {
            let tls_config = load_server_tls_config(&grpc_config).await?;
            let proxy_service = Arc::new(TtsProxyService::new(grpc_config.clone()));
            let grpc_service = MyTtsGatewayService::new(proxy_service);
            
            info!(address = %grpc_config.grpc_listen_addr, "Güvenli gRPC sunucusu dinlemeye başlıyor...");
            
            GrpcServer::builder()
                .tls_config(tls_config)?
                .add_service(TextToSpeechServiceServer::new(grpc_service))
                .serve_with_shutdown(grpc_config.grpc_listen_addr, async {
                    shutdown_rx.recv().await;
                    info!("gRPC sunucusu için kapatma sinyali alındı.");
                })
                .await
                .context("gRPC sunucusu hatayla sonlandı")
        });

        // --- YENİ: HTTP Sunucusunu Başlat ---
        let http_config = self.config.clone();
        let (http_shutdown_tx, http_shutdown_rx) = tokio::sync::oneshot::channel();
        let http_server_handle = tokio::spawn(async move {
            let addr = http_config.http_listen_addr;
            let make_svc = make_service_fn(|_conn| async {
                Ok::<_, Infallible>(service_fn(handle_http_request))
            });

            let server = HttpServer::bind(&addr)
                .serve(make_svc)
                .with_graceful_shutdown(async {
                    http_shutdown_rx.await.ok();
                    info!("HTTP sunucusu için kapatma sinyali alındı.");
                });
            
            info!(address = %addr, "HTTP sağlık kontrol sunucusu dinlemeye başlıyor...");
            if let Err(e) = server.await {
                error!(error = %e, "HTTP sunucusu hatayla sonlandı");
            }
        });

        let ctrl_c = async { tokio::signal::ctrl_c().await.expect("Ctrl+C dinleyicisi kurulamadı"); };
        
        tokio::select! {
            res = grpc_server_handle => {
                error!("gRPC sunucusu beklenmedik şekilde sonlandı!");
                if let Err(e) = res? { return Err(e); }
            },
            res = http_server_handle => {
                error!("HTTP sunucusu beklenmedik şekilde sonlandı!");
                res?;
            },
            _ = ctrl_c => {},
        }

        warn!("Kapatma sinyali alındı. Graceful shutdown başlatılıyor...");
        let _ = shutdown_tx.send(()).await;
        let _ = http_shutdown_tx.send(());
        
        info!("Servis başarıyla durduruldu.");
        Ok(())
    }
}