use crate::types::{ModelCapabilities, ModelCategory, ModelInfo, ProviderKind};
use std::sync::LazyLock;

/// Global catalog of curated models
pub static MODEL_CATALOG: LazyLock<Vec<ModelInfo>> = LazyLock::new(init_catalog);

fn init_catalog() -> Vec<ModelInfo> {
    vec![
        // ── Text-to-Image ──────────────────────────────────────────────────
        ModelInfo {
            id: "flux-dev".into(),
            name: "FLUX.1 [dev]".into(),
            description: "Black Forest Labs 12B parameter state-of-the-art flow matching text-to-image model.".into(),
            category: ModelCategory::TextToImage,
            provider: ProviderKind::Muapi,
            endpoint: "flux-dev".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into(), "4:3".into(), "3:4".into(), "21:9".into()],
                resolutions: vec!["1024x1024".into(), "1344x768".into(), "768x1344".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.025,
            is_local_available: true,
            tags: vec!["bfl".into(), "photorealism".into(), "typography".into(), "flagship".into()],
        },
        ModelInfo {
            id: "flux-schnell".into(),
            name: "FLUX.1 [schnell]".into(),
            description: "Ultra-fast 4-step distilled version of FLUX for rapid prototyping and live previews.".into(),
            category: ModelCategory::TextToImage,
            provider: ProviderKind::Muapi,
            endpoint: "flux-schnell".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into(), "4:3".into(), "3:4".into()],
                resolutions: vec!["1024x1024".into(), "1280x720".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.003,
            is_local_available: true,
            tags: vec!["fast".into(), "distilled".into(), "preview".into()],
        },
        ModelInfo {
            id: "seedream-5.0".into(),
            name: "Seedream 5.0 Ultra".into(),
            description: "High-dynamic-range photo engine specialized in natural skin textures and cinematic lighting.".into(),
            category: ModelCategory::TextToImage,
            provider: ProviderKind::Muapi,
            endpoint: "seedream-5-0".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into(), "4:3".into(), "21:9".into()],
                resolutions: vec!["1080p".into(), "4K".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.035,
            is_local_available: false,
            tags: vec!["cinematic".into(), "hdr".into(), "portrait".into()],
        },
        ModelInfo {
            id: "nano-banana-2".into(),
            name: "Nano Banana 2".into(),
            description: "Lightweight stylized illustration, sticker, and character concept generation.".into(),
            category: ModelCategory::TextToImage,
            provider: ProviderKind::Muapi,
            endpoint: "nano-banana-2".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["1:1".into(), "16:9".into(), "9:16".into()],
                resolutions: vec!["512x512".into(), "1024x1024".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.008,
            is_local_available: true,
            tags: vec!["illustration".into(), "character".into(), "sticker".into()],
        },
        ModelInfo {
            id: "ideogram-v2".into(),
            name: "Ideogram 2.0".into(),
            description: "Premier model for graphic design, accurate typography, logos, and poster compositions.".into(),
            category: ModelCategory::TextToImage,
            provider: ProviderKind::Muapi,
            endpoint: "ideogram-v2".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into(), "4:3".into(), "3:4".into()],
                resolutions: vec!["1024x1024".into(), "1920x1080".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.040,
            is_local_available: false,
            tags: vec!["typography".into(), "graphic-design".into(), "posters".into()],
        },

        // ── Image-to-Image / Inpainting ─────────────────────────────────────
        ModelInfo {
            id: "flux-kontext-pro".into(),
            name: "FLUX Kontext Pro".into(),
            description: "Multi-reference subject-preserving contextual editing with up to 14 image inputs.".into(),
            category: ModelCategory::ImageToImage,
            provider: ProviderKind::Muapi,
            endpoint: "flux-kontext-pro".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into()],
                resolutions: vec!["1024x1024".into(), "1080p".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 14,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.045,
            is_local_available: false,
            tags: vec!["multi-ref".into(), "consistency".into(), "edit".into(), "inpainting".into()],
        },
        ModelInfo {
            id: "nano-banana-2-edit".into(),
            name: "Nano Banana 2 Edit".into(),
            description: "Expressive image style transfer and prompt-directed character modification.".into(),
            category: ModelCategory::ImageToImage,
            provider: ProviderKind::Muapi,
            endpoint: "nano-banana-2-edit".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["1:1".into(), "16:9".into(), "9:16".into()],
                resolutions: vec!["1024x1024".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 4,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.015,
            is_local_available: false,
            tags: vec!["edit".into(), "stylize".into()],
        },
        ModelInfo {
            id: "upscale-clarity-v2".into(),
            name: "Clarity AI 4K Upscaler".into(),
            description: "Super-resolution restoration reconstructing high-frequency skin pores, hair, and fine textures.".into(),
            category: ModelCategory::ImageToImage,
            provider: ProviderKind::Muapi,
            endpoint: "upscale-clarity-v2".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into()],
                resolutions: vec!["2K".into(), "4K".into()],
                max_duration_seconds: None,
                supports_audio: false,
                supports_high_bitrate: true,
                max_input_images: 1,
                supports_camera_motion: false,
                supports_seed: false,
            },
            cost_per_run: 0.020,
            is_local_available: true,
            tags: vec!["upscaler".into(), "enhancement".into(), "4k".into()],
        },

        // ── Text-to-Video ───────────────────────────────────────────────────
        ModelInfo {
            id: "kling-v3-pro".into(),
            name: "Kling v3 Pro".into(),
            description: "Advanced physics-accurate video generation with rich lighting, coherent 3D motion, and 1080p resolution.".into(),
            category: ModelCategory::TextToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "kling-v3-pro".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into()],
                resolutions: vec!["720p".into(), "1080p".into()],
                max_duration_seconds: Some(10),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.25,
            is_local_available: false,
            tags: vec!["video".into(), "cinematic".into(), "physics".into(), "1080p".into()],
        },
        ModelInfo {
            id: "sora-2-turbo".into(),
            name: "Sora 2 Turbo".into(),
            description: "Hyper-realistic multi-shot video generation with prompt adherence and persistent world continuity.".into(),
            category: ModelCategory::TextToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "sora-2-turbo".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into()],
                resolutions: vec!["720p".into(), "1080p".into(), "4K".into()],
                max_duration_seconds: Some(12),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.40,
            is_local_available: false,
            tags: vec!["openai".into(), "photorealism".into(), "multishot".into()],
        },
        ModelInfo {
            id: "veo-3-cinematic".into(),
            name: "Veo 3 Cinematic".into(),
            description: "Google DeepMind Veo 3 engine featuring fluid cinematic camera movements and volumetric lighting.".into(),
            category: ModelCategory::TextToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "veo-3-cinematic".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "21:9".into()],
                resolutions: vec!["1080p".into()],
                max_duration_seconds: Some(8),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.35,
            is_local_available: false,
            tags: vec!["google".into(), "camera-control".into(), "cinematic".into()],
        },
        ModelInfo {
            id: "wan-2.6-t2v".into(),
            name: "Wan 2.6 T2V (Alibaba)".into(),
            description: "Open-weights autoregressive video foundation model with exceptional text prompt tracking.".into(),
            category: ModelCategory::TextToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "wan-2-6-t2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into(), "1:1".into()],
                resolutions: vec!["720p".into(), "1080p".into()],
                max_duration_seconds: Some(5),
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.12,
            is_local_available: true,
            tags: vec!["open-weights".into(), "alibaba".into(), "wan".into()],
        },
        ModelInfo {
            id: "seedance-2.5-t2v".into(),
            name: "Seedance 2.5 T2V".into(),
            description: "ByteDance video synthesis engine with native synchronized audio generation and adaptive aspect ratios.".into(),
            category: ModelCategory::TextToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "seedance-2-5-t2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into(), "16:9".into(), "9:16".into(), "1:1".into(), "4:3".into(), "21:9".into()],
                resolutions: vec!["480p".into(), "720p".into(), "1080p".into()],
                max_duration_seconds: Some(10),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.18,
            is_local_available: false,
            tags: vec!["audio-sync".into(), "adaptive-ratio".into(), "bytedance".into()],
        },

        // ── Image-to-Video ──────────────────────────────────────────────────
        ModelInfo {
            id: "kling-v2.1-i2v".into(),
            name: "Kling v2.1 I2V".into(),
            description: "Breathe lifelike movement, expression, and camera motion into a static starting frame.".into(),
            category: ModelCategory::ImageToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "kling-v2-1-i2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into(), "16:9".into(), "9:16".into()],
                resolutions: vec!["720p".into(), "1080p".into()],
                max_duration_seconds: Some(10),
                supports_audio: false,
                supports_high_bitrate: true,
                max_input_images: 1,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.20,
            is_local_available: false,
            tags: vec!["i2v".into(), "motion".into(), "camera".into()],
        },
        ModelInfo {
            id: "veo-3-i2v".into(),
            name: "Veo 3 I2V".into(),
            description: "High-fidelity conditioning on initial character pose, background depth, and motion prompt.".into(),
            category: ModelCategory::ImageToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "veo-3-i2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into(), "16:9".into(), "9:16".into()],
                resolutions: vec!["1080p".into()],
                max_duration_seconds: Some(8),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 1,
                supports_camera_motion: true,
                supports_seed: true,
            },
            cost_per_run: 0.32,
            is_local_available: false,
            tags: vec!["google".into(), "veo".into(), "i2v".into()],
        },
        ModelInfo {
            id: "wan-2.1-i2v".into(),
            name: "Wan 2.1 I2V Local/Cloud".into(),
            description: "Alibaba Wan 2.1 14B image-to-video model. Run via local Wan2GP Gradio server or cloud gateway.".into(),
            category: ModelCategory::ImageToVideo,
            provider: ProviderKind::LocalWan2Gp,
            endpoint: "wan-2-1-i2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["16:9".into(), "9:16".into()],
                resolutions: vec!["480p".into(), "720p".into()],
                max_duration_seconds: Some(5),
                supports_audio: false,
                supports_high_bitrate: false,
                max_input_images: 1,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.0,
            is_local_available: true,
            tags: vec!["local".into(), "wan2gp".into(), "open-source".into()],
        },

        // ── Video-to-Video ──────────────────────────────────────────────────
        ModelInfo {
            id: "vibe-motion-v2".into(),
            name: "Vibe Motion v2".into(),
            description: "Extract motion choreography from reference video and apply to custom 3D characters or styles.".into(),
            category: ModelCategory::VideoToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "vibe-motion-v2".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into()],
                resolutions: vec!["720p".into(), "1080p".into()],
                max_duration_seconds: Some(15),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 1,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.28,
            is_local_available: false,
            tags: vec!["motion-transfer".into(), "dance".into(), "v2v".into()],
        },
        ModelInfo {
            id: "ai-clipping-pro".into(),
            name: "AI Smart Clipping".into(),
            description: "Detect viral hooks, auto-reframe widescreen video to 9:16, and track speaker faces.".into(),
            category: ModelCategory::VideoToVideo,
            provider: ProviderKind::Muapi,
            endpoint: "ai-clipping-pro".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["9:16".into()],
                resolutions: vec!["1080p".into()],
                max_duration_seconds: Some(60),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: false,
            },
            cost_per_run: 0.05,
            is_local_available: false,
            tags: vec!["shorts".into(), "tiktok".into(), "reframe".into()],
        },

        // ── Lip Sync ────────────────────────────────────────────────────────
        ModelInfo {
            id: "infinite-talk-i2v".into(),
            name: "Infinite Talk I2V".into(),
            description: "State-of-the-art speech-driven talking avatar generator with natural eye blinks and head gestures.".into(),
            category: ModelCategory::LipSync,
            provider: ProviderKind::Muapi,
            endpoint: "infinite-talk-i2v".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into(), "1:1".into(), "9:16".into()],
                resolutions: vec!["720p".into(), "1080p".into()],
                max_duration_seconds: Some(120),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 1,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.09,
            is_local_available: false,
            tags: vec!["lip-sync".into(), "avatar".into(), "speech-to-video".into()],
        },
        ModelInfo {
            id: "wan-2.2-speech-to-video".into(),
            name: "Wan 2.2 Speech to Video".into(),
            description: "Alibaba synchronized multi-lingual speech audio to realistic video face animation.".into(),
            category: ModelCategory::LipSync,
            provider: ProviderKind::Muapi,
            endpoint: "wan-2-2-speech-to-video".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["1:1".into(), "9:16".into(), "16:9".into()],
                resolutions: vec!["720p".into()],
                max_duration_seconds: Some(60),
                supports_audio: true,
                supports_high_bitrate: false,
                max_input_images: 1,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.15,
            is_local_available: false,
            tags: vec!["multilingual".into(), "alibaba".into(), "lip-sync".into()],
        },

        // ── Body Swap / Recast ──────────────────────────────────────────────
        ModelInfo {
            id: "subject-recast-v1".into(),
            name: "Subject Recast AI".into(),
            description: "Seamlessly replace a person in any photo or video with a target character while preserving lighting and clothing folds.".into(),
            category: ModelCategory::Recast,
            provider: ProviderKind::Muapi,
            endpoint: "subject-recast-v1".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec!["adaptive".into()],
                resolutions: vec!["1080p".into()],
                max_duration_seconds: Some(10),
                supports_audio: false,
                supports_high_bitrate: true,
                max_input_images: 2,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.18,
            is_local_available: false,
            tags: vec!["recast".into(), "influencer".into(), "virtual-tryon".into()],
        },

        // ── Audio Generation ────────────────────────────────────────────────
        ModelInfo {
            id: "text-to-music-v2".into(),
            name: "Suno / Udio Style Text-to-Music".into(),
            description: "Generate full commercial-quality stereo music tracks from genre, instruments, and lyric descriptions.".into(),
            category: ModelCategory::Audio,
            provider: ProviderKind::Muapi,
            endpoint: "text-to-music-v2".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec![],
                resolutions: vec![],
                max_duration_seconds: Some(240),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: true,
            },
            cost_per_run: 0.05,
            is_local_available: false,
            tags: vec!["music".into(), "soundtrack".into(), "bgm".into()],
        },
        ModelInfo {
            id: "elevenlabs-voice-tts".into(),
            name: "Neural Voice TTS".into(),
            description: "Ultra-expressive human-like speech synthesis with emotional nuance and custom voice cloning.".into(),
            category: ModelCategory::Audio,
            provider: ProviderKind::Muapi,
            endpoint: "elevenlabs-voice-tts".into(),
            capabilities: ModelCapabilities {
                aspect_ratios: vec![],
                resolutions: vec![],
                max_duration_seconds: Some(300),
                supports_audio: true,
                supports_high_bitrate: true,
                max_input_images: 0,
                supports_camera_motion: false,
                supports_seed: false,
            },
            cost_per_run: 0.015,
            is_local_available: false,
            tags: vec!["tts".into(), "voiceover".into(), "narration".into()],
        },
    ]
}

/// Retrieve a model by its identifier
pub fn get_model(id: &str) -> Option<&'static ModelInfo> {
    MODEL_CATALOG
        .iter()
        .find(|m| m.id == id || m.endpoint == id)
}

/// List all models matching a category
pub fn models_by_category(category: ModelCategory) -> Vec<&'static ModelInfo> {
    MODEL_CATALOG
        .iter()
        .filter(|m| m.category == category)
        .collect()
}

/// Filter models by free-text search (matches id, name, description, tags)
pub fn search_models(query: &str) -> Vec<&'static ModelInfo> {
    let q = query.to_lowercase();
    MODEL_CATALOG
        .iter()
        .filter(|m| {
            m.id.to_lowercase().contains(&q)
                || m.name.to_lowercase().contains(&q)
                || m.description.to_lowercase().contains(&q)
                || m.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

/// Models available for local offline execution (sd.cpp or Wan2GP)
pub fn local_models() -> Vec<&'static ModelInfo> {
    MODEL_CATALOG
        .iter()
        .filter(|m| m.is_local_available)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_integrity() {
        assert!(MODEL_CATALOG.len() >= 15);
        let flux = get_model("flux-dev").expect("flux-dev exists");
        assert_eq!(flux.category, ModelCategory::TextToImage);
        assert!(flux.is_local_available);
    }

    #[test]
    fn test_search_models() {
        let results = search_models("kling");
        assert!(!results.is_empty());
        assert!(results.iter().any(|m| m.id.contains("kling")));
    }

    #[test]
    fn test_categories() {
        let t2v = models_by_category(ModelCategory::TextToVideo);
        assert!(!t2v.is_empty());
        assert!(t2v.iter().any(|m| m.id == "sora-2-turbo"));
    }
}
