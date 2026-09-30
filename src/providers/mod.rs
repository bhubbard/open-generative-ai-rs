pub mod mock;
pub mod muapi;

pub use mock::MockProvider;
pub use muapi::MuapiClient;

use crate::error::Result;
use crate::types::{GenerationRequest, PredictionResponse, UploadResponse};
use async_trait::async_trait;

/// Pluggable backend provider trait
#[async_trait]
pub trait Provider: Send + Sync + std::fmt::Debug {
    /// Human-readable provider identifier
    fn name(&self) -> &'static str;

    /// Submit a generation task asynchronously and return the request ID
    async fn submit(&self, req: &GenerationRequest) -> Result<String>;

    /// Poll or fetch the status of an ongoing request
    async fn poll_status(&self, request_id: &str) -> Result<PredictionResponse>;

    /// Submit and block until generation completes or times out
    async fn generate_and_wait(&self, req: &GenerationRequest) -> Result<PredictionResponse>;

    /// Upload a binary media asset (image/video/audio) and return its public/hosted URL
    async fn upload_media(
        &self,
        filename: &str,
        data: Vec<u8>,
        mime: &str,
    ) -> Result<UploadResponse>;
}
