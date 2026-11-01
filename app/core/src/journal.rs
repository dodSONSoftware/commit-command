//! Minimal durable journal. This is a research implementation, not a transactional database.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub format_version: u32,
    pub state: String,
    pub repo: PathBuf,
    pub git_dir: PathBuf,
    pub git_ref: String,
    pub old_head: String,
    pub new_head: Option<String>,
    pub tree: String,
    pub message: String,
    pub selection: BTreeMap<String, String>,
    pub worktree_fingerprints: BTreeMap<String, String>,
    pub original_index_entries: BTreeMap<String, (String, String)>,
    pub reconciled_index_hash: String,
    pub scratch: PathBuf,
    pub hooks_executed: Vec<String>,
    pub hooks_enabled: bool,
    pub sign: bool,
    pub post_hook_invoked: bool,
    pub post_hook_result: Option<String>,
}

pub fn save(path: &Path, value: &Journal) -> Result<()> {
    fs::create_dir_all(path.parent().ok_or_else(|| Error::new("journal has no parent"))?)?;
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(&tmp)?;
    if let Err(e) = (|| -> std::io::Result<()> { file.write_all(&bytes)?; file.write_all(b"\n")?; file.sync_all() })() {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    drop(file);
    fs::rename(&tmp, path)?;
    sync_directory(path.parent().unwrap());
    Ok(())
}

pub fn load(path: &Path) -> Result<Journal> {
    let value: Journal = serde_json::from_slice(&fs::read(path)?)?;
    if value.format_version != 1 { return Err(Error::new("Unsupported journal version")); }
    Ok(value)
}

pub fn sync_directory(path: &Path) {
    if let Ok(f) = File::open(path) { let _ = f.sync_all(); }
}
