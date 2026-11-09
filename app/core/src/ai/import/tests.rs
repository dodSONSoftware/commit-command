use super::*;
use crate::ai::{change::Change, credentials::Authentication};
use crate::domain::{WorkspaceId, WorktreeId};

fn configured(name: &str) -> Settings {
    let settings = Settings::default();
    let settings = settings
        .changed(
            &settings.revision,
            Change::Connection(ConnectionDraft {
                id: None,
                name: name.into(),
                provider: Provider::Ollama,
                endpoint: "http://127.0.0.1:11434/".into(),
                timeout_ms: 2000,
                authentication: Authentication::StoredBearer,
            }),
        )
        .unwrap();
    let mut settings = settings
        .changed(
            &settings.revision,
            Change::Profile(ProfileDraft {
                id: None,
                name: name.into(),
                connection: settings.connections[0].id.clone(),
                model: "fixture-model".into(),
                system_message: "Exact imported system\n\n".into(),
                parameters: Parameters::default(),
                template: settings.templates[0].id.clone(),
                context_bytes: 16384,
            }),
        )
        .unwrap();
    settings.connections[0].endpoint_approval = digest(settings.connections[0].endpoint.as_bytes());
    settings.validate().unwrap();
    settings
}
fn snapshot(settings: Settings) -> Snapshot {
    Snapshot {
        fingerprint: digest(&serde_json::to_vec(&settings).unwrap()),
        settings,
    }
}
fn incoming(settings: Settings) -> Preview {
    Preview {
        manifest_fingerprint: "a".repeat(64),
        archive_fingerprint: "c".repeat(64),
        settings,
    }
}
fn rename(source: &Settings) -> Choices {
    Choices {
        connections: source
            .connections
            .iter()
            .map(|v| {
                (
                    v.id.clone(),
                    Choice::Rename {
                        name: format!("Imported {}", v.name),
                    },
                )
            })
            .collect(),
        templates: source
            .templates
            .iter()
            .map(|v| {
                (
                    v.id.clone(),
                    Choice::Rename {
                        name: format!("Imported {}", v.name),
                    },
                )
            })
            .collect(),
        profiles: source
            .profiles
            .iter()
            .map(|v| {
                (
                    v.id.clone(),
                    Choice::Rename {
                        name: format!("Imported {}", v.name),
                    },
                )
            })
            .collect(),
        ..Choices::default()
    }
}
fn assert_refused(result: Result<Plan>, message: &str) {
    assert!(result.err().unwrap().to_string().contains(message));
}

#[test]
fn renamed_import_rewires_dependencies_resets_only_imported_authority_and_binds_preview() {
    let current = snapshot(configured("Local"));
    let source = incoming(configured("Source"));
    let before = serde_json::to_vec(&current.settings).unwrap();
    let source_before = serde_json::to_vec(&source.settings).unwrap();
    let plan = prepare(&current, &source, rename(&source.settings), &[]).unwrap();
    plan.verify(&current, &source, &[]).unwrap();
    let merged = plan.candidate();
    let connection = merged
        .connections
        .iter()
        .find(|v| v.name == "Imported Source")
        .unwrap();
    let profile = merged
        .profiles
        .iter()
        .find(|v| v.name == "Imported Source")
        .unwrap();
    let template = merged
        .templates
        .iter()
        .find(|v| v.name.starts_with("Imported "))
        .unwrap();
    assert_ne!(connection.id, source.settings.connections[0].id);
    assert_eq!(profile.connection, connection.id);
    assert_eq!(profile.template, template.id);
    assert_eq!(profile.system_message, "Exact imported system\n\n");
    assert_eq!(connection.authentication, Authentication::None);
    assert!(connection.endpoint_approval.is_empty());
    assert_ne!(template.revision, source.settings.templates[0].revision);
    assert_ne!(connection.revision, source.settings.connections[0].revision);
    assert_ne!(profile.revision, source.settings.profiles[0].revision);
    assert_eq!(
        serde_json::to_vec(&merged.connections[0]).unwrap(),
        serde_json::to_vec(&current.settings.connections[0]).unwrap()
    );
    assert_eq!(plan.changes().len(), 3);
    assert_eq!(plan.fingerprint().len(), 64);
    assert_eq!(plan.source_fingerprint(), source.archive_fingerprint);
    assert_eq!(before, serde_json::to_vec(&current.settings).unwrap());
    assert_eq!(source_before, serde_json::to_vec(&source.settings).unwrap());
    let mut different_archive = incoming(source.settings.clone());
    different_archive.archive_fingerprint = "b".repeat(64);
    assert!(plan.verify(&current, &different_archive, &[]).is_err());
    let mut changed = current.settings.clone();
    changed.global_concurrency = 2;
    assert!(plan.verify(&snapshot(changed), &source, &[]).is_err());
    let mut forged = current.clone();
    forged.settings.global_concurrency = 2;
    assert!(plan.verify(&forged, &source, &[]).is_err());
}

#[test]
fn replacement_requires_exact_stable_id_and_same_names_never_merge() {
    let current = snapshot(configured("Same name"));
    let source = incoming(configured("Same name"));
    let id = &source.settings.connections[0].id;
    let mut choices = rename(&source.settings);
    choices.connections.insert(id.clone(), Choice::Replace);
    assert_refused(
        prepare(&current, &source, choices.clone(), &[]),
        "exact existing stable ID",
    );
    choices.connections.insert(id.clone(), Choice::Add);
    assert_refused(
        prepare(&current, &source, choices, &[]),
        "Duplicate AI identifier or name",
    );
    let source = incoming(current.settings.clone());
    let mut choices = rename(&source.settings);
    for choice in choices
        .connections
        .values_mut()
        .chain(choices.profiles.values_mut())
        .chain(choices.templates.values_mut())
    {
        *choice = Choice::Replace;
    }
    let plan = prepare(&current, &source, choices.clone(), &[]).unwrap();
    assert_eq!(plan.candidate().connections.len(), 1);
    assert_eq!(plan.candidate().profiles.len(), 1);
    assert_eq!(
        plan.candidate().connections[0].id,
        current.settings.connections[0].id
    );
    assert_eq!(
        plan.candidate().connections[0].authentication,
        Authentication::None
    );
    choices
        .connections
        .insert(source.settings.connections[0].id.clone(), Choice::Add);
    assert_refused(
        prepare(&current, &source, choices, &[]),
        "Add requires a new stable ID",
    );
}

#[test]
fn skipped_dependencies_require_explicit_existing_bindings_and_every_entity_needs_a_choice() {
    let current = snapshot(configured("Local"));
    let source = incoming(configured("Source"));
    let mut choices = rename(&source.settings);
    choices.connections.clear();
    assert_refused(
        prepare(&current, &source, choices.clone(), &[]),
        "every imported entity",
    );
    choices
        .connections
        .insert(source.settings.connections[0].id.clone(), Choice::Skip);
    assert_refused(
        prepare(&current, &source, choices.clone(), &[]),
        "dependency was skipped",
    );
    choices.profile_links.insert(
        source.settings.profiles[0].id.clone(),
        ProfileLinks {
            connection: Some(current.settings.connections[0].id.clone()),
            template: None,
        },
    );
    let plan = prepare(&current, &source, choices.clone(), &[]).unwrap();
    assert_eq!(
        plan.candidate().profiles[1].connection,
        current.settings.connections[0].id
    );
    choices
        .profile_links
        .get_mut(&source.settings.profiles[0].id)
        .unwrap()
        .connection = Some(Uuid::new_v4().to_string());
    assert_refused(
        prepare(&current, &source, choices.clone(), &[]),
        "existing local entity",
    );
    choices
        .profiles
        .insert(source.settings.profiles[0].id.clone(), Choice::Skip);
    assert_refused(
        prepare(&current, &source, choices, &[]),
        "selected imported profile",
    );
}

#[test]
fn assignments_never_rebind_implicitly_and_scope_replacement_is_explicit_and_fresh() {
    let workspace = WorkspaceId(Uuid::new_v4().to_string());
    let scope = Scope::Repository {
        workspace,
        worktree: WorktreeId(Uuid::new_v4().to_string()),
    };
    let mut settings = configured("Local");
    settings.assignments.push(Assignment {
        scope: scope.clone(),
        profile: settings.profiles[0].id.clone(),
    });
    settings.instructions.push(Instructions {
        scope: scope.clone(),
        text: "Local instructions".into(),
    });
    let current = snapshot(settings);
    let mut source = configured("Source");
    source.assignments.push(Assignment {
        scope: scope.clone(),
        profile: source.profiles[0].id.clone(),
    });
    source.instructions.push(Instructions {
        scope: scope.clone(),
        text: "Imported instructions\n\n".into(),
    });
    let source = incoming(source);
    let mut choices = rename(&source.settings);
    let omitted = prepare(
        &current,
        &source,
        choices.clone(),
        std::slice::from_ref(&scope),
    )
    .unwrap();
    assert_eq!(omitted.unassigned_scopes(), std::slice::from_ref(&scope));
    assert_eq!(
        omitted.candidate().assignments[0].profile,
        current.settings.profiles[0].id
    );
    assert_eq!(
        omitted.candidate().instructions[0].text,
        "Local instructions"
    );
    choices.scopes.push(ScopeMapping {
        source: scope.clone(),
        target: Some(scope.clone()),
        replace_assignment: false,
        replace_instructions: false,
        profile: None,
    });
    assert_refused(
        prepare(&current, &source, choices.clone(), &[]),
        "currently available identity",
    );
    assert_refused(
        prepare(
            &current,
            &source,
            choices.clone(),
            std::slice::from_ref(&scope),
        ),
        "assignment already exists",
    );
    choices.scopes[0].replace_assignment = true;
    assert_refused(
        prepare(
            &current,
            &source,
            choices.clone(),
            std::slice::from_ref(&scope),
        ),
        "instructions already exist",
    );
    choices.scopes[0].replace_instructions = true;
    let plan = prepare(
        &current,
        &source,
        choices.clone(),
        std::slice::from_ref(&scope),
    )
    .unwrap();
    assert_eq!(
        plan.candidate().instructions[0].text,
        "Imported instructions\n\n"
    );
    assert_ne!(
        plan.candidate().assignments[0].profile,
        current.settings.profiles[0].id
    );
    assert!(plan.verify(&current, &source, &[]).is_err());
    choices.scopes.push(choices.scopes[0].clone());
    assert_refused(
        prepare(&current, &source, choices, std::slice::from_ref(&scope)),
        "duplicate imported scope",
    );
}

#[test]
fn concurrency_requires_an_explicit_choice_and_invalid_result_is_never_returned() {
    let current = snapshot(configured("Local"));
    let mut settings = configured("Source");
    settings.global_concurrency = 4;
    settings.connection_concurrency = 2;
    let source = incoming(settings);
    let mut choices = rename(&source.settings);
    assert_eq!(
        prepare(&current, &source, choices.clone(), &[])
            .unwrap()
            .candidate()
            .global_concurrency,
        1
    );
    choices.import_concurrency = true;
    let plan = prepare(&current, &source, choices.clone(), &[]).unwrap();
    assert_eq!(plan.candidate().global_concurrency, 4);
    assert_eq!(plan.candidate().connection_concurrency, 2);
    choices.connections.insert(
        source.settings.connections[0].id.clone(),
        Choice::Rename {
            name: String::new(),
        },
    );
    assert!(prepare(&current, &source, choices, &[]).is_err());
}

#[test]
fn new_ids_are_preserved_on_add_and_all_skip_does_not_create_a_revision() {
    let current = snapshot(configured("Local"));
    let source = incoming(configured("Source"));
    let mut choices = rename(&source.settings);
    for value in choices
        .connections
        .values_mut()
        .chain(choices.profiles.values_mut())
    {
        *value = Choice::Add;
    }
    for value in choices.templates.values_mut() {
        *value = Choice::Skip;
    }
    choices.profile_links.insert(
        source.settings.profiles[0].id.clone(),
        ProfileLinks {
            connection: None,
            template: Some(current.settings.templates[0].id.clone()),
        },
    );
    let plan = prepare(&current, &source, choices.clone(), &[]).unwrap();
    assert_eq!(
        plan.candidate().connections[1].id,
        source.settings.connections[0].id
    );
    assert_eq!(
        plan.candidate().profiles[1].id,
        source.settings.profiles[0].id
    );
    assert_eq!(
        plan.candidate().profiles[1].template,
        current.settings.templates[0].id
    );
    for value in choices
        .connections
        .values_mut()
        .chain(choices.profiles.values_mut())
    {
        *value = Choice::Skip;
    }
    choices.profile_links.clear();
    assert_refused(
        prepare(&current, &source, choices, &[]),
        "No imported changes selected",
    );
}
