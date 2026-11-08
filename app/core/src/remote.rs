//! Read-only destination inspection. This module does not contact remote servers.
pub mod fetch;
pub mod observation;
pub mod push;
pub mod push_batch;
pub mod push_check;
pub mod push_publication;
pub mod upstream;
pub(crate) mod urls;
use crate::{domain::*, inspection, process, Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

/// Trusted host policy. Renderer requests cannot set transport roots or CA files.
#[derive(Debug, Clone, Default)]
pub struct TransportOptions {
    pub credential_bridge: Option<PathBuf>,
    pub credential_home: Option<PathBuf>,
    pub local_fixture_root: Option<PathBuf>,
    pub ca_file: Option<PathBuf>,
    pub ssh: Option<SshPolicy>,
    pub https_credentials: Option<HttpsCredentialPolicy>,
}

/// Explicit host policy; no private key or credential is accepted from IPC.
#[derive(Debug, Clone)]
pub struct SshPolicy {
    pub known_hosts: PathBuf,
    pub public_identity: PathBuf,
    pub agent_socket: PathBuf,
}

/// A trusted executable implementing Git's username/password helper protocol.
#[derive(Debug, Clone)]
pub struct HttpsCredentialPolicy {
    pub home: PathBuf,
    pub helper: PathBuf,
    pub bridge: PathBuf,
    pub account: String,
}

pub(crate) struct ResolvedDestination {
    pub inspection: Inspection,
    pub url: String,
    pub transport: Transport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Local,
    Https,
    Ssh,
    Git,
    RemoteHelper,
    Unsupported,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Destination {
    pub display: String,
    pub transport: Transport,
    pub credentials_hidden: bool,
    pub restrictions: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigOrigin {
    pub scope: String,
    pub file: Option<NativePath>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Remote {
    pub name: String,
    pub fetch: Vec<Destination>,
    pub push: Vec<Destination>,
    pub origins: Vec<ConfigOrigin>,
    pub fetch_refspec_count: usize,
    pub push_refspec_count: usize,
    pub issues: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Inspection {
    pub remotes: Vec<Remote>,
    pub branch: Option<String>,
    pub upstream_remote: Option<String>,
    pub upstream_ref: Option<String>,
    pub suggested_push_remote: Option<String>,
    pub issues: Vec<String>,
    pub credential_helper_configured: bool,
    pub ssh_command_configured: bool,
    pub proxy_configured: bool,
    pub tls_verification_disabled: bool,
    pub config_digest: String,
    pub generation: String,
    pub observed_at_ms: u64,
}
struct ConfigValue {
    key: String,
    value: Vec<u8>,
    origin: ConfigOrigin,
}
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::UnsupportedConfiguration, message)
}
fn clean(root: &Path, global: Option<&Path>) -> Command {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .current_dir(root)
        .arg("--no-pager")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_ALLOW_PROTOCOL", "");
    if let Some(path) = global {
        command
            .env("GIT_CONFIG_GLOBAL", path)
            .env("GIT_CONFIG_NOSYSTEM", "1");
    }
    command
}
fn config(bytes: &[u8]) -> Result<Vec<ConfigValue>> {
    let fields = bytes
        .strip_suffix(&[0])
        .ok_or_else(|| invalid("Unterminated configuration output"))?
        .split(|b| *b == 0)
        .collect::<Vec<_>>();
    if fields.len() % 3 != 0 {
        return Err(invalid("Invalid configuration origin output"));
    }
    let mut records = Vec::new();
    if fields.len() / 3 > 10_000 {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "Remote configuration exceeds 10,000 records",
        ));
    }
    for fields in fields.as_chunks::<3>().0 {
        let scope =
            std::str::from_utf8(fields[0]).map_err(|_| invalid("Invalid configuration scope"))?;
        if !["system", "global", "local", "worktree", "command"].contains(&scope) {
            return Err(invalid("Unknown configuration scope"));
        }
        let origin = ConfigOrigin {
            scope: scope.into(),
            file: fields[1]
                .strip_prefix(b"file:")
                .map(NativePath::from_git)
                .transpose()?,
        };
        let (key, value) = fields[2]
            .iter()
            .position(|b| *b == b'\n')
            .map(|n| (&fields[2][..n], &fields[2][n + 1..]))
            .unwrap_or((fields[2], &[]));
        let key = std::str::from_utf8(key)
            .map_err(|_| invalid("Non-UTF-8 configuration key"))?
            .to_owned();
        records.push(ConfigValue {
            key,
            value: value.to_vec(),
            origin,
        });
    }
    Ok(records)
}
fn remote_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-/".contains(&b))
}
pub(crate) fn destination(raw: &str) -> Destination {
    if urls::scp_parts(raw).is_some() {
        return Destination {
            display: raw.into(),
            transport: Transport::Ssh,
            credentials_hidden: false,
            restrictions: Vec::new(),
        };
    }
    let mut result = Destination {
        display: String::new(),
        transport: Transport::Unsupported,
        credentials_hidden: false,
        restrictions: Vec::new(),
    };
    if raw.is_empty() || raw.chars().any(char::is_control) || raw.starts_with('-') {
        result.display = "[invalid destination withheld]".into();
        result
            .restrictions
            .push("Destination is empty, option-like or contains control characters".into());
    } else if raw.contains("::") && !raw.contains("://") {
        result.transport = Transport::RemoteHelper;
        result.display = "[remote helper destination withheld]".into();
        result
            .restrictions
            .push("Arbitrary remote helpers are not authorized".into());
    } else if raw.contains("://") {
        match url::Url::parse(raw) {
            Ok(mut url) => {
                result.transport = match url.scheme() {
                    "https" => Transport::Https,
                    "ssh" => Transport::Ssh,
                    "git" => Transport::Git,
                    "file" => Transport::Local,
                    _ => Transport::Unsupported,
                };
                let public_ssh_login = result.transport == Transport::Ssh
                    && !url.username().starts_with('-')
                    && url
                        .username()
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b));
                result.credentials_hidden = (!url.username().is_empty() && !public_ssh_login)
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some();
                if result.credentials_hidden {
                    result.restrictions.push("Embedded user information, query or fragment is hidden; use a supported secure credential flow".into());
                    let _ = url.set_username("");
                    let _ = url.set_password(None);
                    url.set_query(None);
                    url.set_fragment(None);
                }
                result.display = url.to_string();
                if result.transport == Transport::Local
                    && (url.host_str().is_some_and(|h| h != "localhost")
                        || url.to_file_path().is_err())
                {
                    result
                        .restrictions
                        .push("Nonlocal or invalid file URL".into());
                }
            }
            Err(_) => {
                result.display = "[invalid URL withheld]".into();
                result.restrictions.push("Malformed URL".into());
            }
        }
    } else if raw
        .split('/')
        .next()
        .is_some_and(|prefix| prefix.contains(':'))
    {
        result.transport = Transport::Ssh;
        result.display = "[unsupported SCP-style destination withheld]".into();
        result.credentials_hidden = true;
        result.restrictions.push(
            "Use supported SCP syntax with an explicit plain login, or an explicit SSH URL".into(),
        );
    } else {
        result.transport = Transport::Local;
        result.display = raw.escape_default().to_string();
    }
    if matches!(result.transport, Transport::Git | Transport::Unsupported) {
        result
            .restrictions
            .push("Transport is not authorized for mutation".into());
    }
    result
}

/// `global` is trusted host configuration; never accepted from renderer requests.
/// No URL, helper, credential command, SSH command or server is executed by inspection.
pub fn inspect(
    repo: &Path,
    global: Option<&Path>,
    options: &inspection::ReadOptions,
) -> Result<Inspection> {
    let started = Instant::now();
    inspection::read(repo, options, |session| {
        let run = |args: &[&str]| -> Result<Vec<u8>> {
            options.check(started)?;
            let mut remaining = options.clone();
            remaining.timeout = options.timeout.saturating_sub(started.elapsed());
            let mut command = clean(&session.root, global);
            command.args(args);
            let output = process::bounded_input(command, &remaining, None)?;
            if !output.status.success() {
                return Err(invalid("Cannot resolve remote configuration"));
            }
            Ok(output.stdout)
        };
        let query = [
            "config",
            "--null",
            "--show-scope",
            "--show-origin",
            "--list",
            "--includes",
        ];
        let before = run(&query)?;
        let config = config(&before)?;
        // Reject line injection before asking Git for newline-delimited expanded URLs.
        for value in &config {
            let remote_url = value.key.starts_with("remote.")
                && (value.key.ends_with(".url") || value.key.ends_with(".pushurl"));
            if (remote_url || value.key.starts_with("url."))
                && (value.key.chars().any(char::is_control)
                    || value.value.iter().any(u8::is_ascii_control))
            {
                return Err(invalid(
                    "Remote URL configuration contains control characters",
                ));
            }
        }
        let mut grouped: BTreeMap<String, Vec<&ConfigValue>> = BTreeMap::new();
        let mut rewrites = Vec::new();
        for value in &config {
            if value.key.starts_with("url.") {
                rewrites.push(value);
            }
            if let Some(key) = value.key.strip_prefix("remote.") {
                if let Some((name, _)) = key.rsplit_once('.') {
                    if !remote_name(name) {
                        return Err(invalid("Unsupported remote name"));
                    }
                    grouped.entry(name.into()).or_default().push(value);
                }
            }
        }
        if grouped.len() > 64 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Remote inventory exceeds 64 remotes",
            ));
        }
        if rewrites.len() > 256 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Remote configuration exceeds 256 URL rewrite entries",
            ));
        }
        let mut remotes = Vec::new();
        let mut inspected_url_bytes = 0;
        for (name, values) in grouped {
            // Git's own parser validates the name; no interpretation as a command/refspec is permitted.
            session.git(&[
                "check-ref-format",
                &format!("refs/remotes/{name}/gitfleet-check"),
            ])?;
            let suffix = |suffix: &'static str| {
                values
                    .iter()
                    .copied()
                    .filter(move |v| v.key.ends_with(suffix))
            };
            let mut origins = Vec::new();
            for value in values.iter().copied().chain(rewrites.iter().copied()) {
                if !origins.contains(&value.origin) {
                    origins.push(value.origin.clone());
                }
            }
            let mut issues = Vec::new();
            let mut resolve = |push: bool| -> Result<Vec<Destination>> {
                if suffix(".url").next().is_none() {
                    issues.push("Remote has no fetch URL".into());
                    return Ok(Vec::new());
                }
                let args = if push {
                    vec!["remote", "get-url", "--push", "--all", "--", &name]
                } else {
                    vec!["remote", "get-url", "--all", "--", &name]
                };
                let output = run(&args)?;
                inspected_url_bytes += output.len();
                if inspected_url_bytes > 1024 * 1024 {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "Expanded remote URLs exceed 1 MiB",
                    ));
                }
                let raw = std::str::from_utf8(&output)
                    .map_err(|_| invalid("Non-UTF-8 remote URL is unsupported"))?;
                let urls = raw
                    .strip_suffix('\n')
                    .ok_or_else(|| invalid("Unterminated remote URL output"))?
                    .split('\n')
                    .collect::<Vec<_>>();
                if urls.len() > 32 {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "Remote exceeds 32 destinations",
                    ));
                }
                Ok(urls.into_iter().map(destination).collect())
            };
            let fetch = resolve(false)?;
            let push = resolve(true)?;
            if fetch.len() != 1 || push.len() != 1 {
                issues.push("A mutation requires one unambiguous destination; multiple URLs are not automatically chosen".into());
            }
            if suffix(".mirror")
                .any(|v| !matches!(v.value.as_slice(), b"false" | b"no" | b"off" | b"0"))
            {
                issues.push("Mirror remote requires unsupported broad ref authority".into());
            }
            if suffix(".uploadpack").next().is_some()
                || suffix(".receivepack").next().is_some()
                || suffix(".vcs").next().is_some()
            {
                issues.push("Custom remote programs require a separate supported policy".into());
            }
            remotes.push(Remote {
                name,
                fetch,
                push,
                origins,
                fetch_refspec_count: suffix(".fetch").count(),
                push_refspec_count: suffix(".push").count(),
                issues,
            });
        }
        let branch = {
            let output = session.output(&inspection::strings(&["symbolic-ref", "-q", "HEAD"]))?;
            match output.status.code() {
                Some(0) => Some(inspection::output_text(output.stdout)?),
                Some(1) => None,
                _ => return Err(invalid("Cannot inspect current branch")),
            }
        };
        let mut issues = Vec::new();
        let mut upstream_remote = None;
        let mut upstream_ref = None;
        if let Some(name) = branch
            .as_deref()
            .and_then(|b| b.strip_prefix("refs/heads/"))
        {
            let upstream = |suffix: &str| {
                config
                    .iter()
                    .filter(|v| v.key == format!("branch.{name}.{suffix}"))
                    .collect::<Vec<_>>()
            };
            let remote = upstream("remote");
            let merge = upstream("merge");
            if remote.len() == 1 && merge.len() == 1 {
                let r = std::str::from_utf8(&remote[0].value)
                    .map_err(|_| invalid("Invalid upstream remote"))?;
                let reference = std::str::from_utf8(&merge[0].value)
                    .map_err(|_| invalid("Invalid upstream ref"))?;
                if !remote_name(r)
                    || !remotes.iter().any(|remote| remote.name == r)
                    || !reference.starts_with("refs/heads/")
                {
                    issues.push("Upstream is not one named remote branch".into());
                } else {
                    session.git(&["check-ref-format", reference])?;
                    upstream_remote = Some(r.into());
                    upstream_ref = Some(reference.into());
                }
            } else {
                issues.push(
                    "No single configured upstream; choose remote and destination explicitly"
                        .into(),
                );
            }
        } else {
            issues.push("Detached HEAD; normal branch push is blocked".into());
        }
        let push_remote = branch
            .as_deref()
            .and_then(|b| b.strip_prefix("refs/heads/"))
            .and_then(|name| {
                config
                    .iter()
                    .rev()
                    .find(|v| v.key == format!("branch.{name}.pushremote"))
            })
            .or_else(|| config.iter().rev().find(|v| v.key == "remote.pushdefault"));
        let suggested_push_remote = if let Some(value) = push_remote {
            let name =
                std::str::from_utf8(&value.value).map_err(|_| invalid("Invalid push remote"))?;
            if remotes.iter().any(|remote| remote.name == name) {
                Some(name.into())
            } else {
                issues.push("Configured push remote does not name an inspected remote".into());
                None
            }
        } else {
            upstream_remote.clone()
        };
        let lower = |key: &str| key.to_ascii_lowercase();
        let enabled = |v: &[u8]| {
            ![b"false".as_slice(), b"no", b"off", b"0"]
                .iter()
                .any(|false_value| v.eq_ignore_ascii_case(false_value))
        };
        let value = Inspection {
            remotes,
            branch,
            upstream_remote,
            upstream_ref,
            suggested_push_remote,
            issues,
            credential_helper_configured: config.iter().any(|v| {
                lower(&v.key).starts_with("credential.")
                    && v.key.ends_with(".helper")
                    && !v.value.is_empty()
            }),
            ssh_command_configured: config.iter().any(|v| lower(&v.key) == "core.sshcommand"),
            proxy_configured: config.iter().any(|v| {
                let k = lower(&v.key);
                k.ends_with(".proxy") && !v.value.is_empty()
            }),
            tls_verification_disabled: config.iter().any(|v| {
                let k = lower(&v.key);
                k.starts_with("http.") && k.ends_with(".sslverify") && !enabled(&v.value)
            }),
            config_digest: digest(&before),
            generation: session.generation.clone(),
            observed_at_ms: now_ms(),
        };
        if run(&query)? != before {
            return Err(Error::with_kind(
                ErrorKind::ConcurrentChange,
                "Remote configuration changed during inspection",
            ));
        }
        // Never retain raw URL/configuration bytes in the public result or persistence layer.
        Ok(value)
    })
}

/// Resolve again at execution; a display projection is never a transport URL.
pub(crate) fn resolve_for_observation(
    repo: &Path,
    global: Option<&Path>,
    name: &str,
    options: &inspection::ReadOptions,
) -> Result<ResolvedDestination> {
    let started = Instant::now();
    let before = inspect(repo, global, options)?;
    let remote = before
        .remotes
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| invalid("Unknown remote"))?;
    if !remote.issues.is_empty() || remote.fetch.len() != 1 {
        return Err(invalid(
            "Remote destination is ambiguous or has unsupported policy",
        ));
    }
    let selected_destination = &remote.fetch[0];
    if selected_destination.credentials_hidden
        || !selected_destination.restrictions.is_empty()
        || !matches!(
            selected_destination.transport,
            Transport::Https | Transport::Local | Transport::Ssh
        )
    {
        return Err(invalid("Observation requires one supported destination without embedded secrets and explicit host transport policy"));
    }
    let mut command = clean(repo, global);
    command.args(["remote", "get-url", "--all", "--", name]);
    let mut remaining = options.clone();
    remaining.timeout = options.timeout.saturating_sub(started.elapsed());
    let output = process::bounded_input(command, &remaining, None)?;
    if !output.status.success() {
        return Err(invalid("Cannot resolve observation destination"));
    }
    let url = inspection::output_text(output.stdout)?;
    if url.chars().any(char::is_control) || url.is_empty() {
        return Err(invalid("Invalid observation destination"));
    }
    let parsed = destination(&url);
    if parsed.display != selected_destination.display
        || parsed.credentials_hidden
        || !parsed.restrictions.is_empty()
    {
        return Err(Error::with_kind(
            ErrorKind::ConcurrentChange,
            "Observation destination changed",
        ));
    }
    let transport = selected_destination.transport.clone();
    remaining.timeout = options.timeout.saturating_sub(started.elapsed());
    let after = inspect(repo, global, &remaining)?;
    if before.config_digest != after.config_digest || before.generation != after.generation {
        return Err(Error::with_kind(
            ErrorKind::ConcurrentChange,
            "Remote configuration changed while resolving destination",
        ));
    }
    Ok(ResolvedDestination {
        inspection: before,
        url,
        transport,
    })
}
