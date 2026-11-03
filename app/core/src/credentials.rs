//! Endpoint/account-scoped, get-only adapter. Secrets never enter plans or IPC.
use crate::{
    domain::NativePath,
    inspection::ReadOptions,
    process,
    remote::{observation::ToolBinding, HttpsCredentialPolicy},
    Error, ErrorKind, Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    process::Command,
    time::Duration,
};
const LIMIT: usize = 16 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HelperScope {
    pub host: String,
    pub path: String,
    pub account: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BridgePlan {
    scope: HelperScope,
    helper: ToolBinding,
    home: ToolBinding,
}
fn invalid() -> Error {
    Error::with_kind(
        ErrorKind::UnsupportedConfiguration,
        "HTTPS credential helper policy or response is invalid",
    )
}
pub fn valid_account(account: &str) -> Result<()> {
    if account.is_empty()
        || account.len() > 256
        || account.chars().any(|c| c.is_control() || c == ':')
    {
        return Err(invalid());
    }
    Ok(())
}
pub(crate) fn scope(url: &str, account: &str) -> Result<HelperScope> {
    valid_account(account)?;
    let parsed = url::Url::parse(url).map_err(|_| invalid())?;
    if parsed.scheme() != "https"
        || parsed.as_str() != url
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(invalid());
    }
    let host = match parsed.host().ok_or_else(invalid)? {
        url::Host::Ipv6(v) => format!("[{v}]"),
        v => v.to_string(),
    };
    let host = match parsed.port() {
        Some(p) => format!("{host}:{p}"),
        None => host,
    };
    let path = parsed.path().strip_prefix('/').ok_or_else(invalid)?;
    if path.is_empty()
        || path.len() > 4096
        || path.split('/').any(str::is_empty)
        || !path
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-._~/".contains(&c))
    {
        return Err(invalid());
    }
    Ok(HelperScope {
        host,
        path: path.into(),
        account: account.into(),
    })
}
fn fingerprint(path: &Path) -> Result<String> {
    crate::writes::credential_fingerprint(path)
}
fn executable(role: &str, path: &Path) -> Result<ToolBinding> {
    let path = std::fs::canonicalize(path)?;
    let m = std::fs::symlink_metadata(&path)?;
    if !m.is_file()
        || m.mode() & 0o111 == 0
        || m.mode() & 0o022 != 0
        || (m.uid() != unsafe { libc::geteuid() } && m.uid() != 0)
    {
        return Err(invalid());
    }
    Ok(ToolBinding {
        role: role.into(),
        path: NativePath::from_path(&path),
        fingerprint: fingerprint(&path)?,
    })
}
fn helper_home(path: &Path) -> Result<ToolBinding> {
    let path = std::fs::canonicalize(path)?;
    let m = std::fs::symlink_metadata(&path)?;
    if !m.is_dir() || m.mode() & 0o022 != 0 || m.uid() != unsafe { libc::geteuid() } {
        return Err(invalid());
    }
    Ok(ToolBinding {
        role: "credential_home".into(),
        path: NativePath::from_path(&path),
        fingerprint: crate::inspection::fs_identity(&path)?,
    })
}
pub(crate) fn bindings(policy: &HttpsCredentialPolicy) -> Result<Vec<ToolBinding>> {
    valid_account(&policy.account)?;
    Ok(vec![
        executable("credential_helper", &policy.helper)?,
        executable("credential_bridge", &policy.bridge)?,
        helper_home(&policy.home)?,
    ])
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub(crate) fn helper_command(scope: &HelperScope, tools: &[ToolBinding]) -> Result<String> {
    let get = |role: &str| tools.iter().find(|v| v.role == role).ok_or_else(invalid);
    let plan = BridgePlan {
        scope: scope.clone(),
        helper: get("credential_helper")?.clone(),
        home: get("credential_home")?.clone(),
    };
    let bridge = get("credential_bridge")?.path.to_path()?;
    let bridge = bridge.to_str().ok_or_else(invalid)?;
    Ok(format!(
        "!{} --gitfleet-credential-bridge {}",
        quote(bridge),
        quote(&serde_json::to_string(&plan)?)
    ))
}
fn fields(bytes: &[u8], response: bool) -> Result<BTreeMap<&str, &str>> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
    let mut values = BTreeMap::new();
    let mut end = false;
    for line in text.split('\n') {
        if line.is_empty() {
            end = true;
            continue;
        }
        if end || line.contains(['\r', '\0']) {
            return Err(invalid());
        }
        let (key, value) = line.split_once('=').ok_or_else(invalid)?;
        if response
            && !matches!(
                key,
                "protocol" | "host" | "path" | "username" | "password" | "capability[]" | "state[]"
            )
        {
            return Err(invalid());
        }
        if matches!(key, "protocol" | "host" | "path" | "username" | "password")
            && values.insert(key, value).is_some()
        {
            return Err(invalid());
        }
    }
    if !end {
        return Err(invalid());
    }
    Ok(values)
}
fn handle(plan: BridgePlan, operation: &str, input: &[u8], output: &mut impl Write) -> Result<()> {
    if operation == "store" || operation == "erase" {
        return Ok(());
    }
    if operation != "get" {
        return Err(invalid());
    }
    if scope(
        &format!("https://{}/{}", plan.scope.host, plan.scope.path),
        &plan.scope.account,
    )? != plan.scope
    {
        return Err(invalid());
    }
    let values = fields(input, false)?;
    for (key, expected) in [
        ("protocol", "https"),
        ("host", plan.scope.host.as_str()),
        ("path", plan.scope.path.as_str()),
        ("username", plan.scope.account.as_str()),
    ] {
        if values.get(key) != Some(&expected) {
            return Err(invalid());
        }
    }
    if values.contains_key("password") {
        return Err(invalid());
    }
    let path = plan.helper.path.to_path()?;
    if executable("credential_helper", &path)? != plan.helper {
        return Err(invalid());
    }
    let home = plan.home.path.to_path()?;
    if helper_home(&home)? != plan.home {
        return Err(invalid());
    }
    let request = format!(
        "protocol=https\nhost={}\npath={}\nusername={}\n\n",
        plan.scope.host, plan.scope.path, plan.scope.account
    )
    .into_bytes();
    let mut command = Command::new(path);
    command
        .arg("get")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("GIT_TERMINAL_PROMPT", "0");
    let options = ReadOptions {
        timeout: Duration::from_secs(30),
        max_output: LIMIT,
        ..ReadOptions::default()
    };
    // Keep helper descendants inside Git's owned network process group.
    let mut result = process::credential_helper(command, &options, request)?;
    let response = (|| {
        if !result.status.success() {
            return Err(invalid());
        }
        let values = fields(&result.stdout, true)?;
        for (key, expected) in [
            ("protocol", "https"),
            ("host", plan.scope.host.as_str()),
            ("path", plan.scope.path.as_str()),
        ] {
            if values.get(key).is_some_and(|v| *v != expected) {
                return Err(invalid());
            }
        }
        // A helper may return only a password for the account supplied in get.
        // Private helper state/capability extensions are discarded, never forwarded.
        if values
            .get("username")
            .is_some_and(|v| *v != plan.scope.account)
        {
            return Err(invalid());
        }
        let password = values.get("password").ok_or_else(invalid)?;
        if password.is_empty() || password.len() > 8192 || password.chars().any(char::is_control) {
            return Err(invalid());
        }
        // Return only the reviewed account and password, never helper extensions.
        writeln!(
            output,
            "username={}\npassword={}\n",
            plan.scope.account, password
        )?;
        Ok(())
    })();
    result.stdout.fill(0);
    result.stderr.fill(0);
    response
}
/// Entry point shared by the native host and the test/CLI adapter executable.
/// No raw diagnostics, credentials or parsed secret frames are printed on failure.
pub fn bridge_main() -> i32 {
    let result = (|| -> Result<()> {
        let mut args = std::env::args_os().skip(1);
        if args.next().as_deref() != Some(std::ffi::OsStr::new("--gitfleet-credential-bridge")) {
            return Err(invalid());
        }
        let raw = args
            .next()
            .ok_or_else(invalid)?
            .into_string()
            .map_err(|_| invalid())?;
        if raw.len() > 64 * 1024 {
            return Err(invalid());
        }
        let plan: BridgePlan = serde_json::from_str(&raw)?;
        let operation = args
            .next()
            .ok_or_else(invalid)?
            .into_string()
            .map_err(|_| invalid())?;
        if args.next().is_some() {
            return Err(invalid());
        }
        let mut input = Vec::new();
        std::io::stdin()
            .take((64 * 1024 + 1) as u64)
            .read_to_end(&mut input)?;
        let result = if input.len() > 64 * 1024 {
            Err(invalid())
        } else {
            handle(plan, &operation, &input, &mut std::io::stdout())
        };
        input.fill(0);
        result
    })();
    if result.is_ok() {
        0
    } else {
        let _ = std::io::stdout().write_all(b"quit=true\n\n");
        1
    }
}

pub(crate) fn helper_binding(path: &Path) -> Result<ToolBinding> {
    executable("credential_helper", path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_and_protocol_reject_ambiguous_secret_or_cross_account_inputs() {
        let s = scope(
            "https://example.invalid/team/repo.git",
            "reviewed@example.invalid",
        )
        .unwrap();
        assert_eq!(s.path, "team/repo.git");
        assert_eq!(s.host, "example.invalid");
        for url in [
            "http://example.invalid/repo.git",
            "https://name:secret@example.invalid/repo.git",
            "https://example.invalid/repo.git?token=secret",
            "https://example.invalid/repo%2Fsecret",
            "https://example.invalid:443/repo.git",
            "https://example.invalid/",
        ] {
            assert!(scope(url, "account").is_err(), "{url}");
        }
        for account in ["", "bad\nvalue", "bad\rvalue", "colon:value"] {
            assert!(valid_account(account).is_err());
        }
        for frame in [
            b"username=a\nusername=b\npassword=p\n\n".as_slice(),
            b"username=a\npassword=p\nquit=true\n\n",
            b"username=a\npassword=p\r\n\n",
            b"username=a\npassword=p\n\ntrailing=value\n",
        ] {
            assert!(fields(frame, true).is_err());
        }
        assert!(fields(b"protocol=https\nhost=example.invalid\npath=team/repo.git\nusername=a\nwwwauth[]=Basic realm=x\nwwwauth[]=Basic realm=y\n\n",false).is_ok());
    }
    #[test]
    fn ignored_mutations_never_execute_or_validate_a_helper() {
        let plan = || BridgePlan {
            scope: HelperScope {
                host: "example.invalid".into(),
                path: "repo.git".into(),
                account: "account".into(),
            },
            home: ToolBinding {
                role: "credential_home".into(),
                path: NativePath::from_path(Path::new("/no/such/home")),
                fingerprint: String::new(),
            },
            helper: ToolBinding {
                role: "credential_helper".into(),
                path: NativePath::from_path(Path::new("/no/such/helper")),
                fingerprint: String::new(),
            },
        };
        for op in ["store", "erase"] {
            let mut output = Vec::new();
            handle(plan(), op, b"password=must-not-persist\n\n", &mut output).unwrap();
            assert!(output.is_empty());
        }
        let mut output = Vec::new();
        assert!(handle(
            plan(),
            "get",
            b"protocol=https\nhost=other.invalid\npath=repo.git\nusername=account\n\n",
            &mut output
        )
        .is_err());
        assert!(output.is_empty());
    }
}
