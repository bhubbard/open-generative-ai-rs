use crate::providers::Provider;
use crate::registry::{get_model, models_by_category, search_models, MODEL_CATALOG};
use crate::types::{GenerationRequest, ModelCategory, PredictionResponse};
use crate::workflow::{
    ai_influencer_template, popcorn_storyboard_template, Workflow, WorkflowEngine,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub const EMBEDDED_STUDIO_HTML: &str = include_str!("../assets/studio.html");

#[derive(Clone, Debug)]
pub struct AppState {
    pub provider: Arc<dyn Provider>,
    pub workflow_engine: Arc<WorkflowEngine>,
}

pub fn create_router(provider: Arc<dyn Provider>) -> Router {
    let workflow_engine = Arc::new(WorkflowEngine::new(provider.clone()));
    let state = AppState {
        provider,
        workflow_engine,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health probes
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        // Embedded Web Studio
        .route("/", get(studio_handler))
        .route("/studio", get(studio_handler))
        // Model Catalog
        .route("/api/v1/models", get(list_models_handler))
        .route("/api/v1/models/{id}", get(get_model_handler))
        // Generation endpoints
        .route("/api/v1/generate/image", post(generate_image_handler))
        .route("/api/v1/generate/video", post(generate_video_handler))
        .route("/api/v1/generate/audio", post(generate_audio_handler))
        .route("/api/v1/lipsync", post(lipsync_handler))
        .route("/api/v1/recast", post(recast_handler))
        .route("/api/v1/upload_file", post(upload_file_handler))
        // Predictions polling
        .route("/api/v1/predictions/{id}", get(prediction_status_handler))
        .route(
            "/api/v1/predictions/{id}/result",
            get(prediction_status_handler),
        )
        // Workflows
        .route(
            "/api/v1/workflows/templates",
            get(list_workflow_templates_handler),
        )
        .route("/api/v1/workflows/run", post(run_workflow_handler))
        .layer(cors)
        .with_state(state)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "engine": "open-generative-ai-rs",
        "version": env!("CARGO_PKG_VERSION"),
        "models_count": MODEL_CATALOG.len(),
    }))
}

async fn ready_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ready",
        "ready": true,
    }))
}

async fn studio_handler() -> Html<&'static str> {
    Html(EMBEDDED_STUDIO_HTML)
}

#[derive(Deserialize)]
struct ModelsQuery {
    category: Option<String>,
    search: Option<String>,
}

async fn list_models_handler(Query(q): Query<ModelsQuery>) -> Json<serde_json::Value> {
    let mut list: Vec<&'static crate::types::ModelInfo> = MODEL_CATALOG.iter().collect();

    if let Some(cat_str) = q.category {
        let cat = match cat_str.to_lowercase().as_str() {
            "image" | "text-to-image" => Some(ModelCategory::TextToImage),
            "i2i" | "image-to-image" => Some(ModelCategory::ImageToImage),
            "video" | "text-to-video" => Some(ModelCategory::TextToVideo),
            "i2v" | "image-to-video" => Some(ModelCategory::ImageToVideo),
            "v2v" | "video-to-video" => Some(ModelCategory::VideoToVideo),
            "lipsync" | "lip-sync" => Some(ModelCategory::LipSync),
            "recast" | "swap" => Some(ModelCategory::Recast),
            "audio" | "music" | "tts" => Some(ModelCategory::Audio),
            _ => None,
        };
        if let Some(c) = cat {
            list = models_by_category(c);
        }
    }

    if let Some(query) = q.search {
        if !query.trim().is_empty() {
            list = search_models(&query);
        }
    }

    Json(json!({
        "count": list.len(),
        "models": list,
    }))
}

async fn get_model_handler(
    Path(id): Path<String>,
) -> std::result::Result<Json<crate::types::ModelInfo>, (StatusCode, String)> {
    get_model(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Model '{id}' not found")))
}

async fn generate_image_handler(
    State(state): State<AppState>,
    Json(mut req): Json<GenerationRequest>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    if req.model.is_empty() {
        req.model = "flux-dev".into();
    }
    state
        .provider
        .generate_and_wait(&req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn generate_video_handler(
    State(state): State<AppState>,
    Json(mut req): Json<GenerationRequest>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    if req.model.is_empty() {
        req.model = "kling-v3-pro".into();
    }
    state
        .provider
        .generate_and_wait(&req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn generate_audio_handler(
    State(state): State<AppState>,
    Json(mut req): Json<GenerationRequest>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    if req.model.is_empty() {
        req.model = "text-to-music-v2".into();
    }
    state
        .provider
        .generate_and_wait(&req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn lipsync_handler(
    State(state): State<AppState>,
    Json(mut req): Json<GenerationRequest>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    if req.model.is_empty() {
        req.model = "infinite-talk-i2v".into();
    }
    state
        .provider
        .generate_and_wait(&req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn recast_handler(
    State(state): State<AppState>,
    Json(mut req): Json<GenerationRequest>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    if req.model.is_empty() {
        req.model = "subject-recast-v1".into();
    }
    state
        .provider
        .generate_and_wait(&req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn prediction_status_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<PredictionResponse>, (StatusCode, String)> {
    state
        .provider
        .poll_status(&id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn upload_file_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<Json<crate::types::UploadResponse>, (StatusCode, String)> {
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let filename = field.file_name().unwrap_or("upload.dat").to_string();
        let mime = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
            .to_vec();

        let upload_res = state
            .provider
            .upload_media(&filename, data, &mime)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(upload_res));
    }

    Err((
        StatusCode::BAD_REQUEST,
        "No file field found in multipart request".into(),
    ))
}

async fn list_workflow_templates_handler() -> Json<serde_json::Value> {
    Json(json!([
        ai_influencer_template(),
        popcorn_storyboard_template()
    ]))
}

#[derive(Deserialize)]
struct RunWorkflowRequest {
    workflow: Option<Workflow>,
    template_id: Option<String>,
    #[serde(default)]
    inputs: BTreeMap<String, String>,
}

async fn run_workflow_handler(
    State(state): State<AppState>,
    Json(body): Json<RunWorkflowRequest>,
) -> std::result::Result<Json<crate::workflow::WorkflowReport>, (StatusCode, String)> {
    let wf = match body.workflow {
        Some(w) => w,
        None => {
            let tid = body.template_id.unwrap_or_default();
            match tid.as_str() {
                "ai-influencer" => ai_influencer_template(),
                "popcorn-storyboard" => popcorn_storyboard_template(),
                _ => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("Unknown workflow template: '{tid}'"),
                    ))
                }
            }
        }
    };

    state
        .workflow_engine
        .run(&wf, &body.inputs)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
