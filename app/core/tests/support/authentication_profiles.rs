use super::*;
use gitfleet_core::authentication::{Change, ProfilePolicy, Settings};
pub(super) fn settings(t: &mut Test) -> Settings {
    let Data::AuthenticationSettings(s) = t
        .app
        .request(t.app.epoch(), Request::AuthenticationSettings {})
        .unwrap()
        .data
    else {
        panic!()
    };
    s
}
pub(super) fn change(t: &mut Test, change: Change) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(t.app.epoch(), Request::ReviewAuthentication { change })
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
pub(super) fn apply(
    t: &mut Test,
    r: &ReviewDisplay,
    accepted: bool,
) -> gitfleet_core::Result<serde_json::Value> {
    let (cap, _) = t.app.begin_confirmation(&r.id)?;
    t.app.confirm(cap, accepted, Cancellation::default())
}
fn configure(t: &mut Test) -> (String, ProfilePolicy) {
    let secret = super::https_credential_tests::policy(t);
    let old = t.options.remote.https_credentials.take().unwrap();
    t.options.remote.credential_bridge = Some(old.bridge);
    t.options.remote.credential_home = Some(old.home);
    let policy = ProfilePolicy::Https {
        helper: old.helper.to_str().unwrap().into(),
        account: old.account,
        ca_file: t
            .options
            .remote
            .ca_file
            .take()
            .map(|p| p.to_str().unwrap().into()),
    };
    reopen(t);
    (secret, policy)
}
fn assign(t: &mut Test, id: Option<String>) -> ReviewDisplay {
    change(
        t,
        Change::Assign {
            workspace: t.workspace.clone(),
            worktree: t.worktree.clone(),
            remote: "origin".into(),
            profile: id,
        },
    )
}
#[test]
fn profiles_require_native_approval_persist_and_are_not_implicitly_assigned() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let (secret, policy) = configure(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = change(
        &mut t,
        Change::Save {
            id: None,
            name: "Private Gitea".into(),
            policy: policy.clone(),
        },
    );
    assert!(settings(&mut t).profiles.is_empty());
    assert!(!r.content.contains(&secret));
    assert!(!t.f.root.join("helper-calls").exists());
    assert_eq!(
        apply(&mut t, &r, false).unwrap_err().kind,
        ErrorKind::Cancelled
    );
    assert!(settings(&mut t).profiles.is_empty());
    let r = change(
        &mut t,
        Change::Save {
            id: None,
            name: "Private Gitea".into(),
            policy,
        },
    );
    apply(&mut t, &r, true).unwrap();
    let saved = settings(&mut t);
    assert_eq!(saved.profiles.len(), 1);
    assert!(saved.assignments.is_empty());
    reopen(&mut t);
    assert_eq!(settings(&mut t).revision, saved.revision);
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert!(!t.f.root.join("helper-calls").exists());
    assert!(!t.f.root.join("requests").exists());
    super::https_credential_tests::assert_private(&t, &secret);
}
#[test]
fn explicitly_assigned_profile_authenticates_capture_without_host_credential_flags() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let (secret, policy) = configure(&mut t);
    let r = change(
        &mut t,
        Change::Save {
            id: None,
            name: "Gitea".into(),
            policy,
        },
    );
    apply(&mut t, &r, true).unwrap();
    let id = settings(&mut t).profiles[0].id.clone();
    let r = assign(&mut t, Some(id));
    assert!(settings(&mut t).assignments.is_empty());
    apply(&mut t, &r, true).unwrap();
    reopen(&mut t);
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    assert!(r.summary.contains("Authentication profile: Gitea"));
    assert!(!t.f.root.join("helper-calls").exists());
    let observed = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        observed.phase,
        ObservationPhase::Complete,
        "{:?}",
        observed.issue
    );
    assert!(observed.plan.authentication_profile.is_some());
    let r = fetch_review(&mut t, &observed.plan.id);
    let capture = fetch_confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        capture.phase,
        gitfleet_core::remote::fetch::FetchCapturePhase::Complete,
        "{:?}",
        capture.issue
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    super::https_credential_tests::assert_private(&t, &secret);
    let calls = fs::read(t.f.root.join("helper-calls")).unwrap();
    let pending = fetch_review(&mut t, &observed.plan.id);
    let r = assign(&mut t, None);
    apply(&mut t, &r, true).unwrap();
    assert!(settings(&mut t).assignments.is_empty());
    assert!(t.app.begin_confirmation(&pending.id).is_err());
    // A new review may deliberately use the current credential-free policy;
    // an old observation never restores its previous credential assignment.
    let fresh = fetch_review(&mut t, &observed.plan.id);
    assert!(fresh.summary.contains("No credentials"));
    assert!(!fresh.summary.contains("Authentication profile: Gitea"));
    assert_eq!(calls, fs::read(t.f.root.join("helper-calls")).unwrap());
}
#[test]
fn settings_review_refuses_changed_helper_or_destination_and_cancelled_application() {
    for mode in ["helper", "destination", "cancel"] {
        let mut t = setup(false);
        let server = tls_fixture(&mut t, "git");
        configure_https(&mut t, &server, "127.0.0.1");
        let (_, policy) = configure(&mut t);
        let helper = match &policy {
            ProfilePolicy::Https { helper, .. } => helper.clone(),
            _ => unreachable!(),
        };
        let r = change(
            &mut t,
            Change::Save {
                id: None,
                name: "Gitea".into(),
                policy,
            },
        );
        if mode == "helper" {
            use std::io::Write;
            fs::OpenOptions::new()
                .append(true)
                .open(&helper)
                .unwrap()
                .write_all(b"\n# drift\n")
                .unwrap();
            assert!(apply(&mut t, &r, true).is_err());
            assert!(settings(&mut t).profiles.is_empty());
        } else if mode == "cancel" {
            let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
            let token = Cancellation::default();
            token.cancel();
            assert_eq!(
                t.app.confirm(cap, true, token).unwrap_err().kind,
                ErrorKind::Cancelled
            );
            assert!(settings(&mut t).profiles.is_empty());
        } else {
            apply(&mut t, &r, true).unwrap();
            let id = settings(&mut t).profiles[0].id.clone();
            let r = assign(&mut t, Some(id));
            t.f.git(&[
                "remote",
                "set-url",
                "origin",
                "https://example.invalid/replacement.git",
            ]);
            assert!(apply(&mut t, &r, true).is_err());
            assert!(settings(&mut t).assignments.is_empty());
        }
        assert!(!t.f.root.join("helper-calls").exists());
        assert!(!t.f.root.join("requests").exists());
    }
}
#[test]
fn assigned_profile_refuses_destination_drift_and_requires_unassignment_before_delete() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let (_, policy) = configure(&mut t);
    let r = change(
        &mut t,
        Change::Save {
            id: None,
            name: "Gitea".into(),
            policy,
        },
    );
    apply(&mut t, &r, true).unwrap();
    let id = settings(&mut t).profiles[0].id.clone();
    let r = assign(&mut t, Some(id.clone()));
    apply(&mut t, &r, true).unwrap();
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewAuthentication {
                change: Change::Delete { id: id.clone() }
            }
        )
        .is_err());
    t.f.git(&[
        "remote",
        "set-url",
        "origin",
        "https://example.invalid/replacement.git",
    ]);
    trust(&mut t);
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                remote: "origin".into(),
                reference: "refs/heads/main".into()
            }
        )
        .is_err());
    assert!(!t.f.root.join("helper-calls").exists());
    let r = assign(&mut t, None);
    apply(&mut t, &r, true).unwrap();
    let r = change(&mut t, Change::Delete { id });
    apply(&mut t, &r, true).unwrap();
    assert!(settings(&mut t).profiles.is_empty());
}

#[test]
fn profile_and_remote_isolation_and_tool_drift_block_lookup() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let (_, policy) = configure(&mut t);
    let helper = match &policy {
        ProfilePolicy::Https { helper, .. } => helper.clone(),
        _ => unreachable!(),
    };
    let r = change(
        &mut t,
        Change::Save {
            id: None,
            name: "Gitea".into(),
            policy: policy.clone(),
        },
    );
    apply(&mut t, &r, true).unwrap();
    let id = settings(&mut t).profiles[0].id.clone();
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewAuthentication {
                change: Change::Save {
                    id: None,
                    name: "gitea".into(),
                    policy
                }
            }
        )
        .is_err());
    let r = assign(&mut t, Some(id));
    apply(&mut t, &r, true).unwrap();
    t.f.git(&[
        "remote",
        "add",
        "other",
        &format!("https://127.0.0.1:{}/repo.git", server.port),
    ]);
    trust(&mut t);
    let Data::Review(other) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                remote: "other".into(),
                reference: "refs/heads/main".into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(other.summary.contains("No credentials"));
    t.app.cancel_review(&other.id).unwrap();
    let r = review(&mut t, "refs/heads/main");
    use std::io::Write;
    fs::OpenOptions::new()
        .append(true)
        .open(helper)
        .unwrap()
        .write_all(b"\n# changed after operation review\n")
        .unwrap();
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    assert!(!t.f.root.join("helper-calls").exists());
    assert!(!t.f.root.join("requests").exists());
}
