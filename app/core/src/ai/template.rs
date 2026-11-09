//! Structured message rules and deterministic static previews, with no model execution.
use crate::{Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AiStyle")]
pub enum Style {
    Plain,
    Conventional,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiRules")]
pub struct Rules {
    pub style: Style,
    pub subject_max_chars: usize,
    pub allowed_types: Vec<String>,
    pub allow_scope: bool,
    pub forbid_trailing_period: bool,
    pub instructions: String,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            style: Style::Conventional,
            subject_max_chars: 72,
            allowed_types: ["feat", "fix", "docs", "refactor", "test", "chore", "build", "ci", "perf", "style", "revert"].map(str::to_string).to_vec(),
            allow_scope: true,
            forbid_trailing_period: true,
            instructions: "Use a concise subject and explain why in the optional body. Do not invent tests or changes.".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiTemplate")]
pub struct Template {
    pub id: String,
    pub revision: String,
    pub name: String,
    pub rules: Rules,
    pub examples: Vec<String>,
}
impl Default for Template {
    fn default() -> Self {
        Self {
            id: "00000000-0000-4000-8000-000000000001".into(),
            revision: "00000000-0000-4000-8000-000000000001".into(),
            name: "Conventional commits".into(),
            rules: Rules::default(),
            examples: vec!["fix: preserve selected repository changes".into()],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiTemplateDraft")]
pub struct TemplateDraft {
    pub id: Option<String>,
    pub name: String,
    pub rules: Rules,
    pub examples: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiMessage")]
pub struct Message {
    pub subject: String,
    pub body: String,
}
impl Message {
    pub fn text(&self) -> String {
        if self.body.is_empty() {
            format!("{}\n", self.subject)
        } else {
            format!("{}\n\n{}\n", self.subject, self.body.trim_end_matches('\n'))
        }
    }
}
impl Template {
    pub fn validate(&self) -> Result<()> {
        uuid::Uuid::parse_str(&self.id).map_err(|_| invalid("Invalid template identifier"))?;
        uuid::Uuid::parse_str(&self.revision).map_err(|_| invalid("Invalid template revision"))?;
        if self.name.is_empty()
            || self.name.len() > 80
            || self.name.trim() != self.name
            || self.name.chars().any(char::is_control)
        {
            return Err(invalid("Invalid AI template name"));
        }
        if !(1..=200).contains(&self.rules.subject_max_chars)
            || self.rules.instructions.len() > 16384
            || self.rules.instructions.contains('\0')
            || self.examples.len() > 8
        {
            return Err(invalid("Template rules exceed limits"));
        }
        if self.rules.style == Style::Conventional
            && (self.rules.allowed_types.is_empty() || self.rules.allowed_types.len() > 32)
        {
            return Err(invalid("Conventional template needs 1–32 allowed types"));
        }
        let mut types = std::collections::BTreeSet::new();
        for t in &self.rules.allowed_types {
            if t.is_empty()
                || t.len() > 24
                || !t.bytes().all(|c| c.is_ascii_lowercase() || c == b'-')
                || !types.insert(t)
            {
                return Err(invalid("Invalid or duplicate conventional type"));
            }
            if self.rules.style == Style::Conventional
                && self.rules.subject_max_chars <= t.len() + 2
            {
                return Err(invalid("Subject limit leaves no room for a summary"));
            }
        }
        for example in &self.examples {
            if example.len() > 2048 {
                return Err(invalid("Template example exceeds limit"));
            }
            let (subject, body) = example.split_once("\n\n").unwrap_or((example, ""));
            self.check_message(&Message {
                subject: subject.into(),
                body: body.into(),
            })?;
        }
        Ok(())
    }
    pub fn check_message(&self, message: &Message) -> Result<()> {
        let subject = &message.subject;
        fn misleading(c: char) -> bool {
            matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        }
        if subject.trim() != subject
            || subject.is_empty()
            || subject.chars().count() > self.rules.subject_max_chars
            || subject.chars().any(|c| c.is_control() || misleading(c))
            || message.body.len() > 16384
            || message
                .body
                .chars()
                .any(|c| (c.is_control() && c != '\n' && c != '\t') || misleading(c))
        {
            return Err(invalid(
                "Candidate does not satisfy bounded subject/body rules",
            ));
        }
        if self.rules.forbid_trailing_period && subject.ends_with('.') {
            return Err(invalid("Template forbids a trailing subject period"));
        }
        if self.rules.style == Style::Conventional {
            let (prefix, summary) = subject
                .split_once(": ")
                .ok_or_else(|| invalid("Expected a conventional type and summary"))?;
            if summary.trim().is_empty() {
                return Err(invalid("Candidate summary cannot be empty"));
            }
            let kind = if let Some((kind, scope)) = prefix.split_once('(') {
                if !self.rules.allow_scope || !scope.ends_with(')') {
                    return Err(invalid("Template does not permit this scope"));
                }
                let scope = &scope[..scope.len() - 1];
                if scope.is_empty()
                    || scope.len() > 64
                    || !scope
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_/+.".contains(&c))
                {
                    return Err(invalid("Invalid conventional scope"));
                }
                kind
            } else {
                prefix
            };
            if !self.rules.allowed_types.iter().any(|t| t == kind) {
                return Err(invalid(
                    "Candidate type is not allowed by the selected template",
                ));
            }
        }
        Ok(())
    }
    pub fn static_preview(&self) -> Result<Message> {
        self.validate()?;
        let prefix = if self.rules.style == Style::Conventional {
            format!("{}: ", self.rules.allowed_types[0])
        } else {
            String::new()
        };
        let remaining = self.rules.subject_max_chars - prefix.chars().count();
        let summary = "update selected files"
            .chars()
            .take(remaining)
            .collect::<String>();
        let result = Message {
            subject: format!("{prefix}{summary}"),
            body: "Static sample only. No model, source transfer, Action or Git write ran.".into(),
        };
        self.check_message(&result)?;
        Ok(result)
    }
}
