use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("OpenAI API error: {0}")]
    OpenAi(#[from] async_openai::error::OpenAIError),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("ADB error: {0}")]
    Adb(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Service not available: {0}")]
    ServiceUnavailable(String),
}

pub type DomainResult<T> = std::result::Result<T, DomainError>;

impl From<DomainError> for rust_mcp_sdk::schema::RpcError {
    fn from(err: DomainError) -> Self {
        rust_mcp_sdk::schema::RpcError::internal_error().with_message(err.to_string())
    }
}
