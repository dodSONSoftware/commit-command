use super::*;
use crate::{
    ai::{
        config::{Scope, Settings},
        import::{self, Choices, EntityChange, Plan},
        portability::Preview,
        store::{self, Snapshot},
    },
    backup::{
        archive::{Manifest, ValidatedArchive},
        publication::Destination,
    },
    inspection::Cancellation,
};
use std::io::Cursor;

// Portable settings are at most2 MiB; bound all ZIP framing before allocation.
const MAX_SOURCE: usize = 34 * 1024 * 1024;
const LIFETIME: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiImportSource {
    pub source: String,
    pub path: NativePath,
    pub archive_fingerprint: String,
    pub manifest_fingerprint: String,
    pub settings: Settings,
    pub manifest: Manifest,
    pub available_scopes: Vec<Scope>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiImportPreview {
    pub preview: String,
    pub fingerprint: String,
    pub source: String,
    pub archive_fingerprint: String,
    pub current_fingerprint: String,
    pub candidate: Settings,
    pub changes: Vec<EntityChange>,
    pub unassigned_scopes: Vec<Scope>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiImportTarget {
    pub scope: Scope,
    pub name: String,
    pub path: Option<NativePath>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiImportTargets {
    pub items: Vec<AiImportTarget>,
    pub total: usize,
    pub next: Option<usize>,
}
#[derive(Clone)]
struct Source {
    id: String,
    path: PathBuf,
    expires: Instant,
    preview: Preview,
    manifest: Manifest,
}
struct Prepared {
    source: Source,
    plan: Plan,
    view: AiImportPreview,
}
#[derive(Default)]
pub(in crate::application) struct Imports {
    source: Option<Source>,
    prepared: Option<Prepared>,
}
pub(in crate::application) struct ImportReview {
    pub view: AiImportPreview,
    source: Source,
    plan: Plan,
}
impl ImportReview {
    pub(in crate::application) fn remaining_lifetime(&self) -> Duration {
        self.source
            .expires
            .saturating_duration_since(Instant::now())
    }
}

fn cancelled(cancel: &Cancellation) -> Result<()> {
    if cancel.is_cancelled() {
        return Err(Error::with_kind(
            ErrorKind::Cancelled,
            "AI import cancelled",
        ));
    }
    Ok(())
}
fn read_source(path: &Path, cancel: &Cancellation) -> Result<(Preview, Manifest)> {
    cancelled(cancel)?;
    let bytes = inspection::file_bytes(path, MAX_SOURCE)?;
    let archive = ValidatedArchive::inspect(Cursor::new(bytes), cancel)?;
    let manifest = archive.manifest().clone();
    Ok((crate::ai::portability::preview(archive, cancel)?, manifest))
}
fn available(app: &Application) -> Result<Vec<Scope>> {
    let mut scopes = vec![Scope::Application];
    if let Some(workspace) = app.writes.workspace.active()? {
        scopes.push(Scope::Workspace {
            workspace: workspace.clone(),
        });
        for entry in app.writes.workspace.inventory(&workspace)? {
            if entry.superseded_by.is_none()
                && matches!(entry.availability, Availability::Available)
            {
                scopes.push(Scope::Repository {
                    workspace: workspace.clone(),
                    worktree: entry.id,
                });
            }
        }
    }
    Ok(scopes)
}
fn validate_scope(app: &Application, scope: &Scope) -> Result<()> {
    match scope {
        Scope::Application => (),
        Scope::Workspace { workspace } => {
            app.active(&workspace.0)?;
        }
        Scope::Repository {
            workspace,
            worktree,
        } => {
            app.entry(&workspace.0, &worktree.0)?;
        }
    }
    Ok(())
}
fn revalidate(
    app: &Application,
    source: &Source,
    plan: &Plan,
    cancel: &Cancellation,
) -> Result<Snapshot> {
    if Instant::now() >= source.expires {
        return Err(stale());
    }
    cancelled(cancel)?;
    let (fresh, _) = read_source(&source.path, cancel)?;
    let current = store::load(app.writes.workspace.data_directory())?;
    for scope in plan.mapped_scopes() {
        validate_scope(app, scope)?;
    }
    plan.verify(&current, &fresh, &available(app)?)?;
    Ok(current)
}
impl Application {
    /// Native adapters only: the renderer cannot register a filesystem path.
    /// One immutable decoded source is retained; selecting another revokes old tokens.
    pub fn register_ai_import(&mut self, path: &Path) -> Result<AiImportSource> {
        self.ai_imports = Imports::default();
        self.abandon_review()?;
        if !path.is_absolute() {
            return Err(invalid("AI archive selection must be absolute"));
        }
        let (preview, manifest) = read_source(path, &Cancellation::default())?;
        let id = Uuid::new_v4().to_string();
        let view = AiImportSource {
            source: id.clone(),
            path: NativePath::from_path(path),
            archive_fingerprint: preview.archive_fingerprint.clone(),
            manifest_fingerprint: preview.manifest_fingerprint.clone(),
            settings: preview.settings.clone(),
            manifest: manifest.clone(),
            available_scopes: available(self)?,
        };
        self.ai_imports = Imports {
            source: Some(Source {
                id,
                path: path.to_path_buf(),
                expires: Instant::now() + LIFETIME,
                preview,
                manifest,
            }),
            prepared: None,
        };
        Ok(view)
    }
}
pub(in crate::application) fn prepare_import(
    app: &mut Application,
    source_id: &str,
    choices: Choices,
) -> Result<AiImportPreview> {
    app.ai_imports.prepared = None;
    app.abandon_review()?;
    let source = app
        .ai_imports
        .source
        .as_ref()
        .filter(|s| s.id == source_id && Instant::now() < s.expires)
        .ok_or_else(stale)?
        .clone();
    let current = store::load(app.writes.workspace.data_directory())?;
    let plan = import::prepare(&current, &source.preview, choices, &available(app)?)?;
    revalidate(app, &source, &plan, &app.read_options.cancellation)?;
    let view = AiImportPreview {
        preview: Uuid::new_v4().to_string(),
        fingerprint: plan.fingerprint().into(),
        source: source.id.clone(),
        archive_fingerprint: plan.source_fingerprint().into(),
        current_fingerprint: current.fingerprint,
        candidate: plan.candidate().clone(),
        changes: plan.changes().to_vec(),
        unassigned_scopes: plan.unassigned_scopes().to_vec(),
    };
    app.ai_imports.prepared = Some(Prepared {
        source,
        plan,
        view: view.clone(),
    });
    Ok(view)
}
pub(in crate::application) fn inspect_import(
    app: &Application,
    id: &str,
) -> Result<AiImportSource> {
    let source = app
        .ai_imports
        .source
        .as_ref()
        .filter(|s| s.id == id && Instant::now() < s.expires)
        .ok_or_else(stale)?;
    Ok(AiImportSource {
        source: source.id.clone(),
        path: NativePath::from_path(&source.path),
        archive_fingerprint: source.preview.archive_fingerprint.clone(),
        manifest_fingerprint: source.preview.manifest_fingerprint.clone(),
        settings: source.preview.settings.clone(),
        manifest: source.manifest.clone(),
        available_scopes: available(app)?,
    })
}
pub(in crate::application) fn targets(
    app: &Application,
    source: &str,
    filter: &str,
    offset: usize,
) -> Result<AiImportTargets> {
    app.ai_imports
        .source
        .as_ref()
        .filter(|s| s.id == source && Instant::now() < s.expires)
        .ok_or_else(stale)?;
    if filter.len() > 512 || filter.contains('\0') {
        return Err(invalid("AI import target search exceeds its limit"));
    }
    let mut items = vec![AiImportTarget {
        scope: Scope::Application,
        name: "Application".into(),
        path: None,
    }];
    if let Some(workspace) = app.writes.workspace.active()? {
        let definition = app
            .writes
            .workspace
            .list()
            .iter()
            .find(|w| w.id == workspace)
            .ok_or_else(stale)?;
        items.push(AiImportTarget {
            scope: Scope::Workspace {
                workspace: workspace.clone(),
            },
            name: definition.name.clone(),
            path: Some(definition.display_root.clone()),
        });
        for entry in app.writes.workspace.inventory(&workspace)? {
            if entry.superseded_by.is_none()
                && matches!(entry.availability, Availability::Available)
            {
                items.push(AiImportTarget {
                    scope: Scope::Repository {
                        workspace: workspace.clone(),
                        worktree: entry.id,
                    },
                    name: "Repository".into(),
                    path: Some(entry.display_path),
                });
            }
        }
    }
    target_page(items, filter, offset)
}
fn target_page(
    mut items: Vec<AiImportTarget>,
    filter: &str,
    offset: usize,
) -> Result<AiImportTargets> {
    let filter = filter.to_lowercase();
    items.retain(|item| {
        format!(
            "{} {} {:?}",
            item.name,
            item.path
                .as_ref()
                .map(NativePath::display)
                .unwrap_or_default(),
            item.scope
        )
        .to_lowercase()
        .contains(&filter)
    });
    items.sort_by_cached_key(|item| {
        (
            item.path
                .as_ref()
                .map(NativePath::display)
                .unwrap_or_default(),
            format!("{:?}", item.scope),
        )
    });
    let total = items.len();
    if offset > total {
        return Err(invalid(
            "AI import targets changed; search again from the first page",
        ));
    }
    let end = offset.saturating_add(100).min(total);
    Ok(AiImportTargets {
        items: items.into_iter().skip(offset).take(100).collect(),
        total,
        next: (end < total).then_some(end),
    })
}
pub(in crate::application) fn review_import(
    app: &mut Application,
    id: &str,
    fingerprint: &str,
) -> Result<ImportReview> {
    let prepared = app
        .ai_imports
        .prepared
        .take()
        .filter(|p| p.view.preview == id && p.plan.fingerprint() == fingerprint)
        .ok_or_else(stale)?;
    revalidate(
        app,
        &prepared.source,
        &prepared.plan,
        &app.read_options.cancellation,
    )?;
    Ok(ImportReview {
        view: prepared.view,
        source: prepared.source,
        plan: prepared.plan,
    })
}
pub(in crate::application) fn apply_import(
    app: &mut Application,
    review: ImportReview,
    cancel: &Cancellation,
) -> Result<serde_json::Value> {
    if !app.ai_jobs.is_idle()? {
        return Err(Error::with_kind(
            ErrorKind::Busy,
            "Finish or cancel active AI requests before importing configuration",
        ));
    }
    let current = revalidate(app, &review.source, &review.plan, cancel)?;
    let root = app.writes.workspace.data_directory().to_path_buf();
    let safety = Destination::private_child(&root, "ai-import-safety")?;
    let name = format!("ai-pre-import-{}.backup.zip", Uuid::new_v4());
    crate::ai::portability::publish(&current, &safety, &name, now_ms(), cancel)?;
    revalidate(app, &review.source, &review.plan, cancel)?;
    cancelled(cancel)?;
    store::save(&root, &current.fingerprint, review.plan.candidate())?;
    app.ai_imports = Imports::default();
    app.ai_jobs.set_limits(crate::ai::jobs::Limits {
        global: review.plan.candidate().global_concurrency,
        connection: review.plan.candidate().connection_concurrency,
    })?;
    Ok(
        json!({ "ai_import": { "snapshot": store::load(&root)?, "safety_backup": NativePath::from_path(&root.join("ai-import-safety").join(name)) } }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_pages_are_bounded_literal_searches_with_stable_path_order() {
        let items = (0..205)
            .rev()
            .map(|n| AiImportTarget {
                scope: Scope::Repository {
                    workspace: WorkspaceId(Uuid::nil().to_string()),
                    worktree: WorktreeId(Uuid::from_u128(n + 1).to_string()),
                },
                name: "Repository".into(),
                path: Some(NativePath::from_path(&PathBuf::from(format!(
                    "/fixture/repo-{n:03}"
                )))),
            })
            .collect::<Vec<_>>();
        let first = target_page(items.clone(), "", 0).unwrap();
        let second = target_page(items.clone(), "", first.next.unwrap()).unwrap();
        let third = target_page(items.clone(), "", second.next.unwrap()).unwrap();
        assert_eq!(
            (first.items.len(), second.items.len(), third.items.len()),
            (100, 100, 5)
        );
        assert!(third.next.is_none());
        assert_eq!(first.total, 205);
        let scopes = first
            .items
            .iter()
            .chain(&second.items)
            .chain(&third.items)
            .map(|item| item.scope.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(scopes.len(), 205);
        assert_eq!(
            first.items[0].path.as_ref().unwrap().display(),
            "/fixture/repo-000"
        );
        assert_eq!(
            third.items[4].path.as_ref().unwrap().display(),
            "/fixture/repo-204"
        );
        assert_eq!(target_page(items.clone(), "REPO-10", 0).unwrap().total, 10);
        assert_eq!(target_page(items.clone(), "[", 0).unwrap().total, 0);
        assert!(target_page(items, "", 206).is_err());
    }
    #[test]
    fn expired_native_sources_and_previews_are_refused_before_opening_the_source() {
        let root = std::env::temp_dir().join(format!("gitfleet-import-expiry-{}", Uuid::new_v4()));
        let mut app = Application::open(&root, Options::default()).unwrap();
        let incoming = Preview {
            archive_fingerprint: "a".repeat(64),
            manifest_fingerprint: "b".repeat(64),
            settings: Settings::default(),
        };
        let source = Source {
            id: "expired-source".into(),
            path: root.join("must-not-be-opened.zip"),
            expires: Instant::now(),
            preview: incoming,
            manifest: Manifest {
                version: 1,
                application_version: "fixture".into(),
                created_at_ms: 0,
                kind: crate::backup::archive::Kind::AiConfiguration,
                database_schema: None,
                settings_revision: None,
                includes_history: false,
                recovery_policy: "not_included".into(),
                components: Vec::new(),
            },
        };
        app.ai_imports.source = Some(source.clone());
        assert_eq!(
            inspect_import(&app, &source.id).unwrap_err().kind,
            ErrorKind::StaleReview
        );
        assert_eq!(
            prepare_import(&mut app, &source.id, Choices::default())
                .unwrap_err()
                .kind,
            ErrorKind::StaleReview
        );
        let current = store::load(app.writes.workspace.data_directory()).unwrap();
        let mut choices = Choices::default();
        choices.templates.insert(
            source.preview.settings.templates[0].id.clone(),
            import::Choice::Rename {
                name: "Imported fixture".into(),
            },
        );
        let plan = import::prepare(&current, &source.preview, choices, &[]).unwrap();
        assert_eq!(
            revalidate(&app, &source, &plan, &Cancellation::default())
                .unwrap_err()
                .kind,
            ErrorKind::StaleReview
        );
        assert!(!root.join("ai").exists());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
