//! Explicit AI metadata merges. A plan is inert and does not authorize activation.
use super::{config::*, portability::Preview, store::Snapshot};
use crate::{domain::digest, Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Choice {
    Add,
    Replace,
    Rename { name: String },
    Skip,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileLinks {
    /// Explicitly use this existing connection instead of the imported dependency.
    pub connection: Option<String>,
    /// Explicitly use this existing template instead of the imported dependency.
    pub template: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeMapping {
    pub source: Scope,
    /// None explicitly leaves this source scope unassigned.
    pub target: Option<Scope>,
    pub replace_assignment: bool,
    pub replace_instructions: bool,
    /// Explicit existing profile, useful when the source profile is skipped.
    pub profile: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Choices {
    pub connections: BTreeMap<String, Choice>,
    pub templates: BTreeMap<String, Choice>,
    pub profiles: BTreeMap<String, Choice>,
    pub profile_links: BTreeMap<String, ProfileLinks>,
    pub scopes: Vec<ScopeMapping>,
    pub import_concurrency: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntityChange {
    pub entity: String,
    pub source_id: String,
    pub target_id: Option<String>,
    pub choice: Choice,
}

/// Only the controller holding this plan may propose a later confirmation.
/// Displayed hashes and JSON are not an activation capability.
pub struct Plan {
    before: String,
    archive: String,
    manifest: String,
    choices: Choices,
    candidate: Settings,
    changes: Vec<EntityChange>,
    omissions: Vec<Scope>,
    fingerprint: String,
}
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
fn exact_choices<'a>(
    ids: impl Iterator<Item = &'a str>,
    choices: &BTreeMap<String, Choice>,
) -> Result<()> {
    let expected: BTreeSet<_> = ids.collect();
    if expected.len() != choices.len() || choices.keys().any(|id| !expected.contains(id.as_str())) {
        return Err(invalid("Choose add, replace, rename or skip for every imported entity; unknown choices are refused"));
    }
    Ok(())
}
fn identity(
    id: &str,
    name: &str,
    exists: bool,
    choice: &Choice,
) -> Result<Option<(String, String)>> {
    match choice {
        Choice::Skip => Ok(None),
        Choice::Add if !exists => Ok(Some((id.into(), name.into()))),
        Choice::Replace if exists => Ok(Some((id.into(), name.into()))),
        Choice::Rename { name } => Ok(Some((Uuid::new_v4().to_string(), name.clone()))),
        _ => Err(invalid("Add requires a new stable ID; replace requires that exact existing stable ID. Rename creates a new ID")),
    }
}
fn dependency(
    source: &str,
    explicit: Option<&str>,
    imported: &BTreeMap<String, String>,
    mut existing: impl Iterator<Item = String>,
) -> Result<String> {
    if let Some(target) = explicit {
        return existing.find(|id| id == target).ok_or_else(|| {
            invalid("Explicit dependency mapping must name an existing local entity")
        });
    }
    imported.get(source).cloned().ok_or_else(|| invalid("Imported profile dependency was skipped; explicitly map an existing dependency or skip the profile"))
}
fn check_snapshot(snapshot: &Snapshot) -> Result<()> {
    snapshot.settings.validate()?;
    if digest(&serde_json::to_vec(&snapshot.settings)?) != snapshot.fingerprint {
        return Err(invalid("Current AI settings fingerprint is inconsistent"));
    }
    Ok(())
}
impl Plan {
    pub fn candidate(&self) -> &Settings {
        &self.candidate
    }
    pub fn changes(&self) -> &[EntityChange] {
        &self.changes
    }
    pub fn unassigned_scopes(&self) -> &[Scope] {
        &self.omissions
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn source_fingerprint(&self) -> &str {
        &self.archive
    }
    /// The controller must reread and reverify the archive, not echo a renderer hash.
    pub fn verify(
        &self,
        current: &Snapshot,
        verified_archive: &Preview,
        allowed_scopes: &[Scope],
    ) -> Result<()> {
        check_snapshot(current)?;
        if current.fingerprint != self.before
            || verified_archive.archive_fingerprint != self.archive
            || verified_archive.manifest_fingerprint != self.manifest
        {
            return Err(Error::with_kind(
                ErrorKind::StaleReview,
                "AI import archive or current settings changed; create a fresh preview",
            ));
        }
        if self
            .choices
            .scopes
            .iter()
            .filter_map(|m| m.target.as_ref())
            .any(|scope| !allowed_scopes.contains(scope))
        {
            return Err(Error::with_kind(
                ErrorKind::StaleReview,
                "Mapped AI import identity is no longer available",
            ));
        }
        if self.digest()? != self.fingerprint {
            return Err(invalid("AI import plan changed"));
        }
        Ok(())
    }
    fn digest(&self) -> Result<String> {
        Ok(digest(&serde_json::to_vec(&(
            &self.before,
            &self.archive,
            &self.manifest,
            &self.choices,
            &self.candidate,
            &self.changes,
            &self.omissions,
        ))?))
    }
}

/// `allowed_scopes` comes from the owning application's current identities, never
/// from imported names. Unmapped scopes stay inert even if their IDs happen to match.
pub fn prepare(
    current: &Snapshot,
    incoming: &Preview,
    choices: Choices,
    allowed_scopes: &[Scope],
) -> Result<Plan> {
    check_snapshot(current)?;
    if [
        &incoming.manifest_fingerprint,
        &incoming.archive_fingerprint,
    ]
    .iter()
    .any(|value| {
        value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(invalid("Invalid verified AI archive fingerprint"));
    }
    incoming.settings.validate()?;
    let source = incoming.settings.clone().imported()?;
    exact_choices(
        source.connections.iter().map(|e| e.id.as_str()),
        &choices.connections,
    )?;
    exact_choices(
        source.templates.iter().map(|e| e.id.as_str()),
        &choices.templates,
    )?;
    exact_choices(
        source.profiles.iter().map(|e| e.id.as_str()),
        &choices.profiles,
    )?;
    if choices.profile_links.keys().any(|id| {
        !choices
            .profiles
            .get(id)
            .is_some_and(|choice| !matches!(choice, Choice::Skip))
    }) {
        return Err(invalid(
            "Dependency mappings require a selected imported profile",
        ));
    }
    let mut candidate = current.settings.clone();
    let mut changes = Vec::new();
    let mut connections = BTreeMap::new();
    let mut templates = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    for value in &source.connections {
        let choice = &choices.connections[&value.id];
        let selected = identity(
            &value.id,
            &value.name,
            current
                .settings
                .connections
                .iter()
                .any(|v| v.id == value.id),
            choice,
        )?;
        changes.push(EntityChange {
            entity: "connection".into(),
            source_id: value.id.clone(),
            target_id: selected.as_ref().map(|v| v.0.clone()),
            choice: choice.clone(),
        });
        if let Some((id, name)) = selected {
            let mut next = value.clone();
            next.id = id.clone();
            next.name = name;
            if matches!(choice, Choice::Replace) {
                candidate.connections.retain(|v| v.id != id);
            }
            candidate.connections.push(next);
            connections.insert(value.id.clone(), id);
        }
    }
    for value in &source.templates {
        let choice = &choices.templates[&value.id];
        let selected = identity(
            &value.id,
            &value.name,
            current.settings.templates.iter().any(|v| v.id == value.id),
            choice,
        )?;
        changes.push(EntityChange {
            entity: "template".into(),
            source_id: value.id.clone(),
            target_id: selected.as_ref().map(|v| v.0.clone()),
            choice: choice.clone(),
        });
        if let Some((id, name)) = selected {
            let mut next = value.clone();
            next.id = id.clone();
            next.name = name;
            if matches!(choice, Choice::Replace) {
                candidate.templates.retain(|v| v.id != id);
            }
            candidate.templates.push(next);
            templates.insert(value.id.clone(), id);
        }
    }
    for value in &source.profiles {
        let choice = &choices.profiles[&value.id];
        let selected = identity(
            &value.id,
            &value.name,
            current.settings.profiles.iter().any(|v| v.id == value.id),
            choice,
        )?;
        changes.push(EntityChange {
            entity: "profile".into(),
            source_id: value.id.clone(),
            target_id: selected.as_ref().map(|v| v.0.clone()),
            choice: choice.clone(),
        });
        if let Some((id, name)) = selected {
            let links = choices
                .profile_links
                .get(&value.id)
                .cloned()
                .unwrap_or_default();
            let mut next = value.clone();
            next.id = id.clone();
            next.name = name;
            next.connection = dependency(
                &value.connection,
                links.connection.as_deref(),
                &connections,
                current.settings.connections.iter().map(|e| e.id.clone()),
            )?;
            next.template = dependency(
                &value.template,
                links.template.as_deref(),
                &templates,
                current.settings.templates.iter().map(|e| e.id.clone()),
            )?;
            if matches!(choice, Choice::Replace) {
                candidate.profiles.retain(|v| v.id != id);
            }
            candidate.profiles.push(next);
            profiles.insert(value.id.clone(), id);
        }
    }
    let sources: BTreeSet<_> = source
        .assignments
        .iter()
        .map(|v| &v.scope)
        .chain(source.instructions.iter().map(|v| &v.scope))
        .collect();
    let mut mappings = BTreeMap::new();
    let mut targets = BTreeSet::new();
    for mapping in &choices.scopes {
        if !sources.contains(&mapping.source) || mappings.insert(&mapping.source, mapping).is_some()
        {
            return Err(invalid("Unknown or duplicate imported scope mapping"));
        }
        if let Some(target) = &mapping.target {
            target.validate()?;
            if !allowed_scopes.contains(target) || !targets.insert(target) {
                return Err(invalid(
                    "Choose a distinct currently available identity for each mapped scope",
                ));
            }
        } else if mapping.profile.is_some()
            || mapping.replace_assignment
            || mapping.replace_instructions
        {
            return Err(invalid(
                "Unassigned scopes cannot include replacement or profile mappings",
            ));
        }
    }
    let mut omissions = Vec::new();
    for scope in sources {
        let Some(mapping) = mappings.get(scope) else {
            omissions.push(scope.clone());
            continue;
        };
        let Some(target) = mapping.target.as_ref() else {
            omissions.push(scope.clone());
            continue;
        };
        if let Some(assignment) = source.assignments.iter().find(|v| &v.scope == scope) {
            let profile = dependency(
                &assignment.profile,
                mapping.profile.as_deref(),
                &profiles,
                current.settings.profiles.iter().map(|v| v.id.clone()),
            )?;
            if candidate.assignments.iter().any(|v| &v.scope == target)
                && !mapping.replace_assignment
            {
                return Err(invalid(
                    "Mapped assignment already exists; explicitly choose replacement",
                ));
            }
            candidate.assignments.retain(|v| &v.scope != target);
            candidate.assignments.push(Assignment {
                scope: target.clone(),
                profile,
            });
        } else if mapping.profile.is_some() || mapping.replace_assignment {
            return Err(invalid("Scope has no imported assignment to map"));
        }
        if let Some(instructions) = source.instructions.iter().find(|v| &v.scope == scope) {
            if candidate.instructions.iter().any(|v| &v.scope == target)
                && !mapping.replace_instructions
            {
                return Err(invalid(
                    "Mapped instructions already exist; explicitly choose replacement",
                ));
            }
            candidate.instructions.retain(|v| &v.scope != target);
            candidate.instructions.push(Instructions {
                scope: target.clone(),
                text: instructions.text.clone(),
            });
        } else if mapping.replace_instructions {
            return Err(invalid("Scope has no imported instructions to replace"));
        }
    }
    if choices.import_concurrency {
        candidate.global_concurrency = source.global_concurrency;
        candidate.connection_concurrency = source.connection_concurrency;
    }
    if serde_json::to_vec(&candidate)? == serde_json::to_vec(&current.settings)? {
        return Err(invalid("No imported changes selected"));
    }
    candidate.revision = Uuid::new_v4().to_string();
    candidate.validate()?;
    let mut plan = Plan {
        before: current.fingerprint.clone(),
        archive: incoming.archive_fingerprint.clone(),
        manifest: incoming.manifest_fingerprint.clone(),
        choices,
        candidate,
        changes,
        omissions,
        fingerprint: String::new(),
    };
    plan.fingerprint = plan.digest()?;
    Ok(plan)
}

#[cfg(test)]
mod tests;
