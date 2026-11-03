mod support;
use gitfleet_core::{
    inspection::ReadOptions,
    remote::{self, Transport},
    ErrorKind,
};
use std::fs;
use support::{tree_bytes, Fixture};
fn inspect(f: &Fixture) -> remote::Inspection {
    let global = f.root.join("global");
    if !global.exists() {
        fs::write(&global, b"").unwrap();
    }
    remote::inspect(&f.repo, Some(&global), &ReadOptions::default()).unwrap()
}
#[test]
fn effective_fetch_push_rewrites_case_sensitive_names_origins_and_upstream_are_read_only() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["config", "remote.Origin.url", "alias:team/repo"]);
    f.git(&[
        "config",
        "remote.Origin.fetch",
        "+refs/heads/*:refs/remotes/Origin/*",
    ]);
    f.git(&["config", "branch.main.remote", "Origin"]);
    f.git(&["config", "branch.main.merge", "refs/heads/review"]);
    fs::write(f.root.join("global"), b"[url \"https://read.example.invalid/\"]\n insteadOf = alias:\n[url \"ssh://git@write.example.invalid/\"]\n pushInsteadOf = alias:\n").unwrap();
    let before = tree_bytes(&f.repo);
    let result = inspect(&f);
    assert_eq!(result.remotes.len(), 1);
    let r = &result.remotes[0];
    assert_eq!(r.name, "Origin");
    assert_eq!(r.fetch[0].display, "https://read.example.invalid/team/repo");
    assert_eq!(
        r.push[0].display,
        "ssh://git@write.example.invalid/team/repo"
    );
    assert_eq!(r.fetch[0].transport, Transport::Https);
    assert_eq!(r.push[0].transport, Transport::Ssh);
    assert!(r.origins.iter().any(|o| o.scope == "global"));
    assert!(r.origins.iter().any(|o| o.scope == "local"));
    assert_eq!(r.fetch_refspec_count, 1);
    assert_eq!(result.upstream_remote.as_deref(), Some("Origin"));
    assert_eq!(result.upstream_ref.as_deref(), Some("refs/heads/review"));
    assert_eq!(before, tree_bytes(&f.repo));
}
#[test]
fn explicit_pushurl_suppresses_pushinsteadof_and_multiple_destinations_stay_visible() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&[
        "remote",
        "add",
        "origin",
        "https://read.example.invalid/repo",
    ]);
    f.git(&[
        "config",
        "remote.origin.pushurl",
        "https://write.example.invalid/one",
    ]);
    f.git(&[
        "config",
        "--add",
        "remote.origin.pushurl",
        "https://write.example.invalid/two",
    ]);
    f.git(&[
        "config",
        "url.ssh://other.example.invalid/.pushInsteadOf",
        "https://write.example.invalid/",
    ]);
    let result = inspect(&f);
    let r = &result.remotes[0];
    assert_eq!(r.push.len(), 2);
    assert_eq!(r.push[0].display, "https://write.example.invalid/one");
    assert_eq!(r.push[1].display, "https://write.example.invalid/two");
    assert!(r.issues.iter().any(|i| i.contains("unambiguous")));
}
#[test]
fn inspection_never_executes_configured_helpers_and_never_returns_embedded_secrets() {
    let f = Fixture::new();
    f.commit_file();
    let marker = f.root.join("helper-ran");
    let command = format!("!touch {}", marker.display());
    f.git(&[
        "remote",
        "add",
        "origin",
        "https://user:TOP-SECRET@example.invalid/repo?token=QUERY-SECRET#FRAGMENT-SECRET",
    ]);
    f.git(&["config", "credential.helper", &command]);
    f.git(&["config", "core.sshCommand", &command]);
    f.git(&[
        "config",
        "http.proxy",
        "https://proxy:PROXY-SECRET@example.invalid/",
    ]);
    f.git(&["config", "http.sslVerify", "false"]);
    f.git(&["config", "remote.origin.uploadpack", &command]);
    f.git(&["config", "remote.origin.receivepack", &command]);
    let before = tree_bytes(&f.repo);
    let result = inspect(&f);
    let payload = serde_json::to_string(&result).unwrap();
    for secret in [
        "TOP-SECRET",
        "QUERY-SECRET",
        "FRAGMENT-SECRET",
        "PROXY-SECRET",
        "touch",
    ] {
        assert!(
            !payload.contains(secret),
            "Secret/config command leaked: {secret}"
        );
    }
    assert_eq!(
        result.remotes[0].fetch[0].display,
        "https://example.invalid/repo"
    );
    assert!(result.remotes[0].fetch[0].credentials_hidden);
    assert!(
        result.credential_helper_configured
            && result.ssh_command_configured
            && result.proxy_configured
            && result.tls_verification_disabled
    );
    assert!(!marker.exists());
    assert_eq!(before, tree_bytes(&f.repo));
}
#[test]
fn helper_and_hostile_destinations_are_withheld_and_controls_fail_closed() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["config", "remote.origin.url", "ext::echo SECRET-HELPER-ARG"]);
    let result = inspect(&f);
    assert_eq!(
        result.remotes[0].fetch[0].transport,
        Transport::RemoteHelper
    );
    assert!(!serde_json::to_string(&result)
        .unwrap()
        .contains("SECRET-HELPER-ARG"));
    f.git(&[
        "config",
        "remote.origin.url",
        "https://example.invalid/repo\nhttps://injected.invalid/",
    ]);
    assert_eq!(
        remote::inspect(
            &f.repo,
            Some(&f.root.join("global")),
            &ReadOptions::default()
        )
        .unwrap_err()
        .kind,
        ErrorKind::UnsupportedConfiguration
    );
    f.git(&["config", "remote.origin.url", "/tmp/local repository"]);
    let result = inspect(&f);
    assert_eq!(result.remotes[0].fetch[0].transport, Transport::Local);
}
#[test]
fn missing_upstream_detached_head_and_cancellation_are_explicit() {
    let f = Fixture::new();
    f.commit_file();
    let before = tree_bytes(&f.repo);
    let result = inspect(&f);
    assert!(result.remotes.is_empty());
    assert!(result
        .issues
        .iter()
        .any(|i| i.contains("No single configured upstream")));
    assert_eq!(before, tree_bytes(&f.repo));
    f.git(&["checkout", "--detach", "-q"]);
    let result = inspect(&f);
    assert!(result.branch.is_none());
    assert!(result.issues.iter().any(|i| i.contains("Detached")));
    let options = ReadOptions::default();
    options.cancellation.cancel();
    assert_eq!(
        remote::inspect(&f.repo, None, &options).unwrap_err().kind,
        ErrorKind::Cancelled
    );
}

#[test]
fn push_remote_precedence_is_only_a_suggestion_and_unknown_or_local_upstream_is_reported() {
    let f = Fixture::new();
    f.commit_file();
    for name in ["origin", "other", "branch-choice"] {
        f.git(&[
            "remote",
            "add",
            name,
            &format!("https://{name}.example.invalid/repo"),
        ]);
    }
    f.git(&["config", "branch.main.remote", "origin"]);
    f.git(&["config", "branch.main.merge", "refs/heads/main"]);
    f.git(&["config", "remote.pushDefault", "other"]);
    assert_eq!(inspect(&f).suggested_push_remote.as_deref(), Some("other"));
    f.git(&["config", "branch.main.pushRemote", "branch-choice"]);
    assert_eq!(
        inspect(&f).suggested_push_remote.as_deref(),
        Some("branch-choice")
    );
    f.git(&["config", "branch.main.pushRemote", "missing"]);
    let result = inspect(&f);
    assert!(result.suggested_push_remote.is_none());
    assert!(result.issues.iter().any(|i| i.contains("does not name")));
    f.git(&["config", "branch.main.remote", "."]);
    let result = inspect(&f);
    assert!(result.upstream_remote.is_none());
    assert!(result
        .issues
        .iter()
        .any(|i| i.contains("not one named remote")));
}

#[test]
fn scope_limits_malformed_names_and_case_insensitive_tls_overrides_fail_safely() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["remote", "add", "origin", "https://example.invalid/repo"]);
    f.git(&["config", "http.sslVerify", "FALSE"]);
    assert!(inspect(&f).tls_verification_disabled);
    f.git(&[
        "config",
        "remote.bad..name.url",
        "https://example.invalid/repo",
    ]);
    assert!(remote::inspect(
        &f.repo,
        Some(&f.root.join("global")),
        &ReadOptions::default()
    )
    .is_err());
    f.git(&["config", "--remove-section", "remote.bad..name"]);
    for i in 0..64 {
        f.git(&[
            "config",
            &format!("remote.r{i}.url"),
            "https://example.invalid/repo",
        ]);
    }
    assert_eq!(
        remote::inspect(
            &f.repo,
            Some(&f.root.join("global")),
            &ReadOptions::default()
        )
        .unwrap_err()
        .kind,
        ErrorKind::OutputLimit
    );
}

#[test]
fn malformed_scp_style_userinfo_cannot_leak_a_password_in_inspection() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&[
        "config",
        "remote.origin.url",
        "user:SECRET-PASSWORD@example.invalid:repo",
    ]);
    let result = inspect(&f);
    assert!(!serde_json::to_string(&result)
        .unwrap()
        .contains("SECRET-PASSWORD"));
    assert!(result.remotes[0].fetch[0].credentials_hidden);
    assert!(!result.remotes[0].fetch[0].restrictions.is_empty());
}

#[test]
fn explicit_ssh_login_is_visible_but_password_query_fragment_and_encoded_login_stay_withheld() {
    let f = Fixture::new();
    f.commit_file();
    for (url, public) in [
        ("ssh://git@example.invalid/team/repo", true),
        ("ssh://git:SSH_SECRET@example.invalid/team/repo", false),
        (
            "ssh://git@example.invalid/team/repo?token=SSH_SECRET",
            false,
        ),
        ("ssh://git@example.invalid/team/repo#SSH_SECRET", false),
        ("ssh://bad%40SSH_SECRET@example.invalid/team/repo", false),
    ] {
        f.git(&["config", "remote.origin.url", url]);
        let before = tree_bytes(&f.repo);
        let value = inspect(&f);
        let selected = &value.remotes[0].fetch[0];
        assert_eq!(!selected.credentials_hidden, public);
        if public {
            assert_eq!(selected.display, url);
            assert!(selected.restrictions.is_empty());
        } else {
            assert!(!serde_json::to_string(&value)
                .unwrap()
                .contains("SSH_SECRET"));
            assert!(!selected.restrictions.is_empty());
        }
        assert_eq!(before, tree_bytes(&f.repo));
    }
}

#[test]
fn unsupported_scp_query_and_fragment_values_are_withheld() {
    let f = Fixture::new();
    f.commit_file();
    for value in [
        "git@example.invalid:repo?SCP_PRIVATE_VALUE",
        "git@example.invalid:repo#SCP_PRIVATE_VALUE",
    ] {
        f.git(&["config", "remote.origin.url", value]);
        let report = gitfleet_core::remote::inspect(&f.repo, None, &Default::default()).unwrap();
        assert!(!serde_json::to_string(&report)
            .unwrap()
            .contains("SCP_PRIVATE_VALUE"));
        assert!(report.remotes[0].fetch[0].credentials_hidden);
    }
}
