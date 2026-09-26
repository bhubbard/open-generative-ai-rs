use clap::{Parser, Subcommand};
use open_generative_ai::error::Result;
use open_generative_ai::providers::{MockProvider, MuapiClient, Provider};
use open_generative_ai::registry::{get_model, models_by_category, search_models, MODEL_CATALOG};
use open_generative_ai::server::create_router;
use open_generative_ai::types::{GenerationRequest, ModelCategory};
use open_generative_ai::workflow::{
    ai_influencer_template, popcorn_storyboard_template, Workflow, WorkflowEngine,
};
use std::collections::BTreeMap;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "open-genai")]
#[command(
    about = "Open Generative AI: Ultra-fast Rust engine, 400+ model gateway, workflow runner & web studio"
)]
struct Cli {
    /// Use local mock provider for offline testing and zero-cost dry runs
    #[arg(long, global = true)]
    mock: bool,

    /// Muapi API key (defaults to MUAPI_KEY environment variable)
    #[arg(long, env = "MUAPI_KEY", global = true)]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List, search, and inspect catalog models (400+ models)
    Models {
        /// Optional model ID to inspect
        id: Option<String>,

        /// Filter by category (image, video, i2v, lipsync, audio, recast)
        #[arg(short, long)]
        category: Option<String>,

        /// Search query (model name, tag, or description)
        #[arg(short, long)]
        search: Option<String>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate an image, video, or audio asset
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },

    /// Run facial animation and lip sync from face media and speech audio
    Lipsync {
        /// Face image or starting video URL
        #[arg(short, long)]
        image: String,

        /// Speech audio track URL or local file path
        #[arg(short, long)]
        audio: String,

        /// Lip sync model (e.g. infinite-talk-i2v, wan-2.2-speech-to-video)
        #[arg(short, long, default_value = "infinite-talk-i2v")]
        model: String,
    },

    /// Recast person or subject appearance in image or video
    Recast {
        /// Base image or scene video URL
        #[arg(short, long)]
        image: String,

        /// Target character reference image URL
        #[arg(short, long)]
        character: String,

        /// Recast model
        #[arg(short, long, default_value = "subject-recast-v1")]
        model: String,
    },

    /// Multi-step generative AI pipelines and DAG workflows
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },

    /// Launch the high-performance Axum REST API and Web Studio server
    Serve {
        /// Server host bind address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Server port
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[derive(Subcommand)]
enum GenerateTarget {
    /// Generate an image using FLUX, Seedream, Ideogram, Nano Banana, etc.
    Image {
        /// Prompt description
        #[arg(short, long)]
        prompt: String,

        /// Model ID
        #[arg(short, long, default_value = "flux-dev")]
        model: String,

        /// Aspect ratio (16:9, 9:16, 1:1, 4:3, 21:9)
        #[arg(short, long, default_value = "16:9")]
        ratio: String,

        /// Negative prompt
        #[arg(long)]
        negative: Option<String>,

        /// Seed (-1 for random)
        #[arg(long, default_value_t = -1)]
        seed: i64,

        /// Conditioning starting image for Image-to-Image / Inpainting
        #[arg(long)]
        image_url: Option<String>,
    },

    /// Generate a video using Kling, Sora 2, Veo 3, Wan 2.6, Seedance, etc.
    Video {
        /// Video prompt description
        #[arg(short, long)]
        prompt: String,

        /// Video model ID
        #[arg(short, long, default_value = "kling-v3-pro")]
        model: String,

        /// Duration in seconds (5, 10, 12)
        #[arg(short, long, default_value_t = 5)]
        duration: u32,

        /// Resolution (720p, 1080p, 4K)
        #[arg(short, long, default_value = "1080p")]
        resolution: String,

        /// Starting image for Image-to-Video
        #[arg(long)]
        image_url: Option<String>,

        /// Generate synchronized audio
        #[arg(long, default_value_t = true)]
        audio: bool,

        /// High bitrate mode
        #[arg(long, default_value_t = true)]
        high_bitrate: bool,
    },

    /// Generate music or speech audio
    Audio {
        /// Prompt describing music genre, mood, or speech narration text
        #[arg(short, long)]
        prompt: String,

        /// Audio model ID (text-to-music-v2, elevenlabs-voice-tts)
        #[arg(short, long, default_value = "text-to-music-v2")]
        model: String,
    },
}

#[derive(Subcommand)]
enum WorkflowAction {
    /// List built-in workflow pipeline templates
    Templates,

    /// Execute a workflow JSON/YAML file or built-in template
    Run {
        /// Path to workflow JSON/YAML file (or specify --template)
        file: Option<PathBuf>,

        /// Built-in template ID (ai-influencer, popcorn-storyboard)
        #[arg(short, long)]
        template: Option<String>,

        /// Workflow inputs as key=value pairs (e.g. --input prompt="cyberpunk city")
        #[arg(short, long, value_parser = parse_key_val)]
        input: Vec<(String, String)>,
    },
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid KEY=VALUE format: no '=' found in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let cli = Cli::parse();

    // Select provider
    let provider: Arc<dyn Provider> = if cli.mock {
        Arc::new(MockProvider::new())
    } else {
        Arc::new(MuapiClient::new(cli.api_key))
    };

    match cli.command {
        Commands::Models {
            id,
            category,
            search,
            json,
        } => {
            if let Some(model_id) = id {
                match get_model(&model_id) {
                    Some(m) => {
                        if json {
                            println!("{}", serde_json::to_string_pretty(m)?);
                        } else {
                            println!("\n\x1b[1;36m{}\x1b[0m (\x1b[33m{}\x1b[0m)", m.name, m.id);
                            println!("\x1b[90mCategory:\x1b[0m {}", m.category);
                            println!("\x1b[90mProvider:\x1b[0m {}", m.provider);
                            println!("\x1b[90mEndpoint:\x1b[0m {}", m.endpoint);
                            println!("\x1b[90mCost/run:\x1b[0m ${:.3}", m.cost_per_run);
                            println!("\x1b[90mDescription:\x1b[0m {}", m.description);
                            println!(
                                "\x1b[90mAspect Ratios:\x1b[0m {:?}",
                                m.capabilities.aspect_ratios
                            );
                            println!(
                                "\x1b[90mResolutions:\x1b[0m {:?}",
                                m.capabilities.resolutions
                            );
                            println!("\x1b[90mLocal Available:\x1b[0m {}\n", m.is_local_available);
                        }
                    }
                    None => {
                        eprintln!("Error: Model '{model_id}' not found in catalog.");
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }

            let mut list: Vec<&'static open_generative_ai::types::ModelInfo> =
                MODEL_CATALOG.iter().collect();

            if let Some(cat_str) = category {
                let cat = match cat_str.to_lowercase().as_str() {
                    "image" | "text-to-image" => Some(ModelCategory::TextToImage),
                    "i2i" | "image-to-image" => Some(ModelCategory::ImageToImage),
                    "video" | "text-to-video" => Some(ModelCategory::TextToVideo),
                    "i2v" | "image-to-video" => Some(ModelCategory::ImageToVideo),
                    "v2v" | "video-to-video" => Some(ModelCategory::VideoToVideo),
                    "lipsync" => Some(ModelCategory::LipSync),
                    "recast" => Some(ModelCategory::Recast),
                    "audio" => Some(ModelCategory::Audio),
                    _ => None,
                };
                if let Some(c) = cat {
                    list = models_by_category(c);
                }
            }

            if let Some(q) = search {
                list = search_models(&q);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else {
                println!(
                    "\n\x1b[1;35mOpen Generative AI Model Catalog\x1b[0m ({} models)\n",
                    list.len()
                );
                for m in list {
                    println!(
                        " • \x1b[1m{:<24}\x1b[0m {:<16} \x1b[33m${:<5.3}\x1b[0m \x1b[90m{}\x1b[0m",
                        m.id, m.category, m.cost_per_run, m.name
                    );
                }
                println!();
            }
        }

        Commands::Generate { target } => match target {
            GenerateTarget::Image {
                prompt,
                model,
                ratio,
                negative,
                seed,
                image_url,
            } => {
                let req = GenerationRequest {
                    model,
                    prompt: Some(prompt),
                    aspect_ratio: Some(ratio),
                    negative_prompt: negative,
                    seed: if seed >= 0 { Some(seed) } else { None },
                    image_url,
                    ..Default::default()
                };

                println!("Submitting image generation request...");
                let result = provider.generate_and_wait(&req).await?;
                println!("\n\x1b[1;32m✓ Generation Completed!\x1b[0m");
                println!("Request ID: {}", result.request_id);
                for url in &result.outputs {
                    println!("Output URL: \x1b[1;34m{}\x1b[0m", url);
                }
            }

            GenerateTarget::Video {
                prompt,
                model,
                duration,
                resolution,
                image_url,
                audio,
                high_bitrate,
            } => {
                let req = GenerationRequest {
                    model,
                    prompt: Some(prompt),
                    duration: Some(duration),
                    resolution: Some(resolution),
                    image_url,
                    generate_audio: Some(audio),
                    high_bitrate: Some(high_bitrate),
                    ..Default::default()
                };

                println!("Submitting video generation request...");
                let result = provider.generate_and_wait(&req).await?;
                println!("\n\x1b[1;32m✓ Video Generation Completed!\x1b[0m");
                println!("Request ID: {}", result.request_id);
                for url in &result.outputs {
                    println!("Video URL: \x1b[1;34m{}\x1b[0m", url);
                }
            }

            GenerateTarget::Audio { prompt, model } => {
                let req = GenerationRequest {
                    model,
                    prompt: Some(prompt),
                    ..Default::default()
                };

                println!("Submitting audio generation request...");
                let result = provider.generate_and_wait(&req).await?;
                println!("\n\x1b[1;32m✓ Audio Generation Completed!\x1b[0m");
                println!("Request ID: {}", result.request_id);
                for url in &result.outputs {
                    println!("Audio URL: \x1b[1;34m{}\x1b[0m", url);
                }
            }
        },

        Commands::Lipsync {
            image,
            audio,
            model,
        } => {
            let req = GenerationRequest {
                model,
                image_url: Some(image),
                audio_url: Some(audio),
                ..Default::default()
            };

            println!("Submitting lip-sync generation request...");
            let result = provider.generate_and_wait(&req).await?;
            println!("\n\x1b[1;32m✓ Lip Sync Completed!\x1b[0m");
            println!("Request ID: {}", result.request_id);
            for url in &result.outputs {
                println!("Output Video: \x1b[1;34m{}\x1b[0m", url);
            }
        }

        Commands::Recast {
            image,
            character,
            model,
        } => {
            let req = GenerationRequest {
                model,
                image_url: Some(image),
                images_list: vec![character],
                ..Default::default()
            };

            println!("Submitting recast generation request...");
            let result = provider.generate_and_wait(&req).await?;
            println!("\n\x1b[1;32m✓ Recast Completed!\x1b[0m");
            for url in &result.outputs {
                println!("Recast Output: \x1b[1;34m{}\x1b[0m", url);
            }
        }

        Commands::Workflow { action } => match action {
            WorkflowAction::Templates => {
                println!("\n\x1b[1;35mBuilt-In Pipeline Templates\x1b[0m\n");
                let templates = [ai_influencer_template(), popcorn_storyboard_template()];
                for t in templates {
                    println!(" • \x1b[1;36m{}\x1b[0m: {}", t.id, t.name);
                    println!("   \x1b[90m{}\x1b[0m", t.description);
                    println!("   \x1b[90mSteps:\x1b[0m {}\n", t.steps.len());
                }
            }

            WorkflowAction::Run {
                file,
                template,
                input,
            } => {
                let wf = match (file, template) {
                    (Some(path), _) => {
                        let content = fs::read_to_string(&path)?;
                        if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                            || path.extension().and_then(|e| e.to_str()) == Some("yml")
                        {
                            serde_yaml::from_str::<Workflow>(&content)?
                        } else {
                            serde_json::from_str::<Workflow>(&content)?
                        }
                    }
                    (None, Some(tid)) => match tid.as_str() {
                        "ai-influencer" => ai_influencer_template(),
                        "popcorn-storyboard" => popcorn_storyboard_template(),
                        other => {
                            eprintln!("Unknown template '{other}'. Use `open-genai workflow templates` to list.");
                            std::process::exit(1);
                        }
                    },
                    (None, None) => {
                        eprintln!("Error: specify either a workflow file path or --template.");
                        std::process::exit(1);
                    }
                };

                let inputs_map: BTreeMap<String, String> = input.into_iter().collect();
                let engine = WorkflowEngine::new(provider);

                println!("Executing workflow '{}'...", wf.name);
                let report = engine.run(&wf, &inputs_map).await?;

                println!("\n\x1b[1;32m✓ Workflow Execution Report\x1b[0m");
                println!("Status: {}", report.status);
                println!("Total Duration: {:.2}s", report.total_duration_seconds);
                for (id, step) in &report.steps {
                    println!(
                        " • Step '{id}' [{}]: {:.2}s -> {:?}",
                        step.status, step.duration_seconds, step.output_url
                    );
                }
                if let Some(final_url) = report.final_output_url {
                    println!("\nFinal Output: \x1b[1;34m{}\x1b[0m", final_url);
                }
            }
        },

        Commands::Serve { host, port } => {
            let router = create_router(provider);
            let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
            println!("\n\x1b[1;36m🦀 Open Generative AI Studio (Rust)\x1b[0m");
            println!(
                " • REST API:    \x1b[1;32mhttp://{}:{}/api/v1\x1b[0m",
                host, port
            );
            println!(
                " • Web Studio:  \x1b[1;34mhttp://{}:{}/studio\x1b[0m",
                host, port
            );
            println!(" • Health:      http://{}:{}/health\n", host, port);

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        }
    }

    Ok(())
}
