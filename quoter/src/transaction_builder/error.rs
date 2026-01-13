use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionBuilderError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Failed to parse account data")]
    AccountParseError,

    #[error("Unsupported chain type: {0}")]
    UnsupportedChain(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
