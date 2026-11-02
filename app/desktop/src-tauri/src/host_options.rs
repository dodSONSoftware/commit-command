//! Trusted startup policy; credential/agent paths never enter renderer requests.
use gitfleet_core::{remote::SshPolicy, writes::Options};
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
    while let Some(flag) = args.next() {
        let slot = match flag.to_str() {
            Some("--data-dir") => &mut directory,
            Some("--ssh-known-hosts") => &mut known,
            Some("--ssh-public-identity") => &mut identity,
            Some("--ssh-agent-socket") => &mut agent,
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
    Ok((directory, options))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
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
