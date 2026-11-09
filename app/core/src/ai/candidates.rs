//! Model output is bounded text suggestions. It cannot introduce executable authority.
use super::{
    context::Bundle,
    template::{Message, Template},
};
use crate::{Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiCandidate")]
pub struct Candidate {
    pub message: Message,
    pub rationale: String,
    pub summary: String,
    pub concerns: Vec<String>,
    /// Model-reported attribution, not independent proof of semantic analysis.
    pub covered_chunks: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    candidates: Vec<Candidate>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiValidatedCandidate")]
pub struct Validated {
    pub candidate: Candidate,
    pub valid: bool,
    pub validation_errors: Vec<String>,
    pub unreported_chunks: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiCandidateSet")]
pub struct CandidateSet {
    pub model: String,
    pub elapsed_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub context_fingerprint: String,
    pub snapshot_fingerprint: String,
    pub context_complete: bool,
    pub candidates: Vec<Validated>,
    pub warnings: Vec<String>,
}
#[derive(Clone)]
pub(crate) struct Rules {
    pub template: Template,
    pub context: Bundle,
    pub snapshot: String,
    pub profile_id: String,
    pub profile_revision: String,
}
fn invalid() -> Error {
    Error::with_kind(ErrorKind::InvalidData,"AI candidate response is malformed, unsafe, duplicated or outside the approved output schema")
}
fn safe(value: &str, max: usize) -> bool {
    value.len()<=max && !value.chars().any(|c| (c.is_control() && c!='\n' && c!='\t') || matches!(c,'\u{061c}'|'\u{200e}'|'\u{200f}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'))
}
pub(crate) fn chunk_ids(context: &Bundle) -> Result<BTreeSet<String>> {
    let value: serde_json::Value = serde_json::from_str(&context.payload)?;
    value["chunks"]
        .as_array()
        .ok_or_else(invalid)?
        .iter()
        .map(|c| c["id"].as_str().map(str::to_owned).ok_or_else(invalid))
        .collect()
}
impl Rules {
    pub(crate) fn validate(
        &self,
        completion: &super::provider::Completion,
    ) -> Result<CandidateSet> {
        let text = &completion.content;
        if text.len() > 65536 {
            return Err(invalid());
        }
        let reply: Reply = serde_json::from_str(text).map_err(|_| invalid())?;
        if reply.candidates.is_empty() || reply.candidates.len() > 3 {
            return Err(invalid());
        }
        let expected = chunk_ids(&self.context)?;
        let mut distinct = BTreeSet::new();
        let mut candidates = Vec::new();
        for candidate in reply.candidates {
            if candidate.rationale.trim().is_empty()
                || candidate.summary.trim().is_empty()
                || !safe(&candidate.rationale, 4096)
                || !safe(&candidate.summary, 4096)
                || candidate.concerns.len() > 16
                || candidate.concerns.iter().any(|c| !safe(c, 1024))
                || candidate.covered_chunks.len() > 8192
            {
                return Err(invalid());
            }
            let normalized = candidate
                .message
                .text()
                .split_whitespace()
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
                .join(" ");
            if !distinct.insert(normalized) {
                return Err(invalid());
            }
            let claimed = candidate
                .covered_chunks
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if claimed.len() != candidate.covered_chunks.len() || !claimed.is_subset(&expected) {
                return Err(invalid());
            }
            let validation_errors = self
                .template
                .check_message(&candidate.message)
                .err()
                .map(|e| vec![e.to_string()])
                .unwrap_or_default();
            let unreported_chunks = expected.difference(&claimed).cloned().collect();
            candidates.push(Validated {
                valid: validation_errors.is_empty(),
                candidate,
                validation_errors,
                unreported_chunks,
            });
        }
        let mut warnings = self.context.warnings.clone();
        warnings.push("Coverage IDs and rationale are model reports, not verified facts. Review accuracy and meaningful differences; no confidence score or write authority is inferred.".into());
        Ok(CandidateSet {
            model: completion.model.clone(),
            elapsed_ms: completion.elapsed_ms,
            input_tokens: completion.input_tokens,
            output_tokens: completion.output_tokens,
            context_fingerprint: self.context.fingerprint.clone(),
            snapshot_fingerprint: self.snapshot.clone(),
            context_complete: self.context.complete,
            candidates,
            warnings,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rules() -> Rules {
        Rules {
            template: Template::default(),
            context: super::super::context::prepare(
                vec![super::super::context::Fragment {
                    kind: super::super::context::Kind::SelectedDiff,
                    origin: "x".into(),
                    bytes: b"+change\n".to_vec(),
                }],
                &Default::default(),
            )
            .unwrap(),
            snapshot: "snapshot".into(),
            profile_id: "profile".into(),
            profile_revision: "revision".into(),
        }
    }
    impl Rules {
        fn text(&self, text: &str) -> Result<CandidateSet> {
            self.validate(&super::super::provider::Completion {
                model: "fixture-model".into(),
                content: text.into(),
                elapsed_ms: 1,
                input_tokens: None,
                output_tokens: None,
            })
        }
    }
    fn reply(subject: &str) -> serde_json::Value {
        serde_json::json!({"candidates":[{"message":{"subject":subject,"body":""},"rationale":"Describe the change", "summary":"One selected change", "concerns":[],"covered_chunks":["f1.1"]}]})
    }
    #[test]
    fn validates_message_rules_and_preserves_model_attribution_without_authority() {
        let r = rules();
        let good = r.text(&reply("fix: preserve changes").to_string()).unwrap();
        assert!(good.candidates[0].valid);
        assert!(good.candidates[0].unreported_chunks.is_empty());
        assert_eq!(good.snapshot_fingerprint, "snapshot");
        let bad = r
            .text(&reply("invalid conventional format").to_string())
            .unwrap();
        assert!(!bad.candidates[0].valid);
        assert!(!bad.candidates[0].validation_errors.is_empty());
    }
    #[test]
    fn rejects_injected_commands_unknown_provenance_duplicates_and_excess_candidates() {
        let r = rules();
        let mut value = reply("fix: safe text");
        value["commands"] = serde_json::json!(["git push"]);
        assert!(r.text(&value.to_string()).is_err());
        let mut value = reply("fix: safe text");
        value["candidates"][0]["covered_chunks"] = serde_json::json!(["invented"]);
        assert!(r.text(&value.to_string()).is_err());
        let mut value = reply("fix: safe text");
        let c = value["candidates"][0].clone();
        value["candidates"].as_array_mut().unwrap().push(c);
        assert!(r.text(&value.to_string()).is_err());
        assert!(r.text("{\"candidates\":[]}").is_err());
        assert!(r.text("```json\n{}\n```").is_err());
    }
    #[test]
    fn incomplete_attribution_is_explicit_and_controls_cannot_enter_candidate_metadata() {
        let r = rules();
        let mut value = reply("fix: safe text");
        value["candidates"][0]["covered_chunks"] = serde_json::json!([]);
        let result = r.text(&value.to_string()).unwrap();
        assert_eq!(result.candidates[0].unreported_chunks, vec!["f1.1"]);
        value["candidates"][0]["rationale"] = serde_json::json!("unsafe\u{202e}text");
        assert!(r.text(&value.to_string()).is_err());
    }
}
