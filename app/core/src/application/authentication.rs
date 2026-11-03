use super::*;
use crate::authentication::{self as auth, Change, Settings};
#[derive(Clone, Serialize)]
pub(super) struct SettingsPlan {
    expected_revision: String,
    pub description: String,
    next: Settings,
    checked: Vec<String>,
    assignment: Option<auth::Assignment>,
}
impl Application {
    pub(super) fn authentication_settings(&mut self) -> Result<Settings> {
        auth::load(self.writes.workspace.connection())
    }
    pub(super) fn review_authentication(&mut self, change: Change) -> Result<ReviewDisplay> {
        self.abandon_review()?;
        let mut description = serde_json::to_string_pretty(&change)?;
        let mut next = self.authentication_settings()?;
        let expected_revision = next.revision.clone();
        let mut checked = Vec::new();
        let mut assignment = None;
        match change {
            Change::Save { id, name, policy } => {
                if name.trim() != name
                    || name.is_empty()
                    || name.len() > 80
                    || name.chars().any(char::is_control)
                {
                    return Err(invalid("Profile name must be 1–80 bytes without surrounding whitespace or controls"));
                }
                let id = match id {
                    Some(id) => {
                        if !next.profiles.iter().any(|p| p.id == id) {
                            return Err(invalid("Unknown authentication profile"));
                        }
                        id
                    }
                    None => Uuid::new_v4().to_string(),
                };
                if next
                    .profiles
                    .iter()
                    .any(|p| p.id != id && p.name.to_lowercase() == name.to_lowercase())
                {
                    return Err(invalid("Profile name already exists"));
                }
                let (options, kind) = policy.options(&self.writes.authentication_options())?;
                let tools = crate::writes::authentication_tools(&options, &kind)?;
                next.profiles.retain(|p| p.id != id);
                checked.push(id.clone());
                next.profiles.push(auth::Profile {
                    id,
                    name,
                    policy,
                    tools,
                });
            }
            Change::Delete { id } => {
                if next.assignments.iter().any(|a| a.profile == id) {
                    return Err(invalid(
                        "Unassign this profile from every remote before deleting it",
                    ));
                }
                let count = next.profiles.len();
                next.profiles.retain(|p| p.id != id);
                if count == next.profiles.len() {
                    return Err(invalid("Unknown authentication profile"));
                }
            }
            Change::Assign {
                workspace,
                worktree,
                remote,
                profile,
            } => {
                let entry = self.entry(&workspace, &worktree)?;
                let ws = self.active(&workspace)?;
                if remote.is_empty() || remote.len() > 256 || remote.chars().any(char::is_control) {
                    return Err(invalid("Invalid remote name"));
                }
                if let Some(id) = profile {
                    let p = next
                        .profiles
                        .iter()
                        .find(|p| p.id == id)
                        .ok_or_else(|| invalid("Unknown authentication profile"))?;
                    let resolved = crate::remote::resolve_for_observation(
                        &entry.metadata.root.to_path()?,
                        self.writes.authentication_global_config(),
                        &remote,
                        &self.read_options,
                    )?;
                    let (options, kind) =
                        p.policy.options(&self.writes.authentication_options())?;
                    if kind != resolved.transport
                        || crate::writes::authentication_tools(&options, &kind)? != p.tools
                    {
                        return Err(invalid(
                            "Profile transport or tools changed; review the profile again",
                        ));
                    }
                    if let auth::ProfilePolicy::Https { account, .. } = &p.policy {
                        crate::credentials::scope(&resolved.url, account)?;
                    }
                    description.push_str(&format!(
                        "\nAssign profile {} to remote {} at {}.\nWorktree: {}",
                        p.name,
                        remote,
                        resolved.url,
                        entry.metadata.root.to_path()?.display()
                    ));
                    checked.push(id.clone());
                    assignment = Some(auth::Assignment {
                        workspace: ws,
                        worktree: entry.id,
                        remote: remote.clone(),
                        destination: resolved.url,
                        profile: id,
                    });
                }
                next.assignments.retain(|a| {
                    a.workspace.0 != workspace || a.worktree.0 != worktree || a.remote != remote
                });
                if let Some(a) = &assignment {
                    next.assignments.push(a.clone());
                }
            }
        }
        if next.profiles.len() > 64
            || next.assignments.len() > 1024
            || serde_json::to_vec(&next)?.len() > 1024 * 1024
        {
            return Err(invalid("Authentication settings exceed limit"));
        }
        next.revision = Uuid::new_v4().to_string();
        self.set_review(review::Action::Authentication(Box::new(SettingsPlan {
            expected_revision,
            description,
            next,
            checked,
            assignment,
        })))
    }
    pub(super) fn apply_authentication(
        &mut self,
        plan: SettingsPlan,
        cancellation: inspection::Cancellation,
    ) -> Result<serde_json::Value> {
        if cancellation.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Authentication settings cancelled",
            ));
        }
        if self.authentication_settings()?.revision != plan.expected_revision {
            return Err(stale());
        }
        for id in &plan.checked {
            let p = plan
                .next
                .profiles
                .iter()
                .find(|p| &p.id == id)
                .ok_or_else(stale)?;
            let (options, kind) = p.policy.options(&self.writes.authentication_options())?;
            if crate::writes::authentication_tools(&options, &kind)? != p.tools {
                return Err(stale());
            }
        }
        if let Some(a) = &plan.assignment {
            self.active(&a.workspace.0)?;
            let entry = self.entry(&a.workspace.0, &a.worktree.0)?;
            let resolved = crate::remote::resolve_for_observation(
                &entry.metadata.root.to_path()?,
                self.writes.authentication_global_config(),
                &a.remote,
                &self.read_options,
            )?;
            if resolved.url != a.destination {
                return Err(stale());
            }
        }
        auth::save(self.writes.workspace.connection(), &plan.next)?;
        Ok(json!({"authentication_settings_saved":true,"revision":plan.next.revision}))
    }
}
