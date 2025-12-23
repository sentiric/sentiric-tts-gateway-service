use crate::config::AppConfig;
use anyhow::{Context, Result};
use tonic::transport::{Certificate, Identity, ServerTlsConfig, ClientTlsConfig};

pub async fn load_server_tls_config(config: &AppConfig) -> Result<ServerTlsConfig> {
    let cert = tokio::fs::read(&config.tts_gateway_service_cert_path).await
        .context("Cert read failed")?;
    let key = tokio::fs::read(&config.tts_gateway_service_key_path).await
        .context("Key read failed")?;
    let identity = Identity::from_pem(cert, key);
    
    let ca = tokio::fs::read(&config.grpc_tls_ca_path).await
        .context("CA read failed")?;
    let client_ca = Certificate::from_pem(ca);

    Ok(ServerTlsConfig::new().identity(identity).client_ca_root(client_ca))
}

pub async fn load_client_tls_config(config: &AppConfig) -> Result<ClientTlsConfig> {
    let ca = tokio::fs::read(&config.grpc_tls_ca_path).await?;
    let ca_cert = Certificate::from_pem(ca);

    let cert = tokio::fs::read(&config.tts_gateway_service_cert_path).await?;
    let key = tokio::fs::read(&config.tts_gateway_service_key_path).await?;
    let identity = Identity::from_pem(cert, key);

    Ok(ClientTlsConfig::new()
        .domain_name("sentiric.cloud")
        .ca_certificate(ca_cert)
        .identity(identity))
}