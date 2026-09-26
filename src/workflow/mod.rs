use crate::error::{OpenGenAiError, Result};
use crate::providers::Provider;
use crate::types::GenerationRequest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// A step in a generative AI workflow DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub model_id: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, serde_json::Value>,
}

/// A complete multi-step generation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

/// Status of an individual step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub model_id: String,
    pub status: String,
    pub output_url: Option<String>,
    pub outputs: Vec<String>,
    pub duration_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Complete execution report of a workflow run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowReport {
    pub workflow_id: String,
    pub status: String,
    pub total_duration_seconds: f64,
    pub steps: BTreeMap<String, StepExecutionResult>,
    pub final_output_url: Option<String>,
}

pub struct WorkflowEngine {
    provider: Arc<dyn Provider>,
}

impl WorkflowEngine {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    /// Validates DAG integrity: verifies all dependencies exist and checks for circular references
    pub fn validate(workflow: &Workflow) -> Result<()> {
        let step_ids: HashSet<&str> = workflow.steps.iter().map(|s| s.id.as_str()).collect();

        // 1. Verify existence of dependencies
        for step in &workflow.steps {
            for dep in &step.depends_on {
                if !step_ids.contains(dep.as_str()) {
                    return Err(OpenGenAiError::WorkflowError(format!(
                        "Step '{}' depends on non-existent step '{}'",
                        step.id, dep
                    )));
                }
                if dep == &step.id {
                    return Err(OpenGenAiError::WorkflowError(format!(
                        "Step '{}' cannot depend on itself",
                        step.id
                    )));
                }
            }
        }

        // 2. Cycle detection via Kahn's algorithm
        let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
        let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

        for step in &workflow.steps {
            in_degree.insert(&step.id, step.depends_on.len());
            for dep in &step.depends_on {
                adj.entry(dep.as_str()).or_default().push(&step.id);
            }
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut visited_count = 0;
        while let Some(u) = queue.pop() {
            visited_count += 1;
            if let Some(neighbors) = adj.get(u) {
                for &v in neighbors {
                    let deg = in_degree.get_mut(v).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(v);
                    }
                }
            }
        }

        if visited_count != workflow.steps.len() {
            return Err(OpenGenAiError::WorkflowError(
                "Cycle detected in workflow pipeline dependencies".into(),
            ));
        }

        Ok(())
    }

    /// Substitutes `{{steps.<id>.output}}` and `{{inputs.<key>}}` variables in string parameters
    fn substitute_variables(
        val: &serde_json::Value,
        inputs: &BTreeMap<String, String>,
        outputs: &BTreeMap<String, String>,
    ) -> serde_json::Value {
        match val {
            serde_json::Value::String(s) => {
                let mut resolved = s.clone();
                for (k, v) in inputs {
                    let pattern = format!("{{{{inputs.{}}}}}", k);
                    resolved = resolved.replace(&pattern, v);
                }
                for (step_id, out) in outputs {
                    let pattern = format!("{{{{steps.{}.output}}}}", step_id);
                    resolved = resolved.replace(&pattern, out);
                }
                serde_json::Value::String(resolved)
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), Self::substitute_variables(v, inputs, outputs));
                }
                serde_json::Value::Object(new_map)
            }
            serde_json::Value::Array(arr) => {
                let new_arr = arr
                    .iter()
                    .map(|v| Self::substitute_variables(v, inputs, outputs))
                    .collect();
                serde_json::Value::Array(new_arr)
            }
            other => other.clone(),
        }
    }

    /// Execute the complete workflow DAG
    pub async fn run(
        &self,
        workflow: &Workflow,
        inputs: &BTreeMap<String, String>,
    ) -> Result<WorkflowReport> {
        Self::validate(workflow)?;

        let start_time = Instant::now();
        info!(target: "workflow", "Starting execution of workflow '{}'", workflow.name);

        let mut completed_outputs: BTreeMap<String, String> = BTreeMap::new();
        let mut step_results: BTreeMap<String, StepExecutionResult> = BTreeMap::new();
        let mut pending_steps: Vec<WorkflowStep> = workflow.steps.clone();

        while !pending_steps.is_empty() {
            // Find steps whose dependencies have all completed
            let (ready_steps, remaining): (Vec<WorkflowStep>, Vec<WorkflowStep>) =
                pending_steps.into_iter().partition(|s| {
                    s.depends_on
                        .iter()
                        .all(|d| completed_outputs.contains_key(d))
                });

            if ready_steps.is_empty() {
                return Err(OpenGenAiError::WorkflowError(
                    "Deadlock detected: no ready steps found with satisfied dependencies".into(),
                ));
            }

            pending_steps = remaining;

            // Execute ready steps (can run in parallel)
            for step in ready_steps {
                let step_start = Instant::now();
                info!(target: "workflow", "Running step '{}' ({}) with model '{}'", step.id, step.name, step.model_id);

                // Prepare parameters with variable substitution
                let substituted_params = Self::substitute_variables(
                    &serde_json::Value::Object(step.params.clone().into_iter().collect()),
                    inputs,
                    &completed_outputs,
                );

                // Convert parameters into GenerationRequest
                let mut gen_req = GenerationRequest {
                    model: step.model_id.clone(),
                    ..Default::default()
                };

                if let Some(p) = substituted_params.get("prompt").and_then(|v| v.as_str()) {
                    gen_req.prompt = Some(p.to_string());
                }
                if let Some(ratio) = substituted_params
                    .get("aspect_ratio")
                    .and_then(|v| v.as_str())
                {
                    gen_req.aspect_ratio = Some(ratio.to_string());
                }
                if let Some(img) = substituted_params.get("image_url").and_then(|v| v.as_str()) {
                    gen_req.image_url = Some(img.to_string());
                }
                if let Some(audio) = substituted_params.get("audio_url").and_then(|v| v.as_str()) {
                    gen_req.audio_url = Some(audio.to_string());
                }
                if let Some(video) = substituted_params.get("video_url").and_then(|v| v.as_str()) {
                    gen_req.video_url = Some(video.to_string());
                }
                if let Some(dur) = substituted_params.get("duration").and_then(|v| v.as_u64()) {
                    gen_req.duration = Some(dur as u32);
                }

                // Execute via provider
                let exec_res = self.provider.generate_and_wait(&gen_req).await;
                let duration = step_start.elapsed().as_secs_f64();

                match exec_res {
                    Ok(pred) => {
                        let primary = pred
                            .primary_url
                            .clone()
                            .or_else(|| pred.outputs.first().cloned());
                        if let Some(url) = &primary {
                            completed_outputs.insert(step.id.clone(), url.clone());
                        }

                        step_results.insert(
                            step.id.clone(),
                            StepExecutionResult {
                                step_id: step.id,
                                model_id: step.model_id,
                                status: "completed".into(),
                                output_url: primary,
                                outputs: pred.outputs,
                                duration_seconds: duration,
                                error: None,
                            },
                        );
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        warn!(target: "workflow", "Step '{}' failed: {err_msg}", step.id);
                        step_results.insert(
                            step.id.clone(),
                            StepExecutionResult {
                                step_id: step.id.clone(),
                                model_id: step.model_id,
                                status: "failed".into(),
                                output_url: None,
                                outputs: vec![],
                                duration_seconds: duration,
                                error: Some(err_msg.clone()),
                            },
                        );

                        return Ok(WorkflowReport {
                            workflow_id: workflow.id.clone(),
                            status: "failed".into(),
                            total_duration_seconds: start_time.elapsed().as_secs_f64(),
                            steps: step_results,
                            final_output_url: None,
                        });
                    }
                }
            }
        }

        let total_duration = start_time.elapsed().as_secs_f64();
        let last_step = workflow.steps.last().map(|s| s.id.as_str());
        let final_url = last_step.and_then(|id| completed_outputs.get(id).cloned());

        Ok(WorkflowReport {
            workflow_id: workflow.id.clone(),
            status: "completed".into(),
            total_duration_seconds: total_duration,
            steps: step_results,
            final_output_url: final_url,
        })
    }
}

// ── Built-in Canonical Templates ──────────────────────────────────────────

/// AI Influencer Pipeline: Character concept -> Consistent Recast -> Video Motion -> Voiceover -> LipSync
pub fn ai_influencer_template() -> Workflow {
    Workflow {
        id: "ai-influencer".into(),
        name: "AI Influencer Studio".into(),
        description: "Creates consistent character visual, choreographs natural movement, records neural speech, and produces a synchronized talking video.".into(),
        steps: vec![
            WorkflowStep {
                id: "concept_portrait".into(),
                name: "Generate Character Anchor".into(),
                model_id: "flux-dev".into(),
                depends_on: vec![],
                params: [
                    ("prompt".into(), serde_json::json!("A high-fashion professional influencer portrait, 35mm photograph, soft natural lighting, {{inputs.prompt}}")),
                    ("aspect_ratio".into(), serde_json::json!("9:16")),
                ].into_iter().collect(),
            },
            WorkflowStep {
                id: "voiceover".into(),
                name: "Synthesize Voiceover".into(),
                model_id: "elevenlabs-voice-tts".into(),
                depends_on: vec![],
                params: [
                    ("prompt".into(), serde_json::json!("{{inputs.speech_script}}")),
                ].into_iter().collect(),
            },
            WorkflowStep {
                id: "talking_video".into(),
                name: "Lip-Sync Talking Head".into(),
                model_id: "infinite-talk-i2v".into(),
                depends_on: vec!["concept_portrait".into(), "voiceover".into()],
                params: [
                    ("image_url".into(), serde_json::json!("{{steps.concept_portrait.output}}")),
                    ("audio_url".into(), serde_json::json!("{{steps.voiceover.output}}")),
                ].into_iter().collect(),
            },
        ],
    }
}

/// Popcorn Storyboarding Pipeline: Script -> Concept Shot -> Video Motion -> BGM
pub fn popcorn_storyboard_template() -> Workflow {
    Workflow {
        id: "popcorn-storyboard".into(),
        name: "Popcorn Storyboard Studio".into(),
        description: "Turns narrative scenes into cinematic visual frames with dynamic camera movement and synchronized background score.".into(),
        steps: vec![
            WorkflowStep {
                id: "keyframe".into(),
                name: "Cinematic Scene Keyframe".into(),
                model_id: "seedream-5.0".into(),
                depends_on: vec![],
                params: [
                    ("prompt".into(), serde_json::json!("Cinematic 70mm movie still, {{inputs.scene_prompt}}, epic atmospheric volume")),
                    ("aspect_ratio".into(), serde_json::json!("16:9")),
                ].into_iter().collect(),
            },
            WorkflowStep {
                id: "scene_motion".into(),
                name: "Cinematic Camera Motion".into(),
                model_id: "kling-v2.1-i2v".into(),
                depends_on: vec!["keyframe".into()],
                params: [
                    ("image_url".into(), serde_json::json!("{{steps.keyframe.output}}")),
                    ("prompt".into(), serde_json::json!("Slow cinematic pan, dramatic lighting, high depth of field")),
                    ("duration".into(), serde_json::json!(5)),
                ].into_iter().collect(),
            },
            WorkflowStep {
                id: "background_music".into(),
                name: "Original Soundtrack".into(),
                model_id: "text-to-music-v2".into(),
                depends_on: vec![],
                params: [
                    ("prompt".into(), serde_json::json!("Cinematic atmospheric soundtrack, {{inputs.music_genre}}")),
                ].into_iter().collect(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::MockProvider;

    #[tokio::test]
    async fn test_workflow_validation() {
        let wf = ai_influencer_template();
        assert!(WorkflowEngine::validate(&wf).is_ok());

        // Introduce cycle
        let mut bad_wf = wf.clone();
        bad_wf.steps[0].depends_on = vec!["talking_video".into()];
        assert!(WorkflowEngine::validate(&bad_wf).is_err());
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let provider = Arc::new(MockProvider::new());
        let engine = WorkflowEngine::new(provider);
        let wf = ai_influencer_template();

        let mut inputs = BTreeMap::new();
        inputs.insert("prompt".into(), "smiling tech founder in studio".into());
        inputs.insert("speech_script".into(), "Welcome to our AI platform!".into());

        let report = engine.run(&wf, &inputs).await.expect("run workflow");
        assert_eq!(report.status, "completed");
        assert_eq!(report.steps.len(), 3);
        assert!(report.final_output_url.is_some());
    }
}
