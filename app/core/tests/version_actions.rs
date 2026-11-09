#[cfg(feature = "write-test-support")]
use gitfleet_core::ErrorKind;
mod support;
use gitfleet_core::{
    actions::{semver::*, version::*},
    application::*,
    domain::*,
    inspection::Cancellation,
    writes::*,
};
use std::{fs, os::unix::fs::PermissionsExt};
use support::{tree_bytes, Fixture};
fn options(f: &Fixture) -> Options {
    let global = f.root.join("global.config");
    if !global.exists() {
        fs::write(&global, b"").unwrap();
    }
    let home = f.root.join("signer-home");
    fs::create_dir_all(&home).unwrap();
    Options {
        global_config: Some(global),
        signer_home: Some(home),
        ..Options::default()
    }
}
fn request(app: &mut Application, r: Request) -> Data {
    app.request(app.epoch(), r).unwrap().data
}
fn confirm<T: serde::de::DeserializeOwned>(app: &mut Application, r: &ReviewDisplay) -> T {
    let (c, _) = app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(app.confirm(c, true, Cancellation::default()).unwrap()).unwrap()
}
fn setup(sha256: bool) -> (Fixture, Application, String, String) {
    let f = Fixture::new();
    if sha256 {
        fs::remove_dir_all(f.repo.join(".git")).unwrap();
        f.git(&[
            "-c",
            "init.templateDir=",
            "init",
            "-q",
            "-b",
            "main",
            "--object-format=sha256",
        ]);
        for (k, v) in [
            ("user.name", "Version Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
        ] {
            f.git(&["config", k, v]);
        }
    }
    f.commit_file();
    fs::write(f.repo.join("version"), b"1.4.2\r\n").unwrap();
    fs::set_permissions(f.repo.join("version"), fs::Permissions::from_mode(0o750)).unwrap();
    let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    request(
        &mut app,
        Request::Create {
            name: "Version".into(),
            picker,
        },
    );
    let Data::Workspaces(w) = request(&mut app, Request::Workspaces {}) else {
        panic!()
    };
    let ws = w.active.unwrap();
    let Data::Inventory(i) = request(
        &mut app,
        Request::Inventory {
            workspace: ws.clone(),
        },
    ) else {
        panic!()
    };
    let wt = i.entries[0].id.0.clone();
    let Data::Review(r) = request(
        &mut app,
        Request::ReviewTrust {
            workspace: ws.clone(),
            worktrees: vec![wt.clone()],
        },
    ) else {
        panic!()
    };
    let _: serde_json::Value = confirm(&mut app, &r);
    (f, app, ws, wt)
}
fn review(app: &mut Application, ws: &str, wt: &str) -> ReviewDisplay {
    let Data::Review(r) = request(
        app,
        Request::ReviewVersionAction {
            workspace: ws.into(),
            worktree: wt.into(),
            path: NativePath::Utf8("version".into()),
            increment: Increment::Patch,
            prerelease: PrereleasePolicy::Reject,
        },
    ) else {
        panic!()
    };
    r
}
fn records(app: &mut Application, ws: &str) -> VersionPage {
    let Data::VersionActions(p) = request(
        app,
        Request::VersionActions {
            workspace: ws.into(),
            worktree: None,
            offset: 0,
        },
    ) else {
        panic!()
    };
    p
}
#[test]
fn exact_version_action_then_separate_stage_and_commit_preserves_unselected_data() {
    for sha256 in [false, true] {
        let (f, mut app, ws, wt) = setup(sha256);
        fs::write(f.repo.join("staged-control"), b"keep staged\n").unwrap();
        f.git(&["add", "--", "staged-control"]);
        fs::write(f.repo.join("file.txt"), b"keep dirty\n").unwrap();
        let before = tree_bytes(&f.repo);
        let index = fs::read(f.repo.join(".git/index")).unwrap();
        let head = f.git(&["rev-parse", "HEAD"]);
        let r = review(&mut app, &ws, &wt);
        assert_eq!(before, tree_bytes(&f.repo));
        let value: VersionInvocation = confirm(&mut app, &r);
        assert_eq!(value.phase, VersionPhase::Applied);
        assert!(value.applied);
        assert!(value.definition_approved_at_ms.is_some());
        assert_eq!(fs::read(f.repo.join("version")).unwrap(), b"1.4.3\r\n");
        assert_eq!(
            fs::metadata(f.repo.join("version"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o750
        );
        assert_eq!(fs::read(f.repo.join(".git/index")).unwrap(), index);
        assert_eq!(f.git(&["rev-parse", "HEAD"]), head);
        let after = tree_bytes(&f.repo);
        assert_eq!(
            before
                .iter()
                .filter(|(p, _)| p.to_str() != Some("version"))
                .collect::<Vec<_>>(),
            after
                .iter()
                .filter(|(p, _)| p.to_str() != Some("version"))
                .collect::<Vec<_>>()
        );
        let Data::OperationAudit(audit) = request(
            &mut app,
            Request::OperationAudit {
                workspace: ws.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: Some(wt.clone()),
                    operation_kind: Some(OperationAuditKind::VersionAction),
                    result: Some(OperationAuditResult::Complete),
                    since_ms: None,
                    until_ms: None,
                },
            },
        ) else {
            panic!()
        };
        assert_eq!(audit.items.len(), 1);
        assert_eq!(audit.items[0].id, value.plan.id);
        assert!(matches!(
            audit.items[0].record,
            OperationAuditRecord::VersionAction(_)
        ));
        assert!(app.begin_confirmation(&r.id).is_err());
        assert_eq!(records(&mut app, &ws).items[0].plan.id, value.plan.id);
        let Data::Review(stage) = request(
            &mut app,
            Request::ReviewVersionStage {
                workspace: ws.clone(),
                operation: value.plan.id,
            },
        ) else {
            panic!()
        };
        assert_eq!(fs::read(f.repo.join(".git/index")).unwrap(), index);
        let staged: Outcome = confirm(&mut app, &stage);
        assert_eq!(staged.phase, Phase::Complete);
        assert_eq!(f.git(&["show", ":version"]), b"1.4.3\r\n");
        assert_eq!(f.git(&["show", ":staged-control"]), b"keep staged\n");
        let Data::Review(commit) = request(
            &mut app,
            Request::Propose {
                workspace: ws,
                worktree: wt,
                request: WriteRequest::Commit {
                    selected: vec![SelectedPath {
                        path: NativePath::Utf8("version".into()),
                        source: ContentSource::Index,
                    }],
                    message: "Update version without AI".into(),
                    hooks: HookPolicy::Execute,
                    signing: SigningPreference::Unsigned,
                },
            },
        ) else {
            panic!()
        };
        let result: Outcome = confirm(&mut app, &commit);
        assert!(result.published);
        assert_eq!(f.git(&["show", "HEAD:version"]), b"1.4.3\r\n");
        assert!(!f
            .git_output(&["cat-file", "-e", "HEAD:staged-control"])
            .status
            .success());
        assert_eq!(f.git(&["show", ":staged-control"]), b"keep staged\n");
        assert_eq!(fs::read(f.repo.join("file.txt")).unwrap(), b"keep dirty\n");
    }
}
#[test]
fn decline_precancel_file_index_and_trust_drift_never_replace_version() {
    for case in [
        "decline",
        "precancel",
        "file",
        "index",
        "trust",
        "hardlink",
        "symlink",
    ] {
        let (f, mut app, ws, wt) = setup(false);
        let r = review(&mut app, &ws, &wt);
        let (c, _) = app.begin_confirmation(&r.id).unwrap();
        match case {
            "file" => fs::write(f.repo.join("version"), b"9.0.0\r\n").unwrap(),
            "index" => {
                fs::write(f.repo.join("new-index"), b"x").unwrap();
                f.git(&["add", "--", "new-index"]);
            }
            "trust" => {
                request(
                    &mut app,
                    Request::Revoke {
                        workspace: ws.clone(),
                        worktree: wt.clone(),
                    },
                );
            }
            "hardlink" => {
                fs::hard_link(f.repo.join("version"), f.root.join("external-hardlink")).unwrap()
            }
            "symlink" => {
                fs::remove_file(f.repo.join("version")).unwrap();
                std::os::unix::fs::symlink(f.root.join("outside-version"), f.repo.join("version"))
                    .unwrap();
                fs::write(f.root.join("outside-version"), b"1.4.2\r\n").unwrap();
            }
            _ => {}
        }
        let before = tree_bytes(&f.repo);
        let cancel = Cancellation::default();
        if case == "precancel" {
            cancel.cancel();
        }
        let result = app.confirm(c, case != "decline", cancel);
        if case == "decline" || case == "trust" {
            assert!(result.is_err());
        } else {
            let v: VersionInvocation = serde_json::from_value(result.unwrap()).unwrap();
            assert!(matches!(
                v.phase,
                VersionPhase::Blocked | VersionPhase::Cancelled
            ));
            assert!(!v.applied);
        }
        assert_eq!(before, tree_bytes(&f.repo));
    }
}
#[test]
fn restart_restores_results_without_authority_and_stage_refuses_external_edits() {
    let (f, mut app, ws, wt) = setup(false);
    let r = review(&mut app, &ws, &wt);
    let v: VersionInvocation = confirm(&mut app, &r);
    drop(app);
    let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
    assert_eq!(records(&mut app, &ws).items[0].phase, VersionPhase::Applied);
    assert!(app.begin_confirmation(&r.id).is_err());
    assert_eq!(fs::read(f.repo.join("version")).unwrap(), b"1.4.3\r\n");
    fs::write(f.repo.join("version"), b"external\n").unwrap();
    let before = tree_bytes(&f.repo);
    assert!(app
        .request(
            app.epoch(),
            Request::ReviewVersionStage {
                workspace: ws,
                operation: v.plan.id
            }
        )
        .is_err());
    assert_eq!(before, tree_bytes(&f.repo));
}

#[cfg(feature = "write-test-support")]
#[test]
fn interrupted_intent_blocks_other_writes_and_recovery_never_repeats_increment() {
    for after in [false, true] {
        let (f, mut app, ws, wt) = setup(false);
        let r = review(&mut app, &ws, &wt);
        app.set_version_test_fault(after, TestFaultKind::StorageFailure);
        let value: VersionInvocation = confirm(&mut app, &r);
        assert_eq!(value.phase, VersionPhase::Unconfirmed);
        assert!(!value.applied);
        let expected = if after {
            b"1.4.3\r\n".as_slice()
        } else {
            b"1.4.2\r\n"
        };
        assert_eq!(fs::read(f.repo.join("version")).unwrap(), expected);
        let before = tree_bytes(&f.repo);
        let error = app
            .request(
                app.epoch(),
                Request::Propose {
                    workspace: ws.clone(),
                    worktree: wt.clone(),
                    request: WriteRequest::Stage {
                        paths: vec![NativePath::Utf8("version".into())],
                        hooks: HookPolicy::Execute,
                    },
                },
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Busy);
        assert_eq!(
            app.request(
                app.epoch(),
                Request::Remove {
                    workspace: ws.clone()
                }
            )
            .unwrap_err()
            .kind,
            ErrorKind::Busy
        );
        let Data::Workspaces(w) = request(&mut app, Request::Workspaces {}) else {
            panic!()
        };
        assert_eq!(w.version_action_recovery.len(), 1);
        assert_eq!(w.version_action_recovery[0].operation_id, value.plan.id);
        drop(app);
        let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
        assert_eq!(before, tree_bytes(&f.repo));
        assert_eq!(
            records(&mut app, &ws).items[0].phase,
            VersionPhase::Unconfirmed
        );
        let Data::Review(recovery) = request(
            &mut app,
            Request::ReviewVersionRecovery {
                workspace: ws.clone(),
                operation: value.plan.id.clone(),
            },
        ) else {
            panic!()
        };
        let result: VersionInvocation = confirm(&mut app, &recovery);
        assert_eq!(result.phase, VersionPhase::Reconciled);
        assert_eq!(
            result.observed,
            Some(if after {
                ObservedVersionState::Applied
            } else {
                ObservedVersionState::NotApplied
            })
        );
        assert!(!result.applied);
        assert_eq!(before, tree_bytes(&f.repo));
        assert!(app.begin_confirmation(&r.id).is_err());
        assert!(app
            .request(
                app.epoch(),
                Request::ReviewVersionRecovery {
                    workspace: ws,
                    operation: value.plan.id
                }
            )
            .is_err());
        assert_eq!(fs::read(f.repo.join("version")).unwrap(), expected);
    }
}
#[cfg(feature = "write-test-support")]
#[test]
fn recovery_drift_requires_fresh_observation_and_external_content_is_never_overwritten() {
    let (f, mut app, ws, wt) = setup(false);
    let r = review(&mut app, &ws, &wt);
    app.set_version_test_fault(true, TestFaultKind::StorageFailure);
    let value: VersionInvocation = confirm(&mut app, &r);
    let Data::Review(recovery) = request(
        &mut app,
        Request::ReviewVersionRecovery {
            workspace: ws.clone(),
            operation: value.plan.id.clone(),
        },
    ) else {
        panic!()
    };
    let (c, _) = app.begin_confirmation(&recovery.id).unwrap();
    fs::write(f.repo.join("version"), b"9.9.9\r\n").unwrap();
    let before = tree_bytes(&f.repo);
    assert_eq!(
        app.confirm(c, true, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::StaleReview
    );
    assert_eq!(before, tree_bytes(&f.repo));
    let Data::Review(recovery) = request(
        &mut app,
        Request::ReviewVersionRecovery {
            workspace: ws.clone(),
            operation: value.plan.id.clone(),
        },
    ) else {
        panic!()
    };
    let observed: VersionInvocation = confirm(&mut app, &recovery);
    assert_eq!(observed.observed, Some(ObservedVersionState::Conflict));
    assert!(!observed.applied);
    assert_eq!(before, tree_bytes(&f.repo));
    assert!(app
        .request(
            app.epoch(),
            Request::ReviewVersionStage {
                workspace: ws,
                operation: value.plan.id
            }
        )
        .is_err());
}
#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "subprocess-only disposable version Action SIGKILL worker"]
fn version_action_crash_worker() {
    let root = std::path::PathBuf::from(
        std::env::var_os("GITFLEET_VERSION_WORKER_ROOT").expect("owned fixture root"),
    );
    let f = std::mem::ManuallyDrop::new(Fixture {
        repo: root.join("repo"),
        root,
    });
    let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
    let ws = std::env::var("GITFLEET_VERSION_WORKER_WS").unwrap();
    let wt = std::env::var("GITFLEET_VERSION_WORKER_WT").unwrap();
    let r = review(&mut app, &ws, &wt);
    app.set_version_test_fault(
        std::env::var("GITFLEET_VERSION_WORKER_AFTER").unwrap() == "true",
        TestFaultKind::Sigkill,
    );
    let _: VersionInvocation = confirm(&mut app, &r);
    panic!("Expected owned worker SIGKILL");
}
#[cfg(feature = "write-test-support")]
#[test]
fn actual_sigkill_before_and_after_replace_recovers_without_replay() {
    use std::os::unix::process::ExitStatusExt;
    for after in [false, true] {
        let (f, app, ws, wt) = setup(false);
        fs::write(
            f.repo.join(".gitfleet-m2-fixture"),
            b"owned disposable version crash fixture\n",
        )
        .unwrap();
        drop(app);
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "version_action_crash_worker", "--ignored"])
            .env("GITFLEET_VERSION_WORKER_ROOT", &f.root)
            .env("GITFLEET_VERSION_WORKER_WS", &ws)
            .env("GITFLEET_VERSION_WORKER_WT", &wt)
            .env("GITFLEET_VERSION_WORKER_AFTER", after.to_string())
            .output()
            .unwrap()
            .status;
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let before = tree_bytes(&f.repo);
        let expected = if after {
            b"1.4.3\r\n".as_slice()
        } else {
            b"1.4.2\r\n"
        };
        assert_eq!(fs::read(f.repo.join("version")).unwrap(), expected);
        let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
        let item = records(&mut app, &ws).items.remove(0);
        assert_eq!(item.phase, VersionPhase::Unconfirmed);
        assert!(!item.applied);
        assert_eq!(before, tree_bytes(&f.repo));
        let Data::Review(recovery) = request(
            &mut app,
            Request::ReviewVersionRecovery {
                workspace: ws.clone(),
                operation: item.plan.id,
            },
        ) else {
            panic!()
        };
        let value: VersionInvocation = confirm(&mut app, &recovery);
        assert_eq!(
            value.observed,
            Some(if after {
                ObservedVersionState::Applied
            } else {
                ObservedVersionState::NotApplied
            })
        );
        assert!(!value.applied);
        assert_eq!(before, tree_bytes(&f.repo));
        drop(app);
        let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
        assert_eq!(
            records(&mut app, &ws).items[0].phase,
            VersionPhase::Reconciled
        );
        assert_eq!(before, tree_bytes(&f.repo));
        assert_eq!(fs::read(f.repo.join("version")).unwrap(), expected);
    }
}

#[test]
fn renderer_cannot_supply_action_contents_approval_or_replay_fields() {
    let base = serde_json::json!({"kind":"review_version_action","workspace":"ws","worktree":"wt","path":{"encoding":"utf8","value":"version"},"increment":"patch","prerelease":"reject"});
    assert!(serde_json::from_value::<Request>(base.clone()).is_ok());
    for field in [
        "after",
        "before",
        "approved",
        "nonce",
        "execute",
        "command",
        "force",
        "definition_approved_at_ms",
    ] {
        let mut changed = base.clone();
        changed[field] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<Request>(changed).is_err(),
            "{field}"
        );
    }
}

#[test]
fn unfinished_definition_restores_as_interrupted_without_granting_authority() {
    let (f, mut app, ws, wt) = setup(false);
    let r = review(&mut app, &ws, &wt);
    let before = tree_bytes(&f.repo);
    drop(app);
    let mut app = Application::open(&f.root.join("state"), options(&f)).unwrap();
    let item = records(&mut app, &ws).items.remove(0);
    assert_eq!(item.phase, VersionPhase::Interrupted);
    assert!(item.definition_approved_at_ms.is_none());
    assert!(!item.applied);
    assert!(app.begin_confirmation(&r.id).is_err());
    assert_eq!(before, tree_bytes(&f.repo));
}
#[test]
fn an_untrusted_worktree_cannot_approve_a_version_definition() {
    let (f, mut app, ws, wt) = setup(false);
    request(
        &mut app,
        Request::Revoke {
            workspace: ws.clone(),
            worktree: wt.clone(),
        },
    );
    let before = tree_bytes(&f.repo);
    let error = app
        .request(
            app.epoch(),
            Request::ReviewVersionAction {
                workspace: ws.clone(),
                worktree: wt,
                path: NativePath::Utf8("version".into()),
                increment: Increment::Patch,
                prerelease: PrereleasePolicy::Reject,
            },
        )
        .unwrap_err();
    assert_eq!(error.kind, gitfleet_core::ErrorKind::UntrustedRepository);
    assert!(records(&mut app, &ws).items.is_empty());
    assert_eq!(before, tree_bytes(&f.repo));
}
