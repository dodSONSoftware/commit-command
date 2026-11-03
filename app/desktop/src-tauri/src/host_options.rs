//! Trusted startup policy; credential/agent paths never enter renderer requests.
use gitfleet_core::{
    remote::{HttpsCredentialPolicy, SshPolicy},
    writes::Options,
};
use std::{ffi::OsString, path::PathBuf};
pub fn parse(
    args: impl IntoIterator<Item = OsString>,
    home: Option<OsString>,
) -> Result<(PathBuf, Options), String> {
    let mut args = args.into_iter();
    let mut directory = None;
    let mut known = None;
    let mut identity = None;
    let mut agent = None;
    let mut helper = None;
    let mut ca = None;
    let mut account = None;
    while let Some(flag) = args.next() {
        if flag == "--https-credential-account" {
            if account.is_some() {
                return Err("Duplicate startup option".into());
            }
            let value = args
                .next()
                .ok_or("Credential account is required")?
                .into_string()
                .map_err(|_| "Account requires UTF-8")?;
            gitfleet_core::credentials::valid_account(&value)
                .map_err(|_| "Invalid credential account")?;
            account = Some(value);
            continue;
        }
        let slot = match flag.to_str() {
            Some("--data-dir") => &mut directory,
            Some("--ssh-known-hosts") => &mut known,
            Some("--ssh-public-identity") => &mut identity,
            Some("--ssh-agent-socket") => &mut agent,
            Some("--https-credential-helper") => &mut helper,
            Some("--https-ca-file") => &mut ca,
            _ => return Err("Unsupported startup option".into()),
        };
        if slot.is_some() {
            return Err("Duplicate startup option".into());
        }
        let value = PathBuf::from(args.next().ok_or("Startup option needs a path")?);
        if flag != "--data-dir" && !value.is_absolute() {
            return Err("Startup paths must be absolute".into());
        }
        *slot = Some(value);
    }
    let directory = directory
        .or_else(|| {
            home.map(|p| PathBuf::from(p).join("Library/Application Support/dev.gitfleet.desktop"))
        })
        .ok_or("HOME unavailable")?;
    let mut options = Options::default();
    options.remote.ssh =
        match (known, identity, agent) {
            (None, None, None) => None,
            (Some(known_hosts), Some(public_identity), Some(agent_socket)) => Some(SshPolicy {
                known_hosts,
                public_identity,
                agent_socket,
            }),
            _ => return Err(
                "SSH policy requires all three known-hosts, public-identity and agent-socket paths"
                    .into(),
            ),
        };
    options.remote.ca_file = ca;
    options.remote.https_credentials = match (helper, account) {
        (None, None) => None,
        (Some(helper), Some(account)) => Some(HttpsCredentialPolicy {
            helper,
            account,
            bridge: std::env::current_exe()
                .map_err(|_| "Credential adapter executable unavailable")?,
        }),
        _ => return Err("HTTPS helper requires both executable path and account".into()),
    };
    Ok((directory, options))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }
    #[test]
    fn https_helper_policy_is_explicit_complete_and_contains_no_password_option() {
        let (_, o) = parse(args(&["--data-dir", "/tmp/state"]), None).unwrap();
        assert!(o.remote.https_credentials.is_none());
        let (_, o) = parse(
            args(&[
                "--data-dir",
                "/tmp/state",
                "--https-credential-helper",
                "/tmp/helper's file",
                "--https-credential-account",
                "account",
                "--https-ca-file",
                "/tmp/ca.pem",
            ]),
            None,
        )
        .unwrap();
        let p = o.remote.https_credentials.unwrap();
        assert_eq!(p.account, "account");
        assert_eq!(p.helper, PathBuf::from("/tmp/helper's file"));
        assert_eq!(o.remote.ca_file, Some(PathBuf::from("/tmp/ca.pem")));
        for a in [
            vec![
                "--data-dir",
                "/tmp/state",
                "--https-credential-account",
                "account",
            ],
            vec![
                "--data-dir",
                "/tmp/state",
                "--https-credential-helper",
                "relative",
            ],
            vec![
                "--data-dir",
                "/tmp/state",
                "--https-credential-helper",
                "/tmp/h",
                "--https-credential-account",
                "bad\nvalue",
            ],
            vec!["--data-dir", "/tmp/state", "--https-password", "secret"],
            vec!["--data-dir", "/tmp/state", "--https-ca-file", "relative"],
        ] {
            assert!(parse(args(&a), None).is_err());
        }
    }
    #[test]
    fn startup_requires_complete_explicit_ssh_policy_without_changing_default() {
        let (p, o) = parse(args(&["--data-dir", "/tmp/state"]), None).unwrap();
        assert_eq!(p, PathBuf::from("/tmp/state"));
        assert!(o.remote.ssh.is_none());
        let (_, o) = parse(
            args(&[
                "--ssh-agent-socket",
                "/tmp/agent",
                "--ssh-known-hosts",
                "/tmp/hosts",
                "--data-dir",
                "/tmp/state",
                "--ssh-public-identity",
                "/tmp/public key",
            ]),
            None,
        )
        .unwrap();
        assert_eq!(
            o.remote.ssh.unwrap().public_identity,
            PathBuf::from("/tmp/public key")
        );
        for a in [
            vec!["--data-dir", "/tmp/s", "--ssh-known-hosts", "/tmp/h"],
            vec!["--ssh-known-hosts", "relative"],
            vec!["--data-dir", "/tmp/s", "--data-dir", "/tmp/t"],
            vec!["--ssh-private-key", "/tmp/key"],
            vec!["--data-dir"],
        ] {
            assert!(parse(args(&a), None).is_err());
        }
    }
}
