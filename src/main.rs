// Dosya: src/main.rs
mod app;
mod clients;
mod config;
mod error;
mod grpc;
mod logger;
mod metrics;
mod tls; //[ARCH-COMPLIANCE] SUTS v4.0 Formatter Modülü Binary Ağacına Eklendi

use anyhow::Result;
use app::App;
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;

#[tokio::main]
async fn main() -> Result<()> {
    let provider = default_provider();
    CryptoProvider::install_default(provider).expect("Failed to install crypto provider");
    App::run().await
}
