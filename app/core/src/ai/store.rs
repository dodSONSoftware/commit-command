//! Lazy, atomic AI metadata generations outside repositories. Credentials are excluded.
use super::config::Settings;
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use uuid::Uuid;
const LIMIT: usize = 2 * 1024 * 1024;
fn invalid() -> Error {
    Error::with_kind(ErrorKind::Storage, "AI settings cannot be read safely; original files preserved. Manual workflows remain available")
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    version: u32,
    generation: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Disk {
    settings: Settings,
    bodies: Vec<Body>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Body {
    Profile { id: String, file: String },
    Instructions { index: usize, file: String },
    Template { id: String, file: String },
}

#[cfg(unix)]
mod fs {
    use super::*;
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::{MetadataExt, OpenOptionsExt},
        },
    };
    pub(super) struct Dir(File);
    fn owned(file: &File, directory: bool) -> Result<()> {
        let m = file.metadata()?;
        if m.uid() != unsafe { libc::geteuid() }
            || (if directory {
                !m.is_dir()
            } else {
                !m.is_file() || m.nlink() != 1
            })
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn component(name: &str) -> Result<CString> {
        if name.is_empty() || name == "." || name == ".." || name.contains('/') {
            return Err(invalid());
        }
        CString::new(name).map_err(|_| invalid())
    }
    impl Dir {
        pub(super) fn root(path: &Path) -> Result<Self> {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(path)?;
            owned(&file, true)?;
            Ok(Self(file))
        }
        pub(super) fn child(&self, name: &str, create: bool) -> Result<Self> {
            let name = component(name)?;
            if create && unsafe { libc::mkdirat(self.0.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                let e = std::io::Error::last_os_error();
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(e.into());
                }
            }
            let fd = unsafe {
                libc::openat(
                    self.0.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let file = unsafe { File::from_raw_fd(fd) };
            owned(&file, true)?;
            Ok(Self(file))
        }
        pub(super) fn read(&self, name: &str, max: usize) -> Result<Vec<u8>> {
            let name = component(name)?;
            let fd = unsafe {
                libc::openat(
                    self.0.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let file = unsafe { File::from_raw_fd(fd) };
            owned(&file, false)?;
            if file.metadata()?.len() > max as u64 {
                return Err(invalid());
            }
            let mut bytes = Vec::new();
            file.take(max as u64 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > max {
                return Err(invalid());
            }
            Ok(bytes)
        }
        pub(super) fn create(&self, name: &str, bytes: &[u8]) -> Result<()> {
            let name = component(name)?;
            let fd = unsafe {
                libc::openat(
                    self.0.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o600,
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            file.write_all(bytes)?;
            file.sync_all()?;
            #[cfg(target_os = "macos")]
            if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(())
        }
        pub(super) fn publish(&self, temporary: &str) -> Result<()> {
            let name = component(temporary)?;
            let destination = component("current.toml")?;
            if unsafe {
                libc::renameat(
                    self.0.as_raw_fd(),
                    name.as_ptr(),
                    self.0.as_raw_fd(),
                    destination.as_ptr(),
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            self.sync()
        }
        pub(super) fn sync(&self) -> Result<()> {
            self.0.sync_all()?;
            Ok(())
        }
    }
}
#[cfg(unix)]
fn read(root: &Path) -> Result<Settings> {
    let state = fs::Dir::root(root)?;
    let ai = match state.child("ai", false) {
        Ok(dir) => dir,
        Err(e)
            if e.kind == ErrorKind::Io
                && matches!(root.join("ai").symlink_metadata(), Err(e) if e.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(Settings::default())
        }
        Err(e) => return Err(e),
    };
    let pointer = match ai.read("current.toml", 4096) {
        Ok(bytes) => bytes,
        Err(e)
            if e.kind == ErrorKind::Io
                && matches!(root.join("ai/current.toml").symlink_metadata(), Err(e) if e.kind() == std::io::ErrorKind::NotFound) =>
        {
            return Ok(Settings::default())
        }
        Err(e) => return Err(e),
    };
    let pointer: Pointer = toml::from_str(std::str::from_utf8(&pointer).map_err(|_| invalid())?)
        .map_err(|_| invalid())?;
    if pointer.version != 1 {
        return Err(Error::with_kind(
            ErrorKind::NewerSchema,
            "Unsupported AI generation version",
        ));
    }
    Uuid::parse_str(&pointer.generation).map_err(|_| invalid())?;
    let generation = ai.child(&pointer.generation, false)?;
    let raw = generation.read("settings.toml", LIMIT)?;
    let mut disk: Disk =
        toml::from_str(std::str::from_utf8(&raw).map_err(|_| invalid())?).map_err(|_| invalid())?;
    if disk.bodies.len()
        != disk.settings.profiles.len()
            + disk.settings.templates.len()
            + disk.settings.instructions.len()
        || disk.bodies.len() > 1152
    {
        return Err(invalid());
    }
    let mut total = raw.len();
    let mut seen = std::collections::BTreeSet::new();
    for body in disk.bodies {
        let file = match &body {
            Body::Profile { file, .. }
            | Body::Instructions { file, .. }
            | Body::Template { file, .. } => file,
        };
        if !seen.insert(file.clone()) {
            return Err(invalid());
        }
        let bytes = generation.read(file, 65536)?;
        total += bytes.len();
        if total > LIMIT {
            return Err(invalid());
        }
        let text = String::from_utf8(bytes).map_err(|_| invalid())?;
        match body {
            Body::Profile { id, file } => {
                if file != format!("profile-{id}.md") {
                    return Err(invalid());
                }
                let p = disk
                    .settings
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == id)
                    .ok_or_else(invalid)?;
                if !p.system_message.is_empty() {
                    return Err(invalid());
                }
                p.system_message = text;
            }
            Body::Template { id, file } => {
                if file != format!("template-{id}.md") {
                    return Err(invalid());
                }
                let t = disk
                    .settings
                    .templates
                    .iter_mut()
                    .find(|t| t.id == id)
                    .ok_or_else(invalid)?;
                if !t.rules.instructions.is_empty() {
                    return Err(invalid());
                }
                t.rules.instructions = text;
            }
            Body::Instructions { index, file } => {
                if file != format!("instructions-{index}.md") {
                    return Err(invalid());
                }
                let i = disk
                    .settings
                    .instructions
                    .get_mut(index)
                    .ok_or_else(invalid)?;
                if !i.text.is_empty() {
                    return Err(invalid());
                }
                i.text = text;
            }
        }
    }
    disk.settings.validate()?;
    Ok(disk.settings)
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(rename = "AiSettingsSnapshot")]
pub struct Snapshot {
    pub settings: Settings,
    pub fingerprint: String,
}
/// Called only by explicit AI operations; never part of application startup.
pub fn load(root: &Path) -> Result<Snapshot> {
    #[cfg(unix)]
    {
        let settings = read(root).map_err(|e| {
            if e.kind == ErrorKind::NewerSchema {
                e
            } else {
                invalid()
            }
        })?;
        let fingerprint = crate::domain::digest(&serde_json::to_vec(&settings)?);
        Ok(Snapshot {
            settings,
            fingerprint,
        })
    }
    #[cfg(not(unix))]
    {
        let _ = root;
        Err(invalid())
    }
}
/// Caller owns the application's exclusive writer lock. Old generations are retained.
pub fn save(root: &Path, expected_fingerprint: &str, settings: &Settings) -> Result<()> {
    settings.validate()?;
    if load(root)?.fingerprint != expected_fingerprint {
        return Err(Error::with_kind(
            ErrorKind::StaleReview,
            "AI settings changed on disk; reload before saving",
        ));
    }
    #[cfg(unix)]
    {
        let state = fs::Dir::root(root)?;
        let ai = state.child("ai", true)?;
        let id = Uuid::new_v4().to_string();
        let generation = ai.child(&id, true)?;
        let body_bytes: usize = settings
            .profiles
            .iter()
            .map(|p| p.system_message.len())
            .sum::<usize>()
            + settings
                .templates
                .iter()
                .map(|t| t.rules.instructions.len())
                .sum::<usize>()
            + settings
                .instructions
                .iter()
                .map(|i| i.text.len())
                .sum::<usize>();
        let mut disk = Disk {
            settings: settings.clone(),
            bodies: Vec::new(),
        };
        for p in &mut disk.settings.profiles {
            let file = format!("profile-{}.md", p.id);
            generation.create(&file, p.system_message.as_bytes())?;
            p.system_message.clear();
            disk.bodies.push(Body::Profile {
                id: p.id.clone(),
                file,
            });
        }
        for t in &mut disk.settings.templates {
            let file = format!("template-{}.md", t.id);
            generation.create(&file, t.rules.instructions.as_bytes())?;
            t.rules.instructions.clear();
            disk.bodies.push(Body::Template {
                id: t.id.clone(),
                file,
            });
        }
        for (index, i) in disk.settings.instructions.iter_mut().enumerate() {
            let file = format!("instructions-{index}.md");
            generation.create(&file, i.text.as_bytes())?;
            i.text.clear();
            disk.bodies.push(Body::Instructions { index, file });
        }
        let raw = toml::to_string_pretty(&disk).map_err(|_| invalid())?;
        if raw.len() + body_bytes > LIMIT {
            return Err(invalid());
        }
        generation.create("settings.toml", raw.as_bytes())?;
        generation.sync()?;
        ai.sync()?;
        let temporary = format!("pointer-{id}.tmp");
        let pointer = toml::to_string(&Pointer {
            version: 1,
            generation: id,
        })
        .map_err(|_| invalid())?;
        ai.create(&temporary, pointer.as_bytes())?;
        if load(root)?.fingerprint != expected_fingerprint {
            return Err(Error::with_kind(ErrorKind::StaleReview, "AI settings changed while saving; completed generation retained without activation"));
        }
        ai.publish(&temporary)?;
        state.sync()?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = root;
        Err(invalid())
    }
}
