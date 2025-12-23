mod app;
mod config;
mod error;
mod grpc;
mod clients;
mod tls;
// services modülü kaldırıldı, artık doğrudan grpc içinde implemente ediliyor

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    App::run().await
}