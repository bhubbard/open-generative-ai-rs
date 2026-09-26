use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Generative AI task category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCategory {
    TextToImage,
    ImageToImage,
    TextToVideo,
    ImageToVideo,
    VideoToVideo,
    LipSync,
    Recast,
    Audio,
    Workflow,
}

impl std::fmt::Display for ModelCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextToImage => write!(f, "Text-to-Image"),
            Self::ImageToImage => write!(f, "Image-to-Image"),
            Self::TextToVideo => write!(f, "Text-to-Video"),
            Self::ImageToVideo => write!(f, "Image-to-Video"),
            Self::VideoToVideo => write!(f, "Video-to-Video"),
            Self::LipSync => write!(f, "Lip Sync"),
            Self::Recast => write!(f, "Body Swap / Recast"),
            Self::Audio => write!(f, "Audio"),
            Self::Workflow => write!(f, "Workflow"),
        }
    }
}

/// Execution provider backend
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Muapi,
    Replicate,
    FalAi,
    TogetherAi,
    Runware,
    LocalSdCpp,
    LocalWan2Gp,
    Mock,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Muapi => write!(f, "Muapi.ai"),
            Self::Replicate => write!(f, "Replicate"),
            Self::FalAi => write!(f, "fal.ai"),
            Self::TogetherAi => write!(f, "Together AI"),
            Self::Runware => write!(f, "Runware"),
            Self::LocalSdCpp => write!(f, "Local (sd.cpp)"),
            Self::LocalWan2Gp => write!(f, "Local (Wan2GP)"),
            Self::Mock => write!(f, "Mock Engine"),
        }
    }
}

/// Hardware and feature capabilities supported by a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub aspect_ratios: Vec<String>,
    pub resolutions: Vec<String>,
    pub max_duration_seconds: Option<u32>,
    pub supports_audio: bool,
    pub supports_high_bitrate: bool,
    pub max_input_images: usize,
    pub supports_camera_motion: bool,
    pub supports_seed: bool,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            aspect_ratios: vec![
                "16:9".into(),
                "9:16".into(),
                "1:1".into(),
                "4:3".into(),
                "3:4".into(),
            ],
            resolutions: vec!["720p".into(), "1080p".into()],
            max_duration_seconds: Some(5),
            supports_audio: false,
            supports_high_bitrate: false,
            max_input_images: 1,
            supports_camera_motion: false,
            supports_seed: true,
        }
    }
}

/// Metadata and specification for an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ModelCategory,
    pub provider: ProviderKind,
    pub endpoint: String,
    pub capabilities: ModelCapabilities,
    pub cost_per_run: f64,
    pub is_local_available: bool,
    pub tags: Vec<String>,
}

/// Standardized generation request for image, video, audio, or lip sync
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationRequest {
    pub model: String,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub duration: Option<u32>,
    pub quality: Option<String>,
    pub mode: Option<String>,
    pub seed: Option<i64>,
    pub steps: Option<u32>,
    pub guidance_scale: Option<f64>,
    pub strength: Option<f64>,

    // Media conditioning inputs
    pub image_url: Option<String>,
    #[serde(default)]
    pub images_list: Vec<String>,
    pub audio_url: Option<String>,
    pub video_url: Option<String>,

    // Toggles
    pub camera_fixed: Option<bool>,
    pub high_bitrate: Option<bool>,
    pub generate_audio: Option<bool>,

    // Webhook callback
    pub webhook_url: Option<String>,
}

/// Lifecycle status of an async generation job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Starting,
    Processing,
    Completed,
    Failed,
    Canceled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Processing => write!(f, "processing"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Canceled => write!(f, "canceled"),
        }
    }
}

/// Result returned from polling or completing a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResponse {
    pub request_id: String,
    pub model: String,
    pub status: JobStatus,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Submission acknowledgement containing the request ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub request_id: String,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_output: Option<String>,
}

/// Upload response when uploading local files to host/provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
    pub size_bytes: usize,
    pub mime_type: String,
}
