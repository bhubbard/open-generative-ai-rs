pub mod error;
pub mod providers;
pub mod registry;
pub mod server;
pub mod types;
pub mod workflow;

pub use error::{OpenGenAiError, Result};
pub use providers::{MockProvider, MuapiClient, Provider};
pub use registry::{get_model, local_models, models_by_category, search_models, MODEL_CATALOG};
pub use server::create_router;
pub use types::*;
pub use workflow::{
    ai_influencer_template, popcorn_storyboard_template, Workflow, WorkflowEngine, WorkflowReport,
    WorkflowStep,
};
