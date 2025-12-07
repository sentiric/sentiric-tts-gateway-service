mod config;
mod app;
mod error;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Uygulama yaşam döngüsünü başlat
    App::bootstrap().await?.run().await
}