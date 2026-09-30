use super::Provider;
use crate::error::Result;
use crate::types::{GenerationRequest, JobStatus, PredictionResponse, UploadResponse};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct MockProvider {
    counter: AtomicUsize,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(1),
        }
    }
}

impl MockProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &'static str {
        "Mock Local Deterministic Engine"
    }

    async fn submit(&self, req: &GenerationRequest) -> Result<String> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(format!("mock_req_{}_{}", req.model, count))
    }

    async fn poll_status(&self, request_id: &str) -> Result<PredictionResponse> {
        let output_url = if request_id.contains("video")
            || request_id.contains("kling")
            || request_id.contains("sora")
        {
            format!("https://cdn.open-generative-ai.org/mock/{}.mp4", request_id)
        } else if request_id.contains("audio")
            || request_id.contains("music")
            || request_id.contains("tts")
        {
            format!("https://cdn.open-generative-ai.org/mock/{}.mp3", request_id)
        } else {
            format!("https://cdn.open-generative-ai.org/mock/{}.png", request_id)
        };

        Ok(PredictionResponse {
            request_id: request_id.to_string(),
            model: "mock-model".into(),
            status: JobStatus::Completed,
            outputs: vec![output_url.clone()],
            primary_url: Some(output_url),
            duration_seconds: Some(0.05),
            error: None,
            metadata: BTreeMap::new(),
        })
    }

    async fn generate_and_wait(&self, req: &GenerationRequest) -> Result<PredictionResponse> {
        let id = self.submit(req).await?;
        self.poll_status(&id).await
    }

    async fn upload_media(
        &self,
        filename: &str,
        data: Vec<u8>,
        mime: &str,
    ) -> Result<UploadResponse> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let url = format!(
            "https://cdn.open-generative-ai.org/uploads/mock_{}_{}",
            count, filename
        );
        Ok(UploadResponse {
            url,
            filename: filename.to_string(),
            size_bytes: data.len(),
            mime_type: mime.to_string(),
        })
    }
}
