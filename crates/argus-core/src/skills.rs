//! Procedural skill memory for Argus.
//!
//! Skills are reusable, documented procedures retrieved semantically before each turn
//! and injected into the system prompt as background guidance.
//! They encode HOW to do things well — complementing declarative memory which stores WHAT.
//!
//! The instance changes. The accumulated competence doesn't.

use serde::{Deserialize, Serialize};
use crate::embedding::EmbeddingClient;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub skill_name: String,
    pub trigger_description: String,
    pub procedure_steps: String,
    pub times_used: i32,
    pub success_rate: f64,
    pub similarity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSkill {
    pub skill_name: String,
    pub trigger_description: String,
    pub procedure_steps: String,
    pub model_created_by: String,
    pub metadata: Option<serde_json::Value>,
}

// ── Client ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SkillsClient {
    embedding: EmbeddingClient,
}

impl SkillsClient {
    pub fn new(embedding: EmbeddingClient) -> Self {
        Self { embedding }
    }

    /// Format a Vec<f32> embedding as a Postgres vector literal: "[x,y,z,...]"
    fn vec_to_pg(v: &[f32]) -> String {
        format!(
            "[{}]",
            v.iter().map(|f| format!("{:.8}", f)).collect::<Vec<_>>().join(",")
        )
    }

    /// Search for skills relevant to the current user message.
    /// Returns empty vec for short queries — not worth the embedding round-trip.
    /// Called at the start of every agent turn alongside the memory prefetch.
    pub async fn search_relevant(
        &self,
        query: &str,
        threshold: f64,
        limit: usize,
    ) -> Result<Vec<Skill>, String> {
        if query.split_whitespace().count() < 5 {
            return Ok(vec![]);
        }
        let vector = self.embedding
            .embed(query)
            .await
            .map_err(|e| format!("Skill embed failed: {}", e))?;

        let body = serde_json::json!({
            "query_embedding": Self::vec_to_pg(&vector),
            "query_text": query,
            "match_threshold": threshold,
            "match_count": limit
        });

        let result = self.embedding
            .supabase_rpc("search_skills", &body)
            .await
            .map_err(|e| format!("Skill search RPC failed: {}", e))?;

        serde_json::from_value(result)
            .map_err(|e| format!("Skill parse failed: {}", e))
    }

    /// Store a new skill with its embedding.
    /// Called from the background reflection task after tool-heavy turns.
    pub async fn create_skill(&self, skill: NewSkill) -> Result<String, String> {
        let embed_text = format!(
            "{}\n\n{}\n\n{}",
            skill.skill_name, skill.trigger_description, skill.procedure_steps
        );
        let vector = self.embedding
            .embed(&embed_text)
            .await
            .map_err(|e| format!("Skill embed failed: {}", e))?;

        let body = serde_json::json!({
            "skill_name": skill.skill_name,
            "trigger_description": skill.trigger_description,
            "procedure_steps": skill.procedure_steps,
            "model_created_by": skill.model_created_by,
            "embedding": Self::vec_to_pg(&vector),
            "metadata": skill.metadata,
            "times_used": 0,
            "success_rate": 1.0
        });

        self.embedding
            .supabase_insert("argus_skills", &body)
            .await
            .map_err(|e| format!("Skill insert failed: {}", e))?;

        Ok(skill.skill_name)
    }

    /// Record that a skill was used and optionally update its procedure.
    pub async fn record_usage(
        &self,
        skill_id: &str,
        success: bool,
        refined_steps: Option<&str>,
    ) -> Result<(), String> {
        let body = serde_json::json!({
            "skill_id": skill_id,
            "success": success,
            "refined_steps": refined_steps
        });
        self.embedding
            .supabase_rpc("update_skill_usage", &body)
            .await
            .map_err(|e| format!("Skill usage update failed: {}", e))?;
        Ok(())
    }

    /// Format retrieved skills for injection into the system prompt.
    /// Skills are guidance, not commands — the model reads and decides how to apply them.
    pub fn format_for_prompt(skills: &[Skill]) -> String {
        if skills.is_empty() {
            return String::new();
        }

        let mut out = String::from(
            "## Relevant Procedural Skills\n\
             Documented procedures that have worked well for similar tasks.\n\
             Use as guidance — adapt as the situation warrants.\n\n",
        );

        for skill in skills {
            let confidence = match skill.success_rate {
                r if r >= 0.9 => "battle-tested",
                r if r >= 0.7 => "reliable",
                _ => "experimental",
            };
            out.push_str(&format!(
                "### {} ({})\n**When:** {}\n\n{}\n\n---\n\n",
                skill.skill_name, confidence,
                skill.trigger_description, skill.procedure_steps
            ));
        }

        out
    }
}
