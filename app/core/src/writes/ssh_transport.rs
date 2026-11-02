//! Host-selected agent identity, independent of repository/user SSH configuration.
use super::*;
use crate::remote::{observation::ToolBinding, SshPolicy};
use std::io::Read;
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};

fn path(value: &Path) -> Result<PathBuf> {
    let value = fs::canonicalize(value)?;
    let text = value
        .to_str()
        .ok_or_else(|| unsupported("SSH policy paths require UTF-8"))?;
    // OpenSSH expands tokens/environment syntax inside several path options.
    if text.chars().any(|c| c.is_control() || "%$\"\\".contains(c)) {
        return Err(unsupported(
            "SSH policy path contains unsupported expansion syntax",
        ));
    }
    Ok(value)
}
fn policy_file(role: &str, value: &Path, limit: usize) -> Result<(ToolBinding, Vec<u8>)> {
    let value = path(value)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&value)?;
    let meta = file.metadata()?;
    if !meta.is_file()
        || meta.nlink() != 1
        || meta.mode() & 0o022 != 0
        || (meta.uid() != unsafe { libc::geteuid() } && meta.uid() != 0)
        || meta.len() > limit as u64
    {
        return Err(unsupported(
            "SSH policy requires bounded, protected regular files",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(fail(
            ErrorKind::OutputLimit,
            "SSH policy file exceeds its bound",
        ));
    }
    let identity = format!("{}:{}:{:?}", meta.dev(), meta.ino(), meta.created().ok());
    if inspection::fs_identity(&value)? != identity {
        return Err(fail(
            ErrorKind::StaleReview,
            "SSH policy file identity changed",
        ));
    }
    if role == "ssh_public_identity" {
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| unsupported("SSH public identity is invalid"))?;
        let mut fields = text.split_whitespace();
        let kind = fields.next();
        let key = fields.next();
        if kind != Some("ssh-ed25519")
            || key.is_none_or(|v| {
                v.len() != 68
                    || !v
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))
            })
            || text.trim_end_matches(['\r', '\n']).contains(['\r', '\n'])
        {
            return Err(unsupported("Select one plain Ed25519 public key; private keys and certificates are unsupported"));
        }
        // Prevent OpenSSH's implicit sibling certificate discovery.
        if fs::symlink_metadata(value.with_file_name(format!(
                "{}-cert.pub",
                value
                    .file_name()
                    .ok_or_else(|| unsupported("Invalid identity path"))?
                    .to_string_lossy()
            )))
        .is_ok()
        {
            return Err(unsupported(
                "Implicit SSH identity certificates are unsupported",
            ));
        }
    }
    let fingerprint = digest(
        &[
            value.as_os_str().as_encoded_bytes(),
            identity.as_bytes(),
            &bytes,
        ]
        .concat(),
    );
    Ok((
        ToolBinding {
            role: role.into(),
            path: NativePath::from_path(&value),
            fingerprint,
        },
        bytes,
    ))
}
pub(super) fn socket_binding(value: &Path) -> Result<ToolBinding> {
    // Keep the selected absolute socket spelling: canonicalizing /var to
    // /private/var can exceed Darwin's sockaddr_un limit even for a live agent.
    let text = value
        .to_str()
        .ok_or_else(|| unsupported("SSH agent path requires UTF-8"))?;
    if !value.is_absolute()
        || text.len() >= 104
        || text.chars().any(|c| c.is_control() || "%$\"\\".contains(c))
    {
        return Err(unsupported(
            "SSH agent socket path is too long or contains unsupported expansion syntax",
        ));
    }
    let _canonical = path(value)?;
    let meta = fs::symlink_metadata(value)?;
    if !meta.file_type().is_socket()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
    {
        return Err(unsupported(
            "SSH agent must be a private socket owned by the current user",
        ));
    }
    Ok(ToolBinding {
        role: "ssh_agent_socket".into(),
        path: NativePath::from_path(value),
        fingerprint: inspection::fs_identity(value)?,
    })
}
pub(super) fn bindings(policy: Option<&SshPolicy>) -> Result<Vec<ToolBinding>> {
    let policy=policy.ok_or_else(||unsupported("SSH requires an explicit host-selected known-hosts file, public identity and agent socket"))?;
    Ok(vec![
        ToolBinding {
            role: "ssh_client".into(),
            path: NativePath::from_path(Path::new("/usr/bin/ssh")),
            fingerprint: adapter::program_fingerprint(Path::new("/usr/bin/ssh"))?,
        },
        policy_file("ssh_known_hosts", &policy.known_hosts, 1024 * 1024)?.0,
        policy_file("ssh_public_identity", &policy.public_identity, 1024)?.0,
        socket_binding(&policy.agent_socket)?,
    ])
}
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub(super) fn command(bindings: &[ToolBinding], home: &Path) -> Result<String> {
    let get = |role: &str| -> Result<String> {
        if let Some(name) = snapshot_name(role) {
            let value = home
                .join(name)
                .into_os_string()
                .into_string()
                .map_err(|_| unsupported("Invalid SSH snapshot path"))?;
            if value
                .chars()
                .any(|c| c.is_control() || "%$\"\\".contains(c))
            {
                return Err(unsupported(
                    "SSH snapshot path contains unsupported expansion syntax",
                ));
            }
            return Ok(value);
        }
        bindings
            .iter()
            .find(|v| v.role == role)
            .ok_or_else(|| unsupported("SSH policy binding is missing"))?
            .path
            .to_path()?
            .into_os_string()
            .into_string()
            .map_err(|_| unsupported("Invalid SSH policy path"))
    };
    let mut args = vec![
        get("ssh_client")?,
        "-F".into(),
        "/dev/null".into(),
        "-i".into(),
        get("ssh_public_identity")?,
    ];
    for option in [
        "BatchMode=yes",
        "StrictHostKeyChecking=yes",
        "GlobalKnownHostsFile=/dev/null",
        "KnownHostsCommand=none",
        "UpdateHostKeys=no",
        "VerifyHostKeyDNS=no",
        "CheckHostIP=no",
        "IdentitiesOnly=yes",
        "IdentityFile=none",
        "CertificateFile=none",
        "PreferredAuthentications=publickey",
        "PubkeyAuthentication=yes",
        "PubkeyAcceptedAlgorithms=ssh-ed25519",
        "PasswordAuthentication=no",
        "KbdInteractiveAuthentication=no",
        "HostbasedAuthentication=no",
        "GSSAPIAuthentication=no",
        "NumberOfPasswordPrompts=0",
        "AddKeysToAgent=no",
        "UseKeychain=no",
        "PKCS11Provider=none",
        "SecurityKeyProvider=none",
        "ProxyCommand=none",
        "ProxyJump=none",
        "PermitLocalCommand=no",
        "LocalCommand=none",
        "ForwardAgent=no",
        "ClearAllForwardings=yes",
        "ControlMaster=no",
        "ControlPath=none",
        "RequestTTY=no",
        "ConnectTimeout=5",
        "ServerAliveInterval=5",
        "ServerAliveCountMax=1",
    ] {
        args.extend(["-o".into(), option.into()]);
    }
    for (key, role) in [
        ("UserKnownHostsFile", "ssh_known_hosts"),
        ("IdentityAgent", "ssh_agent_socket"),
    ] {
        args.extend(["-o".into(), format!("{key}=\"{}\"", get(role)?)]);
    }
    Ok(args
        .iter()
        .map(|v| shell_quote(v))
        .collect::<Vec<_>>()
        .join(" "))
}
pub(super) fn validate_destination(raw: &str) -> Result<()> {
    let value =
        url::Url::parse(raw).map_err(|_| unsupported("SSH requires a canonical ssh:// URL"))?;
    if !raw.starts_with("ssh://")
        || value.scheme() != "ssh"
        || value.host_str().is_none()
        || value.username().is_empty()
        || value.password().is_some()
        || value.query().is_some()
        || value.fragment().is_some()
        || value.path() == "/"
        || !value
            .username()
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || value.username().starts_with('-')
    {
        return Err(unsupported(
            "SSH requires an explicit plain login and repository path without embedded secrets",
        ));
    }
    Ok(())
}

fn snapshot_name(role: &str) -> Option<&'static str> {
    match role {
        "ssh_known_hosts" => Some("ssh-known-hosts"),
        "ssh_public_identity" => Some("ssh-identity.pub"),
        _ => None,
    }
}
pub(super) fn snapshot(bindings: &[ToolBinding], home: &Path) -> Result<()> {
    for binding in bindings {
        let Some(name) = snapshot_name(&binding.role) else {
            continue;
        };
        let path = binding.path.to_path()?;
        let limit = if binding.role == "ssh_public_identity" {
            1024
        } else {
            1024 * 1024
        };
        let (actual, bytes) = policy_file(&binding.role, &path, limit)?;
        if &actual != binding {
            return Err(fail(
                ErrorKind::StaleReview,
                "SSH policy changed before snapshot",
            ));
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(home.join(name))?;
        file.write_all(&bytes)?;
        store::sync(&file)?;
    }
    store::sync_dir(home)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_ssh_configuration_disables_fallback_and_preserves_shell_quoted_paths() {
        let bindings = vec![
            ToolBinding {
                role: "ssh_client".into(),
                path: NativePath::from_path(Path::new("/usr/bin/ssh")),
                fingerprint: String::new(),
            },
            ToolBinding {
                role: "ssh_agent_socket".into(),
                path: NativePath::from_path(Path::new("/tmp/agent's socket")),
                fingerprint: String::new(),
            },
        ];
        let home =
            std::env::temp_dir().join(format!("gitfleet SSH policy's files-{}", Uuid::new_v4()));
        fs::create_dir(&home).unwrap();
        fs::write(
            home.join("ssh-identity.pub"),
            b"public fixture; -G does not connect or authenticate\n",
        )
        .unwrap();
        let policy = command(&bindings, &home).unwrap();
        let effective = || {
            let result = std::process::Command::new("/bin/sh")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .args(["-c", &format!("{policy} -G fixture.invalid")])
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            String::from_utf8(result.stdout).unwrap()
        };
        let actual = effective();
        for expected in [
            "batchmode yes",
            "stricthostkeychecking true",
            "identitiesonly yes",
            "pubkeyacceptedalgorithms ssh-ed25519",
            "passwordauthentication no",
            "kbdinteractiveauthentication no",
            "forwardagent no",
            "addkeystoagent false",
            "updatehostkeys false",
            "identityagent /tmp/agent's socket",
        ] {
            assert!(actual.contains(expected), "Missing {expected}: {actual}");
        }
        assert!(actual.contains(&format!(
            "userknownhostsfile {}/ssh-known-hosts",
            home.display()
        )));
        assert!(actual.contains(&format!("identityfile {}/ssh-identity.pub", home.display())));
        fs::remove_file(home.join("ssh-identity.pub")).unwrap();
        let missing = effective();
        let identities = missing
            .lines()
            .filter(|s| s.starts_with("identityfile "))
            .collect::<Vec<_>>();
        assert_eq!(identities, vec!["identityfile none"]);
        fs::remove_dir(home).unwrap();
        assert!(command(&bindings, Path::new("/tmp/%d-expanded")).is_err());
        assert!(command(&bindings, Path::new("/tmp/$HOME-expanded")).is_err());
    }
}
