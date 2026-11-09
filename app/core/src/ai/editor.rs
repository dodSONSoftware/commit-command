//! Transient editor tests. No settings generation is written or assigned.
use super::{capture::Captured, config::*, provider::Plan, template::Template};
use crate::{Error, ErrorKind, Result};
use uuid::Uuid;

pub(crate) fn plan(
    connection: ConnectionDraft,
    draft: ProfileDraft,
    mut template: Template,
    instructions: Vec<String>,
    captured: &Captured,
    capture_budget: usize,
) -> Result<Plan> {
    if instructions.len() > 4
        || instructions
            .iter()
            .any(|text| text.len() > 16384 || text.contains('\0'))
    {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "AI test instructions exceed bounds",
        ));
    }
    if capture_budget > draft.context_bytes {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Context budget exceeds the unsaved profile; capture a smaller context",
        ));
    }
    let revision = Uuid::new_v4().to_string();
    let connection_id = connection
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    // The shown editors are the explicit test inputs, independent of saved assignments.
    // Fresh revisions prevent these transient values claiming a saved entity version.
    template.revision = revision.clone();
    let profile = Profile {
        id: draft.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        revision: revision.clone(),
        name: draft.name,
        connection: connection_id.clone(),
        model: draft.model,
        system_message: draft.system_message,
        parameters: draft.parameters,
        template: template.id.clone(),
        context_bytes: draft.context_bytes,
    };
    let temporary = Settings {
        connections: vec![Connection {
            id: connection_id,
            revision,
            name: connection.name.clone(),
            provider: connection.provider,
            endpoint: endpoint(&connection.endpoint)?,
            timeout_ms: connection.timeout_ms,
            authentication: connection.authentication,
            endpoint_approval: String::new(),
        }],
        profiles: vec![profile.clone()],
        templates: vec![template.clone()],
        ..Settings::default()
    };
    temporary.validate()?;
    Plan::editor_test(
        &connection,
        &profile,
        captured.bundle.clone(),
        captured.snapshot_fingerprint.clone(),
        template,
        instructions,
    )
}
