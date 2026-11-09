use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiExport {
    pub path: NativePath,
    pub manifest_fingerprint: String,
}

fn marker(path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn export(app: &mut Application, picker: &str, fingerprint: &str) -> Result<AiExport> {
    let snapshot = crate::ai::store::load(app.writes.workspace.data_directory())?;
    if snapshot.fingerprint != fingerprint {
        return Err(stale());
    }
    let parent = app.directory(picker)?;
    if parent.starts_with(app.writes.workspace.data_directory()) {
        return Err(invalid(
            "Choose a backup destination outside application state",
        ));
    }
    for workspace in app.writes.workspace.list() {
        if parent.starts_with(workspace.root.to_path()?) {
            return Err(invalid(
                "Choose a backup destination outside workspace roots",
            ));
        }
    }
    for ancestor in parent.ancestors() {
        if marker(&ancestor.join(".git"))?
            || (marker(&ancestor.join("HEAD"))? && marker(&ancestor.join("objects"))?)
        {
            return Err(invalid(
                "Choose a backup destination outside Git repositories and control directories",
            ));
        }
    }
    let directory =
        crate::backup::publication::Destination::private_child(&parent, "Commit Command Backups")?;
    let name = format!("ai-configuration-{}.backup.zip", Uuid::new_v4());
    let manifest_fingerprint = crate::ai::portability::publish(
        &snapshot,
        &directory,
        &name,
        now_ms(),
        &app.read_options.cancellation,
    )?;
    Ok(AiExport {
        path: NativePath::from_path(&parent.join("Commit Command Backups").join(name)),
        manifest_fingerprint,
    })
}
