use crate::config::AppConfig;
use anyhow::{Context, Result};
use tonic::transport::{Certificate, Identity, ServerTlsConfig, ClientTlsConfig};

// Sunucu tarafı mTLS konfigürasyonu (Gateway'in kendisi için)
pub async fn load_server_tls_config(config: &AppConfig) -> Result<ServerTlsConfig> {
    let cert = tokio::fs::read(&config.tts_gateway_service_cert_path).await
        .context(format!("Failed to read cert: {}", config.tts_gateway_service_cert_path))?;
    
    let key = tokio::fs::read(&config.tts_gateway_service_key_path).await
        .context(format!("Failed to read key: {}", config.tts_gateway_service_key_path))?;
    
    let identity = Identity::from_pem(cert, key);
    
    let ca_cert = tokio::fs::read(&config.grpc_tls_ca_path).await
        .context(format!("Failed to read CA: {}", config.grpc_tls_ca_path))?;
    
    let client_ca_root = Certificate::from_pem(ca_cert);

    Ok(ServerTlsConfig::new()
        .identity(identity)
        .client_ca_root(client_ca_root))
}

// İstemci tarafı mTLS konfigürasyonu (Upstream servislere bağlanmak için)
pub async fn load_client_tls_config(config: &AppConfig) -> Result<ClientTlsConfig> {
    let ca_cert = tokio::fs::read(&config.grpc_tls_ca_path).await
        .context(format!("Failed to read CA for client: {}", config.grpc_tls_ca_path))?;
    
    let ca = Certificate::from_pem(ca_cert);

    let cert = tokio::fs::read(&config.tts_gateway_service_cert_path).await?;
    let key = tokio::fs::read(&config.tts_gateway_service_key_path).await?;
    let identity = Identity::from_pem(cert, key);

    Ok(ClientTlsConfig::new()
        .domain_name("sentiric.cloud") // Sertifikadaki SAN ile eşleşmeli
        .ca_certificate(ca)
        .identity(identity))
}