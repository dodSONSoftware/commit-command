//! Optional AI metadata. Configuration and model text grant no Git/Action authority.
use crate::{
    domain::{WorkspaceId, WorktreeId},
    Error, ErrorKind, Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use url::Url;
use uuid::Uuid;
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
fn id(value: &str) -> Result<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| invalid("Invalid AI configuration identifier"))
}
fn text(value: &str, max: usize, name: &str) -> Result<()> {
    if value.len() > max || value.contains('\0') {
        return Err(invalid(name));
    }
    Ok(())
}
fn name(value: &str) -> Result<()> {
    if value.trim() != value
        || value.is_empty()
        || value.len() > 80
        || value.chars().any(char::is_control)
    {
        return Err(invalid(
            "AI names require 1–80 bytes without controls or surrounding whitespace",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AiProvider")]
pub enum Provider {
    Ollama,
    Compatible,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiConnectionDraft")]
pub struct ConnectionDraft {
    pub id: Option<String>,
    pub name: String,
    pub provider: Provider,
    pub endpoint: String,
    pub timeout_ms: u64,
    #[serde(default)]
    pub authentication: super::credentials::Authentication,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiConnection")]
pub struct Connection {
    pub id: String,
    pub revision: String,
    pub name: String,
    pub provider: Provider,
    pub endpoint: String,
    pub timeout_ms: u64,
    #[serde(default)]
    pub authentication: super::credentials::Authentication,
    pub endpoint_approval: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiParameters")]
pub struct Parameters {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub min_p: Option<f64>,
    pub context_size: Option<u32>,
    pub output_tokens: Option<u32>,
    pub seed: Option<u64>,
    pub repetition_penalty: Option<f64>,
    pub stop: Option<Vec<String>>,
}
impl Parameters {
    pub fn validate(&self, provider: Provider) -> Result<()> {
        for (value, max) in [
            (self.temperature, 2.0),
            (self.top_p, 1.0),
            (self.min_p, 1.0),
            (self.repetition_penalty, 2.0),
        ] {
            if value.is_some_and(|v| !v.is_finite() || v < 0.0 || v > max) {
                return Err(invalid("AI sampling parameter is out of bounds"));
            }
        }
        if self.top_k.is_some_and(|n| n == 0 || n > 1000)
            || self
                .context_size
                .is_some_and(|n| !(512..=131072).contains(&n))
            || self.output_tokens.is_some_and(|n| n == 0 || n > 65536)
            || self.seed.is_some_and(|n| n > 9_007_199_254_740_991)
        {
            return Err(invalid(
                "AI generation parameter exceeds its supported bound",
            ));
        }
        if provider == Provider::Compatible
            && (self.top_k.is_some()
                || self.min_p.is_some()
                || self.context_size.is_some()
                || self.repetition_penalty.is_some())
        {
            return Err(invalid("This provider adapter does not declare support for the selected additional parameters"));
        }
        if let Some(stop) = &self.stop {
            if stop.len() > 8 {
                return Err(invalid("At most eight stop strings"));
            }
            for s in stop {
                if s.is_empty() {
                    return Err(invalid("Stop strings cannot be empty"));
                }
                text(s, 256, "Stop string exceeds limit")?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiProfileDraft")]
pub struct ProfileDraft {
    pub id: Option<String>,
    pub name: String,
    pub connection: String,
    pub model: String,
    pub system_message: String,
    pub parameters: Parameters,
    pub template: String,
    pub context_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiProfile")]
pub struct Profile {
    pub id: String,
    pub revision: String,
    pub name: String,
    pub connection: String,
    pub model: String,
    pub system_message: String,
    pub parameters: Parameters,
    pub template: String,
    pub context_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(rename = "AiScope")]
pub enum Scope {
    Application,
    Workspace {
        workspace: WorkspaceId,
    },
    Repository {
        workspace: WorkspaceId,
        worktree: WorktreeId,
    },
}
impl Scope {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Application => Ok(()),
            Self::Workspace { workspace } => id(&workspace.0),
            Self::Repository {
                workspace,
                worktree,
            } => {
                id(&workspace.0)?;
                id(&worktree.0)
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiAssignment")]
pub struct Assignment {
    pub scope: Scope,
    pub profile: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiInstructions")]
pub struct Instructions {
    pub scope: Scope,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiSettings")]
pub struct Settings {
    pub version: u32,
    pub revision: String,
    pub connections: Vec<Connection>,
    pub profiles: Vec<Profile>,
    pub templates: Vec<super::template::Template>,
    pub assignments: Vec<Assignment>,
    pub instructions: Vec<Instructions>,
    pub global_concurrency: usize,
    pub connection_concurrency: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            revision: "00000000-0000-4000-8000-000000000000".into(),
            connections: vec![],
            profiles: vec![],
            templates: vec![super::template::Template::default()],
            assignments: vec![],
            instructions: vec![],
            global_concurrency: 1,
            connection_concurrency: 1,
        }
    }
}
pub fn endpoint(value: &str) -> Result<String> {
    if value.len() > 2048 || value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(invalid("Invalid AI endpoint"));
    }
    let mut u =
        Url::parse(value).map_err(|_| invalid("AI endpoint must be an absolute HTTP(S) URL"))?;
    if !["http", "https"].contains(&u.scheme())
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err(invalid(
            "AI endpoint requires HTTP(S), a host and no user information, query or fragment",
        ));
    }
    if u.host_str()
        .is_some_and(|h| ["0.0.0.0", "[::]"].contains(&h))
    {
        return Err(invalid(
            "Choose a concrete AI destination instead of an unspecified bind address",
        ));
    }
    if !u.path().ends_with('/') {
        u.set_path(&format!("{}/", u.path()));
    }
    Ok(u.to_string())
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        id(&self.revision)?;
        if self.version != 1 {
            return Err(Error::with_kind(
                ErrorKind::NewerSchema,
                "Unsupported AI settings version",
            ));
        }
        if self.connections.len() > 32
            || self.profiles.len() > 64
            || self.templates.len() > 64
            || self.assignments.len() > 1024
            || self.instructions.len() > 1024
            || !(1..=4).contains(&self.global_concurrency)
            || !(1..=2).contains(&self.connection_concurrency)
        {
            return Err(invalid("AI settings exceed supported limits"));
        }
        fn unique<'a>(values: impl Iterator<Item = (&'a str, &'a str)>) -> Result<()> {
            let mut ids = BTreeSet::new();
            let mut names = BTreeSet::new();
            for (i, n) in values {
                id(i)?;
                name(n)?;
                if !ids.insert(i) || !names.insert(n.to_lowercase()) {
                    return Err(invalid("Duplicate AI identifier or name"));
                }
            }
            Ok(())
        }
        unique(
            self.connections
                .iter()
                .map(|v| (v.id.as_str(), v.name.as_str())),
        )?;
        unique(
            self.profiles
                .iter()
                .map(|v| (v.id.as_str(), v.name.as_str())),
        )?;
        unique(
            self.templates
                .iter()
                .map(|v| (v.id.as_str(), v.name.as_str())),
        )?;
        for c in &self.connections {
            if c.authentication == super::credentials::Authentication::StoredBearer {
                super::credentials::Binding::new(&c.id, &c.endpoint)?;
            }
            if endpoint(&c.endpoint)? != c.endpoint || !(100..=120000).contains(&c.timeout_ms) {
                return Err(invalid("AI connection endpoint or timeout is invalid"));
            }
            if !c.endpoint_approval.is_empty()
                && c.endpoint_approval != crate::domain::digest(c.endpoint.as_bytes())
            {
                return Err(invalid(
                    "AI endpoint approval does not match this connection",
                ));
            }
            id(&c.revision)?;
        }
        for t in &self.templates {
            t.validate()?;
        }
        for p in &self.profiles {
            id(&p.revision)?;
            let c = self
                .connections
                .iter()
                .find(|c| c.id == p.connection)
                .ok_or_else(|| invalid("AI profile references an unknown connection"))?;
            if p.model.is_empty()
                || p.model.trim() != p.model
                || p.model.len() > 256
                || p.model.chars().any(char::is_control)
            {
                return Err(invalid("Choose an explicit AI model identifier"));
            }
            text(&p.system_message, 65536, "AI system message exceeds limit")?;
            p.parameters.validate(c.provider)?;
            if !(1024..=1024 * 1024).contains(&p.context_bytes)
                || !self.templates.iter().any(|t| t.id == p.template)
            {
                return Err(invalid(
                    "AI profile has an invalid template or context budget",
                ));
            }
        }
        let mut scopes = BTreeSet::new();
        for a in &self.assignments {
            a.scope.validate()?;
            if !scopes.insert(&a.scope) || !self.profiles.iter().any(|p| p.id == a.profile) {
                return Err(invalid(
                    "AI assignment is duplicated or references a missing profile",
                ));
            }
        }
        scopes.clear();
        for i in &self.instructions {
            i.scope.validate()?;
            if !scopes.insert(&i.scope) {
                return Err(invalid("Duplicate AI instruction scope"));
            }
            text(&i.text, 16384, "AI instructions exceed limit")?;
        }
        if serde_json::to_vec(self)?.len() > 2 * 1024 * 1024 {
            return Err(invalid("AI settings exceed 2 MiB"));
        }
        Ok(())
    }
    pub fn effective(
        &self,
        workspace: &WorkspaceId,
        worktree: &WorktreeId,
        one_off: Option<&str>,
    ) -> Result<Effective> {
        self.validate()?;
        let scopes = [
            Scope::Application,
            Scope::Workspace {
                workspace: workspace.clone(),
            },
            Scope::Repository {
                workspace: workspace.clone(),
                worktree: worktree.clone(),
            },
        ];
        let mut selected = None;
        let mut origin = None;
        let mut instructions = Vec::new();
        for scope in scopes {
            if let Some(a) = self.assignments.iter().find(|a| a.scope == scope) {
                selected = Some(a.profile.as_str());
                origin = Some(scope.clone());
            }
            if let Some(i) = self.instructions.iter().find(|i| i.scope == scope) {
                instructions.push(i.text.clone());
            }
        }
        if let Some(id) = one_off {
            selected = Some(id);
            origin = None;
        }
        let selected = selected.ok_or_else(|| {
            Error::with_kind(
                ErrorKind::Unavailable,
                "No AI profile selected; manual workflows remain available",
            )
        })?;
        let profile = self
            .profiles
            .iter()
            .find(|p| p.id == selected)
            .ok_or_else(|| invalid("Explicit AI profile is unavailable; no fallback was selected"))?
            .clone();
        let connection = self
            .connections
            .iter()
            .find(|c| c.id == profile.connection)
            .ok_or_else(|| invalid("AI connection is unavailable"))?
            .clone();
        let template = self
            .templates
            .iter()
            .find(|t| t.id == profile.template)
            .ok_or_else(|| invalid("AI template is unavailable"))?
            .clone();
        Ok(Effective {
            profile,
            connection,
            template,
            instructions,
            origin,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiEffective")]
pub struct Effective {
    pub profile: Profile,
    pub connection: Connection,
    pub template: super::template::Template,
    pub instructions: Vec<String>,
    pub origin: Option<Scope>,
}
