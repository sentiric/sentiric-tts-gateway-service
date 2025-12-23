mod app;
mod config;
mod error;
mod grpc;
mod clients;
mod tls;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    App::run().await
}