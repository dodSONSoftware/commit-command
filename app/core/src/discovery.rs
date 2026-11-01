//! Read-only recursive repository discovery. Never follows directory symlinks.
use crate::git;
use crate::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Repository {
    pub path: PathBuf,
    pub git_dir: String,
    pub branch: Option<String>,
    pub status: String,
}

pub fn discover(root: &Path) -> Result<Vec<Repository>> {
    let root = fs::canonicalize(root)?;
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        let marker = dir.join(".git");
        if marker.is_dir() || marker.is_file() {
            if let Ok(git_dir) = git::run(&dir, &["rev-parse", "--absolute-git-dir"], None) {
                found.push(Repository {
                    path: dir.clone(), git_dir,
                    branch: git::run(&dir, &["symbolic-ref", "-q", "--short", "HEAD"], None).ok(),
                    status: git::status(&dir).unwrap_or_else(|e| format!("status unavailable: {e}")),
                });
            }
        }
        let rd = match fs::read_dir(&dir) { Ok(rd) => rd, Err(_) => continue };
        for entry in rd.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if [".git", "node_modules", "target", ".venv", ".gitfleet-rust-lab"].contains(&name.as_ref()) { continue; }
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() && !ft.is_symlink() { pending.push(entry.path()); }
            }
        }
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}
