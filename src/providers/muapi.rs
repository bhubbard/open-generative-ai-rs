use super::Provider;
use crate::error::{OpenGenAiError, Result};
use crate::registry::get_model;
use crate::types::{GenerationRequest, JobStatus, PredictionResponse, UploadResponse};
use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;
use tracing::{debug, info};

#[derive(Debug)]
pub struct MuapiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    poll_interval: Duration,
    max_poll_attempts: usize,
}

impl Default for MuapiClient {
    fn default() -> Self {
        Self::new(None)
    }
}

impl MuapiClient {
    pub fn new(api_key: Option<String>) -> Self {
        let key = api_key.or_else(|| std::env::var("MUAPI_KEY").ok());
        let base_url = std::env::var("MUAPI_BASE_URL")
            .unwrap_or_else(|_| "https://api.muapi.ai".to_string())
            .trim_end_matches('/')
            .to_string();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            base_url,
            api_key: key,
            poll_interval: Duration::from_millis(2000),
            max_poll_attempts: 120, // 4 minutes max
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_polling(mut self, interval: Duration, max_attempts: usize) -> Self {
        self.poll_interval = interval;
        self.max_poll_attempts = max_attempts;
        self
    }

    fn get_key(&self) -> Result<&str> {
        self.api_key.as_deref().ok_or_else(|| {
            OpenGenAiError::AuthError(
                "Missing Muapi API key. Set MUAPI_KEY env var or provide via --api-key.".into(),
            )
        })
    }

    /// Build JSON payload conforming to Muapi schema
    pub fn build_payload(&self, req: &GenerationRequest) -> Value {
        let mut map = serde_json::Map::new();

        if let Some(prompt) = &req.prompt {
            map.insert("prompt".into(), json!(prompt));
        }
        if let Some(np) = &req.negative_prompt {
            map.insert("negative_prompt".into(), json!(np));
        }
        if let Some(ratio) = &req.aspect_ratio {
            map.insert("aspect_ratio".into(), json!(ratio));
        }
        if let Some(res) = &req.resolution {
            map.insert("resolution".into(), json!(res));
        }
        if let Some(dur) = req.duration {
            map.insert("duration".into(), json!(dur));
        }
        if let Some(q) = &req.quality {
            map.insert("quality".into(), json!(q));
        }
        if let Some(m) = &req.mode {
            map.insert("mode".into(), json!(m));
        }
        if let Some(seed) = req.seed {
            if seed >= 0 {
                map.insert("seed".into(), json!(seed));
            }
        }
        if let Some(steps) = req.steps {
            map.insert("steps".into(), json!(steps));
        }
        if let Some(cfg) = req.guidance_scale {
            map.insert("guidance_scale".into(), json!(cfg));
        }
        if let Some(strength) = req.strength {
            map.insert("strength".into(), json!(strength));
        }

        // Image conditioning
        if let Some(img) = &req.image_url {
            map.insert("image_url".into(), json!(img));
        }
        if !req.images_list.is_empty() {
            map.insert("images_list".into(), json!(req.images_list));
        }
        if let Some(audio) = &req.audio_url {
            map.insert("audio_url".into(), json!(audio));
        }
        if let Some(video) = &req.video_url {
            map.insert("video_url".into(), json!(video));
        }

        // Feature flags
        if let Some(fixed) = req.camera_fixed {
            map.insert("camera_fixed".into(), json!(fixed));
        }
        if let Some(bitrate) = req.high_bitrate {
            map.insert("high_bitrate".into(), json!(bitrate));
        }
        if let Some(gen_audio) = req.generate_audio {
            map.insert("generate_audio".into(), json!(gen_audio));
        }
        if let Some(wh) = &req.webhook_url {
            map.insert("webhook_url".into(), json!(wh));
        }

        Value::Object(map)
    }
}

#[async_trait]
impl Provider for MuapiClient {
    fn name(&self) -> &'static str {
        "Muapi.ai Gateway"
    }

    async fn submit(&self, req: &GenerationRequest) -> Result<String> {
        let key = self.get_key()?;
        let endpoint = get_model(&req.model)
            .map(|m| m.endpoint.as_str())
            .unwrap_or(req.model.as_str());

        let url = format!("{}/api/v1/{}", self.base_url, endpoint);
        let payload = self.build_payload(req);

        debug!(target: "muapi", "Submitting to {url} with payload {payload}");

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", key)
            .json(&payload)
            .send()
            .await?;

        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            return Err(OpenGenAiError::ApiError {
                status: status.as_u16(),
                message: format!("API error from {url}: {body_text}"),
            });
        }

        let parsed: Value = serde_json::from_str(&body_text)?;
        if let Some(id) = parsed.get("request_id").and_then(|v| v.as_str()) {
            Ok(id.to_string())
        } else if let Some(id) = parsed.get("id").and_then(|v| v.as_str()) {
            Ok(id.to_string())
        } else {
            // Some models return the result synchronously
            Ok(format!("sync_{}", uuid::Uuid::new_v4()))
        }
    }

    async fn poll_status(&self, request_id: &str) -> Result<PredictionResponse> {
        let key = self.get_key()?;
        let url = format!("{}/api/v1/predictions/{}/result", self.base_url, request_id);

        let resp = self
            .client
            .get(&url)
            .header("x-api-key", key)
            .send()
            .await?;

        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            return Err(OpenGenAiError::ApiError {
                status: status.as_u16(),
                message: format!("Status poll failed for {request_id}: {body_text}"),
            });
        }

        let data: Value = serde_json::from_str(&body_text)?;

        let raw_status = data
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("processing");

        let job_status = match raw_status.to_lowercase().as_str() {
            "completed" | "succeeded" | "success" => JobStatus::Completed,
            "failed" | "error" => JobStatus::Failed,
            "canceled" | "cancelled" => JobStatus::Canceled,
            "starting" | "queued" => JobStatus::Starting,
            _ => JobStatus::Processing,
        };

        let mut outputs = Vec::new();
        if let Some(arr) = data.get("outputs").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    outputs.push(s.to_string());
                }
            }
        }
        if outputs.is_empty() {
            if let Some(u) = data.get("url").and_then(|v| v.as_str()) {
                outputs.push(u.to_string());
            } else if let Some(u) = data.pointer("/output/url").and_then(|v| v.as_str()) {
                outputs.push(u.to_string());
            }
        }

        let primary_url = outputs.first().cloned();
        let error = data.get("error").and_then(|e| e.as_str()).map(String::from);

        Ok(PredictionResponse {
            request_id: request_id.to_string(),
            model: data
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
                .to_string(),
            status: job_status,
            outputs,
            primary_url,
            duration_seconds: data.get("duration").and_then(|d| d.as_f64()),
            error,
            metadata: BTreeMap::new(),
        })
    }

    async fn generate_and_wait(&self, req: &GenerationRequest) -> Result<PredictionResponse> {
        let request_id = self.submit(req).await?;

        // Handle direct synchronous response
        if request_id.starts_with("sync_") {
            return Ok(PredictionResponse {
                request_id: request_id.clone(),
                model: req.model.clone(),
                status: JobStatus::Completed,
                outputs: vec![],
                primary_url: None,
                duration_seconds: Some(0.1),
                error: None,
                metadata: BTreeMap::new(),
            });
        }

        info!(target: "muapi", "Polling generation result for request: {request_id}");

        for attempt in 1..=self.max_poll_attempts {
            tokio::time::sleep(self.poll_interval).await;

            let result = self.poll_status(&request_id).await?;
            match result.status {
                JobStatus::Completed => {
                    info!(target: "muapi", "Job {request_id} finished successfully in attempt {attempt}");
                    return Ok(result);
                }
                JobStatus::Failed => {
                    let err = result
                        .error
                        .unwrap_or_else(|| "Unknown provider error".into());
                    return Err(OpenGenAiError::JobFailed(request_id, err));
                }
                JobStatus::Canceled => {
                    return Err(OpenGenAiError::JobFailed(
                        request_id,
                        "Job was canceled".into(),
                    ));
                }
                JobStatus::Starting | JobStatus::Processing => {
                    debug!(target: "muapi", "Job {request_id} still processing (attempt {attempt}/{})", self.max_poll_attempts);
                }
            }
        }

        Err(OpenGenAiError::JobTimeout(request_id))
    }

    async fn upload_media(
        &self,
        filename: &str,
        data: Vec<u8>,
        mime: &str,
    ) -> Result<UploadResponse> {
        let key = self.get_key()?;
        let url = format!("{}/api/v1/upload_file", self.base_url);

        let part = Part::bytes(data.clone())
            .file_name(filename.to_string())
            .mime_str(mime)
            .map_err(|e| OpenGenAiError::InvalidParameter(e.to_string()))?;

        let form = Form::new().part("file", part);

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", key)
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            return Err(OpenGenAiError::ApiError {
                status: status.as_u16(),
                message: format!("Upload failed: {body_text}"),
            });
        }

        let parsed: Value = serde_json::from_str(&body_text)?;
        let remote_url = parsed
            .get("url")
            .and_then(|v| v.as_str())
            .or_else(|| parsed.get("file_url").and_then(|v| v.as_str()))
            .ok_or_else(|| OpenGenAiError::ApiError {
                status: status.as_u16(),
                message: format!("No URL returned from upload response: {body_text}"),
            })?;

        Ok(UploadResponse {
            url: remote_url.to_string(),
            filename: filename.to_string(),
            size_bytes: data.len(),
            mime_type: mime.to_string(),
        })
    }
}
