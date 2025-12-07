use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Unknown error")]
    Unknown,
}