use thiserror::Error;

pub type Result<T> = std::result::Result<T, OpenGenAiError>;

#[derive(Error, Debug)]
pub enum OpenGenAiError {
    #[error("API Error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Model not found: '{0}'")]
    ModelNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Provider '{0}' is unavailable")]
    ProviderUnavailable(String),

    #[error("Job timeout waiting for request ID '{0}'")]
    JobTimeout(String),

    #[error("Job failed for request ID '{0}': {1}")]
    JobFailed(String, String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),

    #[error("Local inference error: {0}")]
    LocalInferenceError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    YamlSerialization(#[from] serde_yaml::Error),

    #[error("Network request error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),
}
