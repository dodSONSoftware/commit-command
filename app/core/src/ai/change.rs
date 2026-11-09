//! Validated editor changes. Endpoint metadata is not transport authorization.
use super::{
    config::*,
    template::{Template, TemplateDraft},
};
use crate::{Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(rename = "AiChange")]
pub enum Change {
    Connection(ConnectionDraft),
    Profile(ProfileDraft),
    Template(TemplateDraft),
    DeleteConnection {
        id: String,
    },
    DeleteProfile {
        id: String,
    },
    DeleteTemplate {
        id: String,
    },
    Assign {
        scope: Scope,
        profile: Option<String>,
    },
    Instructions {
        scope: Scope,
        text: String,
    },
    Concurrency {
        global: usize,
        per_connection: usize,
    },
}
fn missing() -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, "AI editor item no longer exists")
}
fn identity<T>(items: &[T], requested: Option<String>, get: impl Fn(&T) -> &str) -> Result<String> {
    match requested {
        Some(id) if items.iter().any(|item| get(item) == id) => Ok(id),
        Some(_) => Err(missing()),
        None => Ok(Uuid::new_v4().to_string()),
    }
}
fn replace<T>(items: &mut Vec<T>, value: T, get: impl Fn(&T) -> &str) {
    if let Some(i) = items.iter().position(|item| get(item) == get(&value)) {
        items[i] = value;
    } else {
        items.push(value);
    }
}
fn delete<T>(items: &mut Vec<T>, id: &str, get: impl Fn(&T) -> &str) -> Result<()> {
    let i = items
        .iter()
        .position(|item| get(item) == id)
        .ok_or_else(missing)?;
    items.remove(i);
    Ok(())
}
impl Settings {
    /// Produce a new validated value; errors leave the caller's original untouched.
    pub fn changed(&self, expected_revision: &str, change: Change) -> Result<Self> {
        self.validate()?;
        if self.revision != expected_revision {
            return Err(Error::with_kind(
                ErrorKind::StaleReview,
                "AI settings changed; reload the editor",
            ));
        }
        let mut next = self.clone();
        let revision = Uuid::new_v4().to_string();
        match change {
            Change::Connection(v) => {
                let id = identity(&next.connections, v.id, |v| &v.id)?;
                let value = Connection {
                    id,
                    revision: revision.clone(),
                    name: v.name,
                    provider: v.provider,
                    endpoint: endpoint(&v.endpoint)?,
                    timeout_ms: v.timeout_ms,
                    authentication: v.authentication,
                    endpoint_approval: String::new(),
                };
                replace(&mut next.connections, value, |v| &v.id);
            }
            Change::Profile(v) => {
                let id = identity(&next.profiles, v.id, |v| &v.id)?;
                let value = Profile {
                    id,
                    revision: revision.clone(),
                    name: v.name,
                    connection: v.connection,
                    model: v.model,
                    system_message: v.system_message,
                    parameters: v.parameters,
                    template: v.template,
                    context_bytes: v.context_bytes,
                };
                replace(&mut next.profiles, value, |v| &v.id);
            }
            Change::Template(v) => {
                let id = identity(&next.templates, v.id, |v| &v.id)?;
                replace(
                    &mut next.templates,
                    Template {
                        id,
                        revision: revision.clone(),
                        name: v.name,
                        rules: v.rules,
                        examples: v.examples,
                    },
                    |v| &v.id,
                );
            }
            Change::DeleteConnection { id } => delete(&mut next.connections, &id, |v| &v.id)?,
            Change::DeleteProfile { id } => delete(&mut next.profiles, &id, |v| &v.id)?,
            Change::DeleteTemplate { id } => delete(&mut next.templates, &id, |v| &v.id)?,
            Change::Assign { scope, profile } => {
                next.assignments.retain(|a| a.scope != scope);
                if let Some(profile) = profile {
                    next.assignments.push(Assignment { scope, profile });
                }
            }
            Change::Instructions { scope, text } => {
                next.instructions.retain(|i| i.scope != scope);
                if !text.is_empty() {
                    next.instructions.push(Instructions { scope, text });
                }
            }
            Change::Concurrency {
                global,
                per_connection,
            } => {
                next.global_concurrency = global;
                next.connection_concurrency = per_connection;
            }
        }
        next.revision = revision;
        next.validate()?;
        Ok(next)
    }
    /// Imported metadata cannot carry a destination approval into another installation.
    pub fn imported(mut self) -> Result<Self> {
        for c in &mut self.connections {
            c.endpoint_approval.clear();
            c.authentication = super::credentials::Authentication::None;
            c.revision = Uuid::new_v4().to_string();
        }
        for p in &mut self.profiles {
            p.revision = Uuid::new_v4().to_string();
        }
        for template in &mut self.templates {
            template.revision = Uuid::new_v4().to_string();
        }
        self.revision = Uuid::new_v4().to_string();
        self.validate()?;
        Ok(self)
    }
}
