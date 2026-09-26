use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use open_generative_ai::providers::MockProvider;
use open_generative_ai::registry::{get_model, models_by_category, search_models, MODEL_CATALOG};
use open_generative_ai::server::create_router;
use open_generative_ai::types::{GenerationRequest, ModelCategory};
use open_generative_ai::workflow::{
    ai_influencer_template, popcorn_storyboard_template, WorkflowEngine,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_model_catalog_metadata() {
    assert!(MODEL_CATALOG.len() >= 15);

    let flux = get_model("flux-dev").expect("flux-dev should exist");
    assert_eq!(flux.category, ModelCategory::TextToImage);
    assert_eq!(flux.endpoint, "flux-dev");
    assert!(flux
        .capabilities
        .aspect_ratios
        .contains(&"16:9".to_string()));

    let kling = get_model("kling-v3-pro").expect("kling-v3-pro should exist");
    assert_eq!(kling.category, ModelCategory::TextToVideo);
    assert!(kling.capabilities.supports_audio);
    assert!(kling.capabilities.supports_camera_motion);

    let talk = get_model("infinite-talk-i2v").expect("infinite-talk-i2v should exist");
    assert_eq!(talk.category, ModelCategory::LipSync);

    // Search query test
    let search_res = search_models("sora");
    assert!(!search_res.is_empty());
    assert_eq!(search_res[0].id, "sora-2-turbo");

    // Category filter test
    let video_models = models_by_category(ModelCategory::TextToVideo);
    assert!(video_models.len() >= 5);
}

#[tokio::test]
async fn test_mock_provider_generation() {
    use open_generative_ai::providers::Provider;

    let provider = MockProvider::new();
    let req = GenerationRequest {
        model: "flux-dev".into(),
        prompt: Some("Photorealistic cyberpunk city".into()),
        aspect_ratio: Some("16:9".into()),
        ..Default::default()
    };

    let result = provider
        .generate_and_wait(&req)
        .await
        .expect("generation should succeed");
    assert!(result.request_id.contains("flux-dev"));
    assert_eq!(
        result.status,
        open_generative_ai::types::JobStatus::Completed
    );
    assert!(!result.outputs.is_empty());
    assert!(result.outputs[0].ends_with(".png"));
}

#[tokio::test]
async fn test_workflow_engine_execution() {
    let provider = Arc::new(MockProvider::new());
    let engine = WorkflowEngine::new(provider);

    // Test 1: AI Influencer Pipeline
    let inf_wf = ai_influencer_template();
    let mut inputs = BTreeMap::new();
    inputs.insert("prompt".into(), "smiling tech reporter".into());
    inputs.insert("speech_script".into(), "Welcome to our AI channel!".into());

    let report = engine
        .run(&inf_wf, &inputs)
        .await
        .expect("workflow should complete");
    assert_eq!(report.status, "completed");
    assert_eq!(report.steps.len(), 3);
    assert!(report.final_output_url.is_some());

    // Test 2: Popcorn Storyboarding Pipeline
    let sb_wf = popcorn_storyboard_template();
    let mut sb_inputs = BTreeMap::new();
    sb_inputs.insert("scene_prompt".into(), "starship warping into nebula".into());
    sb_inputs.insert("music_genre".into(), "epic cinematic orchestral".into());

    let sb_report = engine
        .run(&sb_wf, &sb_inputs)
        .await
        .expect("storyboard workflow should complete");
    assert_eq!(sb_report.status, "completed");
    assert_eq!(sb_report.steps.len(), 3);
}

#[tokio::test]
async fn test_rest_api_endpoints() {
    let provider = Arc::new(MockProvider::new());
    let app = create_router(provider);

    // 1. Health
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. Ready
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Web Studio Homepage
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/studio")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. List Models
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/models?category=video")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Generate Image API
    let body = serde_json::json!({
        "model": "flux-dev",
        "prompt": "futuristic sunset skyline",
        "aspect_ratio": "16:9"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generate/image")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Generate Video API
    let vid_body = serde_json::json!({
        "model": "kling-v3-pro",
        "prompt": "hyperlapse drone shot of waterfall",
        "duration": 5
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/generate/video")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&vid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 7. Workflow Run API
    let wf_body = serde_json::json!({
        "template_id": "ai-influencer",
        "inputs": {
            "prompt": "influencer host",
            "speech_script": "Checking out the latest in Rust generative AI"
        }
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workflows/run")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&wf_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
