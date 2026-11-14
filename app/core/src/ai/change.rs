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
    SingleSetup {
        connection: ConnectionDraft,
        model: String,
        system_message: String,
        parameters: Parameters,
    },
    Connection(ConnectionDraft),
    Profile(ProfileDraft),
    ProfileSetup {
        connection: Option<ConnectionDraft>,
        profile: ProfileDraft,
        make_default: bool,
    },
    DeleteProfileWithAssignments {
        id: String,
    },
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
fn save_connection(next: &mut Settings, draft: ConnectionDraft, revision: &str) -> Result<String> {
    let id = identity(&next.connections, draft.id, |value| &value.id)?;
    replace(
        &mut next.connections,
        Connection {
            id: id.clone(),
            revision: revision.into(),
            name: draft.name,
            provider: draft.provider,
            endpoint: endpoint(&draft.endpoint)?,
            timeout_ms: draft.timeout_ms,
            authentication: draft.authentication,
            endpoint_approval: String::new(),
        },
        |value| &value.id,
    );
    Ok(id)
}
fn save_profile(next: &mut Settings, draft: ProfileDraft, revision: &str) -> Result<String> {
    let id = identity(&next.profiles, draft.id, |value| &value.id)?;
    replace(
        &mut next.profiles,
        Profile {
            id: id.clone(),
            revision: revision.into(),
            name: draft.name,
            connection: draft.connection,
            model: draft.model,
            system_message: draft.system_message,
            parameters: draft.parameters,
            template: draft.template,
            context_bytes: draft.context_bytes,
        },
        |value| &value.id,
    );
    Ok(id)
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
            Change::SingleSetup {
                connection,
                model,
                system_message,
                parameters,
            } => {
                let previous = next
                    .assignments
                    .iter()
                    .find(|a| a.scope == Scope::Application)
                    .and_then(|a| next.profiles.iter().find(|p| p.id == a.profile))
                    .or_else(|| next.profiles.first());
                let profile_id = previous.map(|p| p.id.clone());
                let connection_id = save_connection(&mut next, connection, &revision)?;
                let template = Template::default();
                let id = save_profile(
                    &mut next,
                    ProfileDraft {
                        id: profile_id,
                        name: "AI".into(),
                        connection: connection_id.clone(),
                        model,
                        system_message,
                        parameters,
                        template: template.id.clone(),
                        context_bytes: 16384,
                    },
                    &revision,
                )?;
                // Replace the old hierarchy atomically so every workspace uses this AI.
                next.connections.retain(|c| c.id == connection_id);
                next.profiles.retain(|p| p.id == id);
                next.templates = vec![template];
                next.assignments = vec![Assignment {
                    scope: Scope::Application,
                    profile: id,
                }];
                next.instructions.clear();
                next.global_concurrency = 1;
                next.connection_concurrency = 1;
            }
            Change::Connection(v) => {
                save_connection(&mut next, v, &revision)?;
            }
            Change::Profile(v) => {
                save_profile(&mut next, v, &revision)?;
            }
            Change::ProfileSetup {
                connection,
                mut profile,
                make_default,
            } => {
                if let Some(connection) = connection {
                    profile.connection = save_connection(&mut next, connection, &revision)?;
                }
                let id = save_profile(&mut next, profile, &revision)?;
                next.assignments.retain(|assignment| {
                    assignment.scope != Scope::Application
                        || (!make_default && assignment.profile != id)
                });
                if make_default {
                    next.assignments.push(Assignment {
                        scope: Scope::Application,
                        profile: id,
                    });
                }
            }
            Change::DeleteProfileWithAssignments { id } => {
                delete(&mut next.profiles, &id, |value| &value.id)?;
                next.assignments
                    .retain(|assignment| assignment.profile != id);
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
