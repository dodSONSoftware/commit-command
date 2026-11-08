use super::*;
use gitfleet_core::{
    remote::{push::PushPlanningSelection, push_publication::*},
    writes::HookPolicy,
};

fn propose(t: &mut Test, selection: PushPlanningSelection) -> gitfleet_core::Result<ReviewDisplay> {
    let Data::Review(review) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushPublication {
                workspace: t.workspace.clone(),
                selection,
                hooks: HookPolicy::Execute,
            },
        )?
        .data
    else {
        panic!()
    };
    Ok(review)
}
fn finish(
    t: &mut Test,
    review: &ReviewDisplay,
    accepted: bool,
    token: Cancellation,
) -> gitfleet_core::Result<PushPublication> {
    let (cap, _) = t.app.begin_confirmation(&review.id)?;
    serde_json::from_value(t.app.confirm(cap, accepted, token)?).map_err(Into::into)
}
fn audit(t: &mut Test) -> OperationAuditPage {
    let Data::OperationAudit(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: None,
                    operation_kind: Some(OperationAuditKind::PushPublication),
                    result: None,
                    since_ms: None,
                    until_ms: None,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    page
}
#[test]
fn approved_exact_fast_forward_publishes_once_without_managed_repository_changes() {
    for sha256 in [false, true] {
        let (mut t, selection) = super::push_check::ready(sha256);
        let source = selection.source_oid.clone();
        t.f.git(&["branch", "unselected"]);
        t.f.git(&["checkout", "-q", "unselected"]);
        fs::write(
            t.f.repo.join("private"),
            b"Unselected object must not be exported",
        )
        .unwrap();
        t.f.git(&["add", "private"]);
        t.f.git(&["commit", "-qm", "Unselected private commit"]);
        let private = String::from_utf8(t.f.git(&["rev-parse", "HEAD:private"])).unwrap();
        t.f.git(&["checkout", "-q", "main"]);
        let before = tree_bytes(&t.f.repo);
        let review = propose(&mut t, selection).unwrap();
        let plan: PushPublicationPlan = serde_json::from_str(&review.content).unwrap();
        assert_ne!(plan.id, plan.check.id);
        assert_eq!(tree_bytes(&t.f.repo), before);
        let value = finish(&mut t, &review, true, Cancellation::default()).unwrap();
        assert_eq!(value.phase, PushPublicationPhase::Complete, "{value:?}");
        assert!(value.published);
        assert_eq!(value.observed_remote_oid, Some(source.clone()));
        assert_eq!(
            String::from_utf8(t.f.git(&[
                "-C",
                t.remote.to_str().unwrap(),
                "rev-parse",
                "refs/heads/main"
            ]))
            .unwrap()
            .trim(),
            source.hex
        );
        let output = std::process::Command::new("git")
            .args([
                "-C",
                t.remote.to_str().unwrap(),
                "cat-file",
                "-e",
                private.trim(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert!(t.app.begin_confirmation(&review.id).is_err());
        assert_eq!(
            audit(&mut t).items[0].result,
            OperationAuditResult::Complete
        );
        reopen(&mut t);
        assert_eq!(
            audit(&mut t).items[0].result,
            OperationAuditResult::Complete
        );
        assert_eq!(tree_bytes(&t.f.repo), before);
    }
}
#[test]
fn declined_and_precancelled_publication_never_changes_destination() {
    let (mut t, selection) = super::push_check::ready(false);
    let before = tree_bytes(&t.remote);
    let review = propose(&mut t, selection.clone()).unwrap();
    assert_eq!(
        finish(&mut t, &review, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(tree_bytes(&t.remote), before);
    let review = propose(&mut t, selection).unwrap();
    let token = Cancellation::default();
    token.cancel();
    let result = finish(&mut t, &review, true, token);
    match result {
        Ok(v) => {
            assert_eq!(v.phase, PushPublicationPhase::Cancelled);
            assert!(!v.may_have_published);
        }
        Err(e) => assert_eq!(e.kind, ErrorKind::Cancelled),
    }
    assert_eq!(tree_bytes(&t.remote), before);
}
#[test]
fn destination_movement_blocks_before_publication() {
    let (mut t, selection) = super::push_check::ready(false);
    let review = propose(&mut t, selection.clone()).unwrap();
    t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "-c",
        "protocol.file.allow=always",
        "fetch",
        "-q",
        t.f.repo.to_str().unwrap(),
        "refs/heads/main:refs/heads/main",
    ]);
    let before = tree_bytes(&t.remote);
    let value = finish(&mut t, &review, true, Cancellation::default()).unwrap();
    assert_eq!(value.phase, PushPublicationPhase::Blocked);
    assert!(!value.may_have_published);
    assert_eq!(tree_bytes(&t.remote), before);
}

#[test]
fn source_or_hook_drift_invalidates_native_publication_approval() {
    for source in [false, true] {
        let (mut t, selection) = super::push_check::ready(false);
        let review = propose(&mut t, selection).unwrap();
        if source {
            fs::write(t.f.repo.join("drift"), b"Changed source").unwrap();
            t.f.git(&["add", "drift"]);
            t.f.git(&["commit", "-qm", "Source drift"]);
        } else {
            use std::os::unix::fs::PermissionsExt;
            let hook = t.f.repo.join(".git/hooks/pre-push");
            fs::create_dir_all(hook.parent().unwrap()).unwrap();
            fs::write(&hook, b"#!/bin/sh\nexit 0\n").unwrap();
            fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let before = tree_bytes(&t.remote);
        match finish(&mut t, &review, true, Cancellation::default()) {
            Err(error) => assert_eq!(error.kind, ErrorKind::StaleReview),
            Ok(value) => {
                assert_eq!(value.phase, PushPublicationPhase::Blocked);
                assert!(!value.may_have_published);
                assert_eq!(value.issue.unwrap().kind, ErrorKind::StaleReview);
            }
        }
        assert_eq!(tree_bytes(&t.remote), before);
    }
}
#[test]
fn server_rejection_remains_unconfirmed_until_separately_reviewed_observation() {
    use std::os::unix::fs::PermissionsExt;
    let (mut t, mut selection) = super::push_check::ready(false);
    selection.observation = observed(&mut t).plan.id;
    let hook = t.remote.join("hooks/pre-receive");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(
        &hook,
        b"#!/bin/sh\nprintf 'private-server-diagnostic' >&2\nexit 1\n",
    )
    .unwrap();
    fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
    let old = t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main",
    ]);
    let review = propose(&mut t, selection).unwrap();
    let result = finish(&mut t, &review, true, Cancellation::default()).unwrap();
    assert_eq!(result.phase, PushPublicationPhase::Unconfirmed);
    assert!(result.may_have_published);
    assert!(!result.published);
    assert_eq!(
        t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ]),
        old
    );
    assert!(!serde_json::to_string(&audit(&mut t))
        .unwrap()
        .contains("private-server"));
    reopen(&mut t);
    let Data::Workspaces(list) = t
        .app
        .request(t.app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(list.push_publication_recovery.len(), 1);
    // A still-recent observation from before this attempt cannot resolve its outcome.
    assert_eq!(
        t.app
            .request(
                t.app.epoch(),
                Request::ReviewPushPublicationRecovery {
                    workspace: t.workspace.clone(),
                    operation: result.plan.id.clone(),
                    observation: result.plan.check.selection.observation.clone(),
                }
            )
            .unwrap_err()
            .kind,
        ErrorKind::StaleReview
    );
    let fresh = observed(&mut t);
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushPublicationRecovery {
                workspace: t.workspace.clone(),
                operation: result.plan.id.clone(),
                observation: fresh.plan.id,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let before = tree_bytes(&t.remote);
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let recovered: PushPublication =
        serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap();
    assert_eq!(recovered.phase, PushPublicationPhase::Reconciled);
    assert_eq!(
        recovered.remote_state,
        Some(PushRemoteState::ExpectedOldPresent)
    );
    assert!(!recovered.published);
    assert_eq!(tree_bytes(&t.remote), before);
    let Data::Workspaces(list) = t
        .app
        .request(t.app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(list.push_publication_recovery.is_empty());
}
#[test]
fn restart_projects_publishing_as_unconfirmed_without_replay() {
    let (mut t, selection) = super::push_check::ready(false);
    let review = propose(&mut t, selection).unwrap();
    let plan: PushPublicationPlan = serde_json::from_str(&review.content).unwrap();
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let key = format!("push-publication:{}", plan.id);
    let raw: String = db
        .query_row("select value from state where key=?1", [&key], |r| r.get(0))
        .unwrap();
    let mut record: serde_json::Value = serde_json::from_str(&raw).unwrap();
    record["value"]["phase"] = serde_json::json!("publishing");
    db.execute(
        "update state set value=?1 where key=?2",
        rusqlite::params![record.to_string(), key],
    )
    .unwrap();
    drop(db);
    let before = tree_bytes(&t.remote);
    reopen(&mut t);
    let page = audit(&mut t);
    let OperationAuditRecord::PushPublication(v) = &page.items[0].record else {
        panic!()
    };
    assert_eq!(v.phase, PushPublicationPhase::Unconfirmed);
    assert!(v.may_have_published);
    assert_eq!(page.items[0].result, OperationAuditResult::NeedsAttention);
    assert!(t.app.begin_confirmation(&review.id).is_err());
    assert_eq!(tree_bytes(&t.remote), before);
}
#[test]
fn publication_request_rejects_unreviewed_force_fields() {
    let (_, selection) = super::push_check::ready(false);
    let mut request = serde_json::to_value(Request::ReviewPushPublication {
        workspace: "fixture".into(),
        selection,
        hooks: HookPolicy::Execute,
    })
    .unwrap();
    request["force"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Request>(request).is_err());
}

#[test]
fn changed_local_destination_metadata_is_refused_before_transport() {
    let (mut t, selection) = super::push_check::ready(false);
    let review = propose(&mut t, selection).unwrap();
    let refs = t.remote.join("refs");
    let saved = t.remote.join("saved-refs");
    fs::rename(&refs, &saved).unwrap();
    std::os::unix::fs::symlink(&saved, &refs).unwrap();
    let old = t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main",
    ]);
    let value = finish(&mut t, &review, true, Cancellation::default()).unwrap();
    assert_eq!(value.phase, PushPublicationPhase::Blocked);
    assert!(!value.may_have_published);
    assert_eq!(
        t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ]),
        old
    );
    fs::remove_file(refs).unwrap();
    fs::rename(saved, t.remote.join("refs")).unwrap();
}

#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "subprocess-only disposable SIGKILL worker"]
fn publication_crash_worker() {
    use gitfleet_core::writes::TestFaultKind;
    let marker = PathBuf::from(
        std::env::var_os("GITFLEET_PUBLICATION_CRASH_MARKER").expect("worker marker"),
    );
    let after = std::env::var("GITFLEET_PUBLICATION_CRASH_AFTER").unwrap() == "true";
    let (mut t, selection) = super::push_check::ready(false);
    fs::write(
        t.f.repo.join(".gitfleet-m2-fixture"),
        b"disposable fixture\n",
    )
    .unwrap();
    let review = propose(&mut t, selection).unwrap();
    let plan: PushPublicationPlan = serde_json::from_str(&review.content).unwrap();
    fs::write(
        marker,
        serde_json::to_vec(&(
            t.f.root.clone(),
            t.workspace.clone(),
            t.worktree.clone(),
            plan,
        ))
        .unwrap(),
    )
    .unwrap();
    t.app
        .set_publication_test_fault(after, TestFaultKind::Sigkill);
    let result = finish(&mut t, &review, true, Cancellation::default());
    panic!("Expected process loss: {result:?}");
}
#[cfg(feature = "write-test-support")]
#[test]
fn actual_sigkill_before_and_after_transport_requires_observation_and_never_replays() {
    use std::os::unix::process::ExitStatusExt;
    for after in [false, true] {
        let owner = Fixture::new();
        let marker = owner.root.join("worker.json");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "push_publication::publication_crash_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("GITFLEET_PUBLICATION_CRASH_MARKER", &marker)
            .env("GITFLEET_PUBLICATION_CRASH_AFTER", after.to_string())
            .output()
            .unwrap();
        assert_eq!(
            output.status.signal(),
            Some(libc::SIGKILL),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let (root, workspace, worktree, plan): (PathBuf, String, String, PushPublicationPlan) =
            serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
        let options = Options {
            global_config: Some(root.join("global.config")),
            signer_home: Some(root.join("home")),
            remote: TransportOptions {
                local_fixture_root: Some(root.clone()),
                ..TransportOptions::default()
            },
            ..Options::default()
        };
        let f = Fixture {
            repo: root.join("repo"),
            root: root.clone(),
        };
        let before = tree_bytes(&f.repo);
        let mut t = Test {
            f,
            app: Application::open(&root.join("state"), options.clone()).unwrap(),
            workspace,
            worktree,
            remote: root.join("remote.git"),
            options,
        };
        let remote = tree_bytes(&t.remote);
        let expected = if after {
            &plan.check.preview.source_oid
        } else {
            &plan.check.preview.observed_oid
        };
        assert_eq!(
            String::from_utf8(t.f.git(&[
                "-C",
                t.remote.to_str().unwrap(),
                "rev-parse",
                "refs/heads/main"
            ]))
            .unwrap()
            .trim(),
            expected.hex
        );
        let page = audit(&mut t);
        let OperationAuditRecord::PushPublication(v) = &page.items[0].record else {
            panic!()
        };
        assert_eq!(v.phase, PushPublicationPhase::Unconfirmed);
        assert!(!v.published);
        let fresh = observed(&mut t);
        let Data::Review(review) = t
            .app
            .request(
                t.app.epoch(),
                Request::ReviewPushPublicationRecovery {
                    workspace: t.workspace.clone(),
                    operation: plan.id,
                    observation: fresh.plan.id,
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let (cap, _) = t.app.begin_confirmation(&review.id).unwrap();
        let result: PushPublication =
            serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap())
                .unwrap();
        assert_eq!(
            result.remote_state,
            Some(if after {
                PushRemoteState::ApprovedSourcePresent
            } else {
                PushRemoteState::ExpectedOldPresent
            })
        );
        assert!(
            !result.published,
            "Observation is not retrospective proof of which actor published"
        );
        reopen(&mut t);
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.remote), remote);
    }
}

#[test]
fn cancellation_during_proven_server_check_is_uncertain_and_never_retried() {
    use std::os::unix::fs::PermissionsExt;
    let (mut t, selection) = super::push_check::ready(false);
    let started = t.f.root.join("server-started");
    let completed = t.f.root.join("server-completed");
    let hook = t.remote.join("hooks/pre-receive");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\nprintf started > '{}'\nsleep 30\nprintf completed > '{}'\n",
            started.display(),
            completed.display()
        ),
    )
    .unwrap();
    fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
    let old = t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main",
    ]);
    let review = propose(&mut t, selection).unwrap();
    let token = Cancellation::default();
    let cancel = token.clone();
    let marker = started.clone();
    let waiter = std::thread::spawn(move || {
        for _ in 0..6000 {
            if marker.exists() {
                cancel.cancel();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        cancel.cancel();
    });
    let result = finish(&mut t, &review, true, token).unwrap();
    waiter.join().unwrap();
    assert!(
        started.exists(),
        "Cancellation must reach a proven running server check"
    );
    assert!(!completed.exists());
    assert_eq!(result.phase, PushPublicationPhase::Unconfirmed);
    assert!(result.may_have_published);
    assert!(!result.published);
    assert_eq!(
        t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ]),
        old
    );
    assert!(t.app.begin_confirmation(&review.id).is_err());
}
