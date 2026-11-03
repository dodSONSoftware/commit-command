use super::*;
use sha2::Digest;
use std::os::unix::fs::MetadataExt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Entry {
    pub path: NativePath,
    pub mode: String,
    pub oid: String,
}
pub(super) fn index_entries(bytes: &[u8], format: &ObjectFormat) -> Result<Vec<Entry>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let hash_len = if *format == ObjectFormat::Sha1 {
        20
    } else {
        32
    };
    if bytes.len() < 12 + hash_len || &bytes[..4] != b"DIRC" {
        return Err(unsupported("Invalid index header"));
    }
    let payload = &bytes[..bytes.len() - hash_len];
    let checksum = if hash_len == 20 {
        sha1::Sha1::digest(payload).to_vec()
    } else {
        sha2::Sha256::digest(payload).to_vec()
    };
    if checksum != bytes[bytes.len() - hash_len..] {
        return Err(unsupported("Index checksum mismatch"));
    }
    let number = |offset| {
        u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("bounded header"),
        )
    };
    if ![2, 3].contains(&number(4)) {
        return Err(unsupported(
            "Only ordinary index v2/v3 writes are supported",
        ));
    }
    let count = number(8) as usize;
    if count > 100_000 {
        return Err(unsupported("Index entry limit exceeded"));
    }
    let mut entries = Vec::new();
    let mut offset = 12;
    for _ in 0..count {
        let start = offset;
        let fixed = 40 + hash_len + 2;
        if offset + fixed > payload.len() {
            return Err(unsupported("Truncated index entry"));
        }
        let mode = format!("{:o}", number(offset + 24));
        if !["100644", "100755", "120000", "160000"].contains(&mode.as_str()) {
            return Err(unsupported("Unsupported index mode"));
        }
        let oid = bytes[offset + 40..offset + 40 + hash_len]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let flags = u16::from_be_bytes(
            bytes[offset + 40 + hash_len..offset + fixed]
                .try_into()
                .expect("bounded flags"),
        );
        if flags & 0xb000 != 0 {
            return Err(unsupported(
                "Unmerged or assume-valid index entries are unsupported",
            ));
        }
        offset += fixed;
        if flags & 0x4000 != 0 {
            if number(4) != 3 || offset + 2 > payload.len() || bytes[offset..offset + 2] != [0, 0] {
                return Err(unsupported("Extended index flags are unsupported"));
            }
            offset += 2;
        }
        let end = bytes[offset..payload.len()]
            .iter()
            .position(|b| *b == 0)
            .map(|n| n + offset)
            .ok_or_else(|| unsupported("Unterminated index path"))?;
        let path = NativePath::from_git(&bytes[offset..end])?;
        path.relative()?;
        offset = start + (end + 1 - start).div_ceil(8) * 8;
        if offset > payload.len() {
            return Err(unsupported("Invalid index padding"));
        }
        entries.push(Entry { path, mode, oid });
    }
    while offset < payload.len() {
        if offset + 8 > payload.len() {
            return Err(unsupported("Invalid index extension"));
        }
        let size = number(offset + 4) as usize;
        if &bytes[offset..offset + 4] != b"TREE" {
            return Err(unsupported(
                "Unsupported index extension (split/sparse/cache flags)",
            ));
        }
        offset = offset
            .checked_add(8 + size)
            .filter(|end| *end <= payload.len())
            .ok_or_else(|| unsupported("Invalid index extension length"))?;
    }
    Ok(entries)
}
pub(super) fn original_index(meta: &Metadata) -> Result<Vec<u8>> {
    let path = meta.git_dir.to_path()?.join("index");
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            let bytes = inspection::file_bytes(&path, 16 * 1024 * 1024)?;
            if bytes.is_empty() {
                return Err(unsupported("Empty existing index"));
            }
            Ok(bytes)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}
pub(super) fn binding(meta: &Metadata) -> Result<String> {
    let mut value = serde_json::to_vec(meta)?;
    for path in [&meta.root, &meta.git_dir, &meta.common_dir] {
        let path = path.to_path()?;
        let m = fs::metadata(path)?;
        value.extend_from_slice(format!("{}:{}:{}", m.uid(), m.gid(), m.mode()).as_bytes());
    }
    Ok(digest(&value))
}
type RepositoryState = (Vec<u8>, Vec<Entry>, String, Option<String>);
pub(super) fn preflight(meta: &Metadata, context: &Options) -> Result<RepositoryState> {
    preflight_locked(meta, context, false)
}
pub(super) fn preflight_locked(
    meta: &Metadata,
    context: &Options,
    owned_locks: bool,
) -> Result<RepositoryState> {
    let options = &context.process;
    #[cfg(not(target_os = "macos"))]
    return Err(unsupported(
        "Production controlled writes are validated only on macOS/APFS",
    ));
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let root = meta.root.to_path()?;
        for location in [&meta.root, &meta.git_dir, &meta.common_dir] {
            let location = location.to_path()?;
            let native = CString::new(location.as_os_str().as_bytes())
                .map_err(|_| unsupported("Invalid root"))?;
            let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
            if unsafe { libc::statfs(native.as_ptr(), stat.as_mut_ptr()) } != 0 {
                return Err(unsupported("Cannot establish filesystem support"));
            }
            let stat = unsafe { stat.assume_init() };
            let name = unsafe { std::ffi::CStr::from_ptr(stat.f_fstypename.as_ptr()) }.to_bytes();
            if name != b"apfs" || stat.f_flags & libc::MNT_LOCAL as u32 == 0 {
                return Err(unsupported(
                    "Controlled writes require a local APFS filesystem",
                ));
            }
        }
        let current = inspection::repository_metadata(&root)?;
        if binding(&current)? != binding(meta)? {
            return Err(fail(
                ErrorKind::UntrustedRepository,
                "Repository identity changed",
            ));
        }
        let git_dir = meta.git_dir.to_path()?;
        safe_metadata_paths(meta)?;
        for marker in [
            "MERGE_HEAD",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "rebase-merge",
            "rebase-apply",
            "sequencer",
            "BISECT_LOG",
            "SQUASH_MSG",
        ] {
            if git_dir.join(marker).exists() {
                return Err(unsupported("In-progress Git operation is unsupported"));
            }
        }
        let (hash, config, _) =
            adapter::config_snapshot(&root, options, context.global_config.as_deref())?;
        let _ = hash;
        if config
            .get("extensions.refstorage")
            .is_some_and(|value| value != "files")
        {
            return Err(unsupported("Unvalidated ref backend"));
        }
        if config.get("core.autocrlf").is_some_and(|v| {
            !matches!(
                v.to_ascii_lowercase().as_str(),
                "false" | "no" | "off" | "0"
            )
        }) {
            return Err(unsupported("Working-tree conversion is unsupported"));
        }
        if config.contains_key("core.attributesfile") {
            return Err(unsupported(
                "Custom attributesFile configuration is unsupported",
            ));
        }
        let branch = adapter::text(adapter::run(
            &root,
            &["symbolic-ref", "-q", "HEAD"],
            None,
            None,
            None,
            None,
            options,
        )?)?;
        if inspection::file_bytes(&git_dir.join("HEAD"), 65536)?
            != format!("ref: {branch}\n").as_bytes()
        {
            return Err(unsupported("Symbolic branch aliases are unsupported"));
        }
        if !branch.starts_with("refs/heads/") {
            return Err(unsupported("A checked-out local branch is required"));
        }
        let worktrees = adapter::run(
            &root,
            &["worktree", "list", "--porcelain", "-z"],
            None,
            None,
            None,
            None,
            options,
        )?;
        let checked_out = format!("branch {branch}");
        if worktrees
            .split(|b| *b == 0)
            .filter(|field| *field == checked_out.as_bytes())
            .count()
            > 1
        {
            return Err(unsupported(
                "Branch is checked out in another linked worktree",
            ));
        }
        let output = adapter::git(
            &root,
            &inspection::strings(&["rev-parse", "--verify", "--quiet", "HEAD"]),
            None,
            None,
            None,
            None,
            options,
        )?;
        let head = if output.status.success() {
            Some(adapter::text(output.stdout)?)
        } else if output.status.code() == Some(1) {
            None
        } else {
            return Err(unsupported("Cannot resolve HEAD"));
        };
        let bytes = original_index(meta)?;
        let entries = index_entries(&bytes, &meta.object_format)?;
        if !owned_locks
            && (git_dir.join("index.lock").exists() || git_dir.join("HEAD.lock").exists())
        {
            return Err(fail(ErrorKind::Busy, "Foreign or unresolved index lock"));
        }
        Ok((bytes, entries, branch, head))
    }
}
pub(super) fn hooks(meta: &Metadata, values: &BTreeMap<String, String>) -> Result<Vec<Hook>> {
    named_hooks(
        meta,
        values,
        &[
            "pre-commit",
            "prepare-commit-msg",
            "commit-msg",
            "post-commit",
            "post-index-change",
            "reference-transaction",
        ],
    )
}
pub(super) fn named_hooks(
    meta: &Metadata,
    values: &BTreeMap<String, String>,
    names: &[&str],
) -> Result<Vec<Hook>> {
    let root = meta.root.to_path()?;
    if values
        .get("core.hookspath")
        .is_some_and(|path| path.is_empty())
    {
        return Ok(Vec::new());
    }
    let directory = if let Some(path) = values.get("core.hookspath") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    } else {
        meta.common_dir.to_path()?.join("hooks")
    };
    let mut result = Vec::new();
    for &name in names {
        let source = directory.join(name);
        if !source.exists() {
            continue;
        }
        let executable = fs::metadata(&source)?.mode() & 0o111 != 0;
        let resolved = fs::canonicalize(&source)?;
        let content = inspection::file_bytes(&resolved, 2 * 1024 * 1024)?;
        let framework = String::from_utf8_lossy(&content).to_ascii_lowercase();
        result.push(Hook {
            name: name.into(),
            source: NativePath::from_path(&source),
            resolved: NativePath::from_path(&resolved),
            executable,
            fingerprint: adapter::program_fingerprint(&resolved)?,
            framework: framework.contains("husky")
                || framework.contains("lefthook")
                || directory.to_string_lossy().contains(".husky"),
        });
    }
    Ok(result)
}
pub(super) fn effective(
    meta: &Metadata,
    preference: SigningPreference,
    options: &Options,
) -> Result<EffectiveConfig> {
    let root = meta.root.to_path()?;
    let (config_hash, values, config_origins) =
        adapter::config_snapshot(&root, &options.process, options.global_config.as_deref())?;
    let name = values
        .get("user.name")
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .ok_or_else(|| unsupported("Explicit user.name is required"))?;
    let email = values
        .get("user.email")
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .ok_or_else(|| unsupported("Explicit user.email is required"))?;
    if [&name, &email]
        .iter()
        .any(|s| s.chars().any(|c| c.is_control() || matches!(c, '<' | '>')))
    {
        return Err(unsupported("Invalid commit identity"));
    }
    let hooks = hooks(meta, &values)?;
    let signing = if preference == SigningPreference::Unsigned
        || !boolean(&values, "commit.gpgsign", false)?
    {
        None
    } else {
        let format = values
            .get("gpg.format")
            .map(String::as_str)
            .unwrap_or("openpgp");
        if !["openpgp", "ssh"].contains(&format) {
            return Err(unsupported("Unsupported signing format"));
        }
        let key = values.get("user.signingkey").cloned().ok_or_else(|| {
            fail(
                ErrorKind::SigningFailed,
                "An explicit signing key is required",
            )
        })?;
        let program = adapter::find_program(
            values
                .get(&format!("gpg.{format}.program"))
                .or_else(|| {
                    if format == "openpgp" {
                        values.get("gpg.program")
                    } else {
                        None
                    }
                })
                .map(String::as_str)
                .unwrap_or(if format == "ssh" { "ssh-keygen" } else { "gpg" }),
        )?;
        let program_hash = adapter::program_fingerprint(&program)?;
        let (fingerprint, public_key) = if format == "openpgp" {
            if ![40, 64].contains(&key.len()) || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "OpenPGP requires a full key fingerprint",
                ));
            }
            (key.to_ascii_uppercase(), None)
        } else {
            if key.starts_with("key::") {
                return Err(unsupported("Inline/dynamic SSH keys are not yet supported"));
            }
            let key_path = PathBuf::from(&key);
            if !key_path.is_absolute() {
                return Err(unsupported("SSH key paths must be absolute"));
            }
            let public_path = if key_path
                .extension()
                .is_some_and(|extension| extension == "pub")
            {
                key_path.clone()
            } else {
                PathBuf::from(format!("{}.pub", key_path.display()))
            };
            let public = inspection::file_bytes(&public_path, 65536)?;
            if !public.starts_with(b"ssh-") {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "Expected a plain SSH public key",
                ));
            }
            let mut command = adapter::clean(Path::new("/usr/bin/ssh-keygen"));
            command.args(["-lf"]).arg(&public_path);
            let output = process::bounded_input(command, &options.process, None)?;
            if !output.status.success() {
                return Err(fail(
                    ErrorKind::SigningFailed,
                    "Cannot inspect SSH key fingerprint",
                ));
            }
            let fingerprint = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .nth(1)
                .filter(|s| s.starts_with("SHA256:"))
                .ok_or_else(|| fail(ErrorKind::SigningFailed, "Invalid SSH key fingerprint"))?
                .to_owned();
            (fingerprint, Some(public))
        };
        Some(Signing {
            format: format.into(),
            key,
            program: Some(NativePath::from_path(&program)),
            program_hash,
            fingerprint,
            public_key,
        })
    };
    if signing.as_ref().is_some_and(|s| s.format == "openpgp") && options.signer_home.is_none() {
        return Err(unsupported(
            "OpenPGP requires an existing explicit or standard signer home",
        ));
    }
    let signer_home = options
        .signer_home
        .as_ref()
        .map(|path| fs::canonicalize(path).map(|path| NativePath::from_path(&path)))
        .transpose()?;
    Ok(EffectiveConfig {
        config_hash,
        config_origins,
        name,
        email,
        date: format!("{} +0000", now_ms() / 1000),
        hooks,
        signing,
        signer_home,
        file_mode: boolean(&values, "core.filemode", true)?,
    })
}
pub(super) fn validate_hook_policy(config: &EffectiveConfig, policy: HookPolicy) -> Result<()> {
    validate_hook_manifest(&config.hooks, policy)
}
pub(super) fn validate_hook_manifest(hooks: &[Hook], policy: HookPolicy) -> Result<()> {
    if policy == HookPolicy::Execute
        && hooks
            .iter()
            .any(|hook| hook.executable && (hook.name == "reference-transaction" || hook.framework))
    {
        return Err(unsupported("Execute mode does not support reference-transaction hooks or unvalidated hook managers; explicit reviewed bypass-all is available"));
    }
    Ok(())
}

fn boolean(values: &BTreeMap<String, String>, key: &str, default: bool) -> Result<bool> {
    match values.get(key).map(|v| v.to_ascii_lowercase()).as_deref() {
        None => Ok(default),
        Some("" | "true" | "yes" | "on" | "1") => Ok(true),
        Some("false" | "no" | "off" | "0") => Ok(false),
        _ => Err(unsupported("Invalid boolean policy configuration")),
    }
}
// Reject metadata redirection before creating locks, objects, refs or reflogs.
// Hard links and hostile same-user replacement remain outside the supported envelope.
pub(super) fn safe_metadata_paths(meta: &Metadata) -> Result<()> {
    for base in [meta.git_dir.to_path()?, meta.common_dir.to_path()?] {
        for relative in ["HEAD", "index", "objects", "refs", "logs", "packed-refs"] {
            let path = base.join(relative);
            if let Ok(m) = fs::symlink_metadata(&path) {
                if m.is_symlink() {
                    return Err(unsupported(
                        "Symlinked writable Git metadata is unsupported",
                    ));
                }
                if m.is_dir() {
                    safe_directory(&path, &mut 200_000, 0)?;
                }
            }
        }
    }
    Ok(())
}
fn safe_directory(path: &Path, remaining: &mut usize, depth: usize) -> Result<()> {
    if depth > 32 {
        return Err(unsupported("Writable metadata depth limit exceeded"));
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        *remaining = remaining
            .checked_sub(1)
            .ok_or_else(|| unsupported("Writable metadata entry limit exceeded"))?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(unsupported(
                "Symlinked writable Git metadata is unsupported",
            ));
        }
        if kind.is_dir() {
            safe_directory(&entry.path(), remaining, depth + 1)?;
        }
    }
    Ok(())
}
