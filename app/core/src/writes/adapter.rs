//! Controlled write subprocesses are separate from M1 inspection policy.
use super::*;
use std::ffi::OsString;
use std::process::{Command, Output};

pub(super) fn clean(program: &Path) -> Command {
    let mut command = Command::new(program);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", "");
    command
}
pub(super) fn git(
    root: &Path,
    args: &[OsString],
    index: Option<&Path>,
    objects: Option<&Path>,
    input: Option<Vec<u8>>,
    config: Option<&EffectiveConfig>,
    options: &ReadOptions,
) -> Result<Output> {
    let mut command = clean(Path::new("git"));
    command
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args([
            "--no-pager",
            "--literal-pathspecs",
            "-c",
            "core.hooksPath=",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "maintenance.auto=false",
            "-c",
            "gc.auto=0",
            "-c",
            "core.fsync=all",
            "-c",
            "core.fsyncMethod=fsync",
            "-c",
            "commit.gpgsign=false",
        ]);
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    if let Some(objects) = objects {
        command.env("GIT_OBJECT_DIRECTORY", objects);
    }
    if let Some(config) = config {
        command
            .env("GIT_AUTHOR_NAME", &config.name)
            .env("GIT_AUTHOR_EMAIL", &config.email)
            .env("GIT_COMMITTER_NAME", &config.name)
            .env("GIT_COMMITTER_EMAIL", &config.email);
        command
            .env("GIT_AUTHOR_DATE", &config.date)
            .env("GIT_COMMITTER_DATE", &config.date);
        if let Some(home) = &config.signer_home {
            command.env("GNUPGHOME", home.to_path()?);
        }
        if let Some(signing) = &config.signing {
            command.args([
                "-c",
                &format!("gpg.format={}", signing.format),
                "-c",
                &format!("user.signingkey={}", signing.key),
            ]);
            if let Some(program) = &signing.program {
                command.arg("-c").arg(format!(
                    "gpg.{}.program={}",
                    signing.format,
                    program.to_path()?.to_str().ok_or_else(|| fail(
                        ErrorKind::UnsupportedConfiguration,
                        "Signing program path must be UTF-8"
                    ))?
                ));
            }
        }
    }
    command.args(args);
    process::bounded_input(command, options, input)
}
pub(super) fn hash_file(root: &Path, file: std::fs::File, options: &ReadOptions) -> Result<String> {
    let mut command = clean(Path::new("git"));
    command
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "core.fsync=all",
            "-c",
            "core.fsyncMethod=fsync",
            "hash-object",
            "-w",
            "--stdin",
        ]);
    let output = process::bounded_file_input(command, options, file)?;
    if !output.status.success() {
        return Err(fail(
            ErrorKind::UnsupportedRepositoryState,
            "Git could not write the selected file object",
        ));
    }
    text(output.stdout)
}
pub(super) fn run(
    root: &Path,
    args: &[&str],
    index: Option<&Path>,
    objects: Option<&Path>,
    input: Option<Vec<u8>>,
    config: Option<&EffectiveConfig>,
    options: &ReadOptions,
) -> Result<Vec<u8>> {
    let output = git(
        root,
        &inspection::strings(args),
        index,
        objects,
        input,
        config,
        options,
    )?;
    if !output.status.success() {
        return Err(fail(
            ErrorKind::UnsupportedRepositoryState,
            "Controlled Git command failed; no success inferred",
        ));
    }
    Ok(output.stdout)
}
type ConfigSnapshot = (String, BTreeMap<String, String>, Vec<Vec<u8>>);
pub(super) fn config_snapshot(
    root: &Path,
    options: &ReadOptions,
    global: Option<&Path>,
) -> Result<ConfigSnapshot> {
    let mut command = clean(Path::new("git"));
    command.current_dir(root).args([
        "--no-pager",
        "config",
        "--null",
        "--show-origin",
        "--list",
        "--includes",
    ]);
    if let Some(global) = global {
        command
            .env("GIT_CONFIG_GLOBAL", global)
            .env("GIT_CONFIG_NOSYSTEM", "1");
    }
    let output = process::bounded_input(command, options, None)?;
    if !output.status.success() {
        return Err(fail(
            ErrorKind::UnsupportedConfiguration,
            "Cannot resolve effective configuration",
        ));
    }
    let hash = digest(&output.stdout);
    let mut result = BTreeMap::new();
    let mut fields = output.stdout.split(|b| *b == 0).collect::<Vec<_>>();
    if fields.last() == Some(&b"".as_slice()) {
        fields.pop();
    }
    if fields.len() % 2 != 0 {
        return Err(fail(
            ErrorKind::UnsupportedConfiguration,
            "Invalid configuration origin records",
        ));
    }
    let mut origins = Vec::new();
    for pair in fields.as_chunks::<2>().0 {
        let origin = pair[0].to_vec();
        if !origins.contains(&origin) {
            origins.push(origin);
        }
        let record = pair[1];
        let (key, value) = record
            .iter()
            .position(|b| *b == b'\n')
            .map(|n| (&record[..n], &record[n + 1..]))
            .unwrap_or((record, &[]));
        let key = String::from_utf8(key.to_vec())
            .map_err(|_| {
                fail(
                    ErrorKind::UnsupportedConfiguration,
                    "Non-UTF-8 configuration key",
                )
            })?
            .to_ascii_lowercase();
        if key.starts_with("filter.")
            && [".clean", ".smudge", ".process"]
                .iter()
                .any(|suffix| key.ends_with(suffix))
            || key == "index.sparse" && ![b"false".as_slice(), b"no", b"off", b"0"].contains(&value)
            || key == "gpg.ssh.defaultkeycommand"
        {
            return Err(fail(
                ErrorKind::UnsupportedConfiguration,
                "Content filters, sparse-index or dynamic signer commands are unsupported",
            ));
        }
        // Store only needed non-secret values. The hash binds the entire snapshot.
        if [
            "user.name",
            "user.email",
            "core.hookspath",
            "commit.gpgsign",
            "gpg.format",
            "user.signingkey",
            "gpg.program",
            "gpg.openpgp.program",
            "gpg.ssh.program",
            "extensions.refstorage",
            "core.autocrlf",
            "core.attributesfile",
            "core.safecrlf",
            "core.filemode",
        ]
        .contains(&key.as_str())
        {
            result.insert(
                key,
                String::from_utf8(value.to_vec()).map_err(|_| {
                    fail(
                        ErrorKind::UnsupportedConfiguration,
                        "Non-UTF-8 policy value",
                    )
                })?,
            );
        }
    }
    if result.contains_key("core.hookspath") {
        let mut command = clean(Path::new("git"));
        command
            .current_dir(root)
            .args(["config", "--path", "--get", "core.hooksPath"]);
        if let Some(global) = global {
            command
                .env("GIT_CONFIG_GLOBAL", global)
                .env("GIT_CONFIG_NOSYSTEM", "1");
        }
        let output = process::bounded_input(command, options, None)?;
        if !output.status.success() {
            return Err(fail(
                ErrorKind::UnsupportedConfiguration,
                "Cannot resolve hooksPath",
            ));
        }
        result.insert("core.hookspath".into(), text(output.stdout)?);
    }
    Ok((hash, result, origins))
}
pub(super) fn text(bytes: Vec<u8>) -> Result<String> {
    inspection::output_text(bytes)
}
pub(super) fn program_fingerprint(path: &Path) -> Result<String> {
    let path = fs::canonicalize(path)?;
    let bytes = inspection::file_bytes(&path, 64 * 1024 * 1024)?;
    Ok(digest(
        &[
            path.as_os_str().as_encoded_bytes(),
            inspection::fs_identity(&path)?.as_bytes(),
            &bytes,
        ]
        .concat(),
    ))
}
pub(super) fn find_program(name: &str) -> Result<PathBuf> {
    let path = Path::new(name);
    if path.components().count() > 1 {
        return fs::canonicalize(path).map_err(|_| {
            fail(
                ErrorKind::SigningFailed,
                "Configured executable is unavailable",
            )
        });
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .map(fs::canonicalize)
        .transpose()?
        .ok_or_else(|| {
            fail(
                ErrorKind::SigningFailed,
                "Required executable is unavailable",
            )
        })
}
