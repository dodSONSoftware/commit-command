mod support;
use gitfleet_core::{application::*, domain::*, inspection::Cancellation, writes::*, ErrorKind};
use serde_json::json;
use std::{fs, path::Path};
use support::{tree_bytes, Fixture};
fn fixture_options(f: &Fixture) -> Options {
    let global = f.root.join("global.config");
    if !global.exists() {
        fs::write(&global, b"").unwrap();
    }
    let home = f.root.join("signer-home");
    if !home.exists() {
        fs::create_dir(&home).unwrap();
    }
    Options {
        global_config: Some(global),
        signer_home: Some(home),
        ..Options::default()
    }
}
fn setup() -> (Fixture, Application, String, String) {
    let f = Fixture::new();
    f.commit_file();
    let mut app = Application::open(&f.root.join("state"), fixture_options(&f)).unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "M3".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(w) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    let ws = w.active.unwrap();
    let Data::Inventory(i) = app
        .request(
            app.epoch(),
            Request::Inventory {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let wt = i.entries[0].id.0.clone();
    (f, app, ws, wt)
}
fn trust(app: &mut Application, ws: &str, wt: &str) {
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::ReviewTrust {
                workspace: ws.into(),
                worktrees: vec![wt.into()],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    app.confirm(cap, true, Cancellation::default()).unwrap();
}
#[test]
fn scoped_reads_and_contracts_preserve_every_repository_byte() {
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    for request in [
        Request::Inventory {
            workspace: ws.clone(),
        },
        Request::Refresh {
            workspace: ws.clone(),
            worktree: wt.clone(),
        },
        Request::History {
            workspace: ws.clone(),
            worktree: wt.clone(),
            cursor: None,
        },
        Request::TrustStatus {
            workspace: ws.clone(),
        },
    ] {
        app.request(app.epoch(), request).unwrap();
    }
    assert_eq!(before, tree_bytes(&f.repo));
    assert!(serde_json::from_value::<Request>(
        serde_json::json!({"kind":"execute","approved":true})
    )
    .is_err());
    assert!(serde_json::from_value::<Request>(
        serde_json::json!({"kind":"workspaces","shell":"evil"})
    )
    .is_err());
    assert!(app
        .request(
            app.epoch(),
            Request::History {
                workspace: "foreign".into(),
                worktree: wt,
                cursor: None
            }
        )
        .is_err());
    assert!(Application::request_schema()
        .to_string()
        .contains("additionalProperties"));
}
#[test]
fn native_capability_is_required_decline_replay_and_context_changes_fail_closed() {
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::ReviewTrust {
                workspace: ws.clone(),
                worktrees: vec![wt.clone()],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(app.begin_confirmation("forged").is_err());
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    assert!(app.begin_confirmation(&r.id).is_err());
    assert_eq!(
        app.confirm(cap, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert!(app.review(&r.id).is_err());
    assert_eq!(before, tree_bytes(&f.repo));
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"M3 approved\n").unwrap();
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws.clone(),
                worktree: wt.clone(),
                request: WriteRequest::Commit {
                    selected: vec![SelectedPath {
                        path: NativePath::from_path(Path::new("file.txt")),
                        source: ContentSource::WorkingTree,
                    }],
                    message: "M3 exact message".into(),
                    hooks: HookPolicy::Execute,
                    signing: SigningPreference::Configured,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let before = tree_bytes(&f.repo);
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    assert_eq!(before, tree_bytes(&f.repo));
    let out = app.confirm(cap, true, Cancellation::default()).unwrap();
    assert_eq!(out["phase"], "complete");
    assert_eq!(f.git(&["show", "HEAD:file.txt"]), b"M3 approved\n");
    assert!(app.review(&r.id).is_err());
}
#[test]
fn navigation_migration_restart_and_newer_data_are_preserved() {
    let (f, mut app, ws, wt) = setup();
    let mut state = DesktopState::default();
    state.tabs.push(RepositoryTab {
        worktree: wt.clone(),
        view: "history".into(),
        ..RepositoryTab::default()
    });
    state.drafts.insert(wt.clone(), "draft".into());
    state.selected.push(wt);
    app.request(
        app.epoch(),
        Request::SaveState {
            workspace: ws.clone(),
            state: state.clone(),
        },
    )
    .unwrap();
    drop(app);
    let mut app = Application::open(&f.root.join("state"), fixture_options(&f)).unwrap();
    let Data::State(restored) = app
        .request(
            app.epoch(),
            Request::LoadState {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(restored.drafts, state.drafts);
    assert_eq!(restored.tabs[0].view, "history");
    state.version = 999;
    assert!(app
        .request(
            app.epoch(),
            Request::SaveState {
                workspace: ws.clone(),
                state
            }
        )
        .is_err());
    drop(app);
    let ws = gitfleet_core::workspace::WorkspaceService::open(&f.root.join("state")).unwrap();
    assert_eq!(ws.list().len(), 1);
    assert!(fs::read_dir(f.root.join("state")).unwrap().any(|p| p
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with("state.pre-m3-")));
}
#[test]
fn workspace_switch_invalidates_reviews_and_old_epoch_and_stops_old_watchers() {
    let (_f, mut app, ws, wt) = setup();
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::ReviewTrust {
                workspace: ws.clone(),
                worktrees: vec![wt],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    let old = app.epoch();
    app.request(
        old,
        Request::Activate {
            workspace: ws.clone(),
        },
    )
    .unwrap();
    assert!(app.confirm(cap, true, Cancellation::default()).is_err());
    assert!(app
        .request(old, Request::Inventory { workspace: ws })
        .is_err());
    app.shutdown().unwrap();
    assert!(app.review(&r.id).is_err());
}
#[test]
fn notification_reconciliation_detects_external_edits_without_writes() {
    let (f, mut app, ws, wt) = setup();
    fs::write(f.repo.join("file.txt"), b"external\n").unwrap();
    let before = tree_bytes(&f.repo);
    let start = std::time::Instant::now();
    let mut changed = false;
    while start.elapsed() < std::time::Duration::from_secs(12) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if app.reconcile().unwrap() {
            changed = true;
            break;
        }
    }
    assert!(changed);
    let Data::Inventory(i) = app
        .request(app.epoch(), Request::Inventory { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    let e = i.entries.iter().find(|e| e.id.0 == wt).unwrap();
    assert_eq!(e.status.as_ref().unwrap().counts.unstaged, 1);
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn cancelled_and_replaced_reviews_record_nonpublication_and_cancellation_is_per_operation() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"selected\n").unwrap();
    let request = WriteRequest::Commit {
        selected: vec![SelectedPath {
            path: NativePath::from_path(Path::new("file.txt")),
            source: ContentSource::WorkingTree,
        }],
        message: "native-gated".into(),
        hooks: HookPolicy::Execute,
        signing: SigningPreference::Configured,
    };
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws.clone(),
                worktree: wt.clone(),
                request: request.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    app.cancel_review(&r.id).unwrap();
    let Data::Operations(page) = app
        .request(
            app.epoch(),
            Request::Operations {
                workspace: ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items[0].phase, Phase::CancelledBeforePublication);
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws.clone(),
                worktree: wt.clone(),
                request: request.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    let token = Cancellation::default();
    token.cancel();
    let before = tree_bytes(&f.repo);
    let out = app.confirm(cap, true, token).unwrap();
    assert_eq!(out["phase"], "cancelled_before_publication");
    assert_eq!(before, tree_bytes(&f.repo));
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws,
                worktree: wt,
                request,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    assert_eq!(
        app.confirm(cap, true, Cancellation::default()).unwrap()["phase"],
        "complete"
    );
    assert_eq!(f.git(&["show", "HEAD:file.txt"]), b"selected\n");
}
#[test]
fn interrupted_records_block_scope_loss_and_are_reported_for_guarded_recovery() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"selected\n").unwrap();
    app.request(
        app.epoch(),
        Request::Propose {
            workspace: ws.clone(),
            worktree: wt,
            request: WriteRequest::Stage {
                paths: vec![NativePath::from_path(Path::new("file.txt"))],
                hooks: HookPolicy::Execute,
            },
        },
    )
    .unwrap();
    // Simulate a recorded interruption, not a power-loss or SIGKILL test.
    let db = rusqlite::Connection::open(f.root.join("state/state.db")).unwrap();
    let (id, raw): (String, String) = db
        .query_row(
            "SELECT id,payload FROM write_operations ORDER BY rowid DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let mut payload: serde_json::Value = serde_json::from_str(&raw).unwrap();
    payload["phase"] = "publication_pending".into();
    db.execute(
        "UPDATE write_operations SET payload=?2 WHERE id=?1",
        rusqlite::params![id, payload.to_string()],
    )
    .unwrap();
    drop(db);
    let before = tree_bytes(&f.repo);
    let Data::Workspaces(w) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(w.recovery[0].operation_id, id);
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
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::ReviewRecovery {
                workspace: ws.clone(),
                operation: id,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    assert_eq!(
        app.confirm(cap, true, Cancellation::default()).unwrap()["phase"],
        "cancelled_before_publication"
    );
    assert_eq!(before, tree_bytes(&f.repo));
    app.request(app.epoch(), Request::Remove { workspace: ws })
        .unwrap();
}
#[test]
fn malformed_state_paths_duplicate_tabs_and_unknown_picker_cannot_modify_storage() {
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    assert!(app
        .request(
            app.epoch(),
            Request::Create {
                name: "forged".into(),
                picker: f.repo.display().to_string()
            }
        )
        .is_err());
    let mut state = DesktopState::default();
    let tab = RepositoryTab {
        worktree: wt,
        view: "changes".into(),
        ..RepositoryTab::default()
    };
    state.tabs = vec![tab.clone(), tab];
    assert!(app
        .request(
            app.epoch(),
            Request::SaveState {
                workspace: ws.clone(),
                state
            }
        )
        .is_err());
    let state = DesktopState {
        sidebar_width: 0,
        ..DesktopState::default()
    };
    assert!(app
        .request(
            app.epoch(),
            Request::SaveState {
                workspace: ws.clone(),
                state
            }
        )
        .is_err());
    let mut state = DesktopState::default();
    state.tabs.push(RepositoryTab {
        path: Some(NativePath::from_path(Path::new("../escape"))),
        ..RepositoryTab::default()
    });
    assert!(app
        .request(
            app.epoch(),
            Request::SaveState {
                workspace: ws,
                state
            }
        )
        .is_err());
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn review_text_escapes_bidi_and_reset_invalidates_picker_and_native_capability() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"selected\n").unwrap();
    let Data::Review(r) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws,
                worktree: wt,
                request: WriteRequest::Commit {
                    selected: vec![SelectedPath {
                        path: NativePath::from_path(Path::new("file.txt")),
                        source: ContentSource::WorkingTree,
                    }],
                    message: "safe\u{202e}evil".into(),
                    hooks: HookPolicy::Execute,
                    signing: SigningPreference::Configured,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(!r.content.contains('\u{202e}'));
    assert!(r.content.contains("\\u{202e}") || r.content.contains("\\u202e"));
    let (cap, _) = app.begin_confirmation(&r.id).unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    let before = tree_bytes(&f.repo);
    app.reset_presentation().unwrap();
    assert!(app.confirm(cap, true, Cancellation::default()).is_err());
    assert!(app
        .request(
            app.epoch(),
            Request::Create {
                name: "stale picker".into(),
                picker
            }
        )
        .is_err());
    assert_eq!(before, tree_bytes(&f.repo));
}

fn fast_setup() -> (Fixture, Application, String) {
    let f = Fixture::new();
    f.commit_file();
    let mut app = Application::open_with_monitoring(
        &f.root.join("state"),
        fixture_options(&f),
        MonitoringOptions {
            debounce: std::time::Duration::from_millis(20),
            maximum_coalescing: std::time::Duration::from_millis(100),
            status_interval: std::time::Duration::from_millis(200),
            discovery_interval: std::time::Duration::from_secs(1),
        },
    )
    .unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "monitored".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(w) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    (f, app, w.active.unwrap())
}
fn wait_inventory(
    app: &mut Application,
    ws: &str,
    predicate: impl Fn(&InventoryView) -> bool,
) -> InventoryView {
    let started = std::time::Instant::now();
    loop {
        app.reconcile().unwrap();
        let Data::Inventory(i) = app
            .request(
                app.epoch(),
                Request::Inventory {
                    workspace: ws.into(),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        if predicate(&i) {
            return i;
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(8),
            "Reconciliation did not observe the filesystem change"
        );
        std::thread::sleep(std::time::Duration::from_millis(80));
    }
}
#[test]
fn automatic_discovery_retains_identity_across_rename_and_missing_entries_after_removal() {
    let (f, mut app, ws) = fast_setup();
    let child = f.repo.join("nested");
    f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "-q",
        "-b",
        "main",
        child.to_str().unwrap(),
    ]);
    let inventory = wait_inventory(&mut app, &ws, |i| {
        i.entries
            .iter()
            .any(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&child).unwrap())
    });
    let id = inventory
        .entries
        .iter()
        .find(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&child).unwrap())
        .unwrap()
        .id
        .clone();
    let moved = f.repo.join("renamed");
    fs::rename(&child, &moved).unwrap();
    let inventory = wait_inventory(&mut app, &ws, |i| {
        i.entries.iter().any(|e| {
            e.id == id && e.metadata.root.to_path().unwrap() == fs::canonicalize(&moved).unwrap()
        })
    });
    assert_eq!(
        inventory
            .entries
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .metadata
            .root
            .to_path()
            .unwrap(),
        fs::canonicalize(&moved).unwrap()
    );
    fs::remove_dir_all(&moved).unwrap();
    wait_inventory(&mut app, &ws, |i| {
        i.entries.iter().any(|e| {
            e.id == id
                && matches!(
                    e.availability,
                    gitfleet_core::workspace::Availability::Missing
                )
        })
    });
}
#[test]
fn switching_workspace_stops_refreshing_inactive_repository_state() {
    let (f, mut app, old_ws) = fast_setup();
    let other = f.root.join("other");
    f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "-q",
        "-b",
        "main",
        other.to_str().unwrap(),
    ]);
    let picker = app.register_directory(&other).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "other".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(w) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    let ws = w.active.unwrap();
    assert_ne!(ws, old_ws);
    let db = rusqlite::Connection::open(f.root.join("state/state.db")).unwrap();
    let before: String = db
        .query_row(
            "SELECT status FROM membership WHERE workspace_id=?1",
            [&old_ws],
            |r| r.get(0),
        )
        .unwrap();
    fs::write(f.repo.join("file.txt"), b"inactive external edit\n").unwrap();
    let bytes = tree_bytes(&f.repo);
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_millis(1400) {
        app.reconcile().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(80));
    }
    let after: String = db
        .query_row(
            "SELECT status FROM membership WHERE workspace_id=?1",
            [&old_ws],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(bytes, tree_bytes(&f.repo));
    assert!(app
        .request(app.epoch(), Request::Inventory { workspace: old_ws })
        .is_err());
    app.shutdown().unwrap();
}

#[test]
fn trust_currentness_is_distinct_from_supported_hook_policy_eligibility() {
    use std::os::unix::fs::PermissionsExt;
    let (f, mut app, ws, wt) = setup();
    let hook = f.repo.join(".git/hooks/pre-commit");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(&hook, b"#!/bin/sh\n# lefthook\nexit 0\n").unwrap();
    fs::set_permissions(hook, fs::Permissions::from_mode(0o700)).unwrap();
    trust(&mut app, &ws, &wt);
    let Data::Trust(states) = app
        .request(
            app.epoch(),
            Request::TrustStatus {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(states[0].validity, "current");
    assert!(!states[0].eligible);
    assert!(states[0]
        .reason
        .as_ref()
        .unwrap()
        .contains("unvalidated hook managers"));
    app.request(
        app.epoch(),
        Request::HookPolicy {
            workspace: ws.clone(),
            worktree: wt,
            policy: HookPolicy::BypassAll,
        },
    )
    .unwrap();
    let Data::Trust(states) = app
        .request(app.epoch(), Request::TrustStatus { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(states[0].eligible);
    assert_eq!(states[0].policy, Some(HookPolicy::BypassAll));
}

#[test]
fn operation_reviews_show_exact_content_and_only_applicable_commit_fields() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"reviewed content\n").unwrap();
    let request = |request| Request::Propose {
        workspace: ws.clone(),
        worktree: wt.clone(),
        request,
    };
    let paths = vec![NativePath::from_path(Path::new("file.txt"))];
    let Data::Review(stage) = app
        .request(
            app.epoch(),
            request(WriteRequest::Stage {
                paths: paths.clone(),
                hooks: HookPolicy::Execute,
            }),
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(stage.title, "Review stage");
    assert!(stage.content.contains("file.txt — working tree → index"));
    assert!(stage.content.contains("+reviewed content"));
    assert!(!stage.summary.contains("Message subject"));
    assert!(!stage.summary.contains("Signer:"));
    assert!(!stage.content.contains("Commit identity"));
    assert!(stage
        .summary
        .contains("HEAD and working files are preserved"));
    app.cancel_review(&stage.id).unwrap();
    let Data::Review(commit) = app
        .request(
            app.epoch(),
            request(WriteRequest::Commit {
                selected: vec![SelectedPath {
                    path: paths[0].clone(),
                    source: ContentSource::WorkingTree,
                }],
                message: "Exact reviewed message\n\nBody\n".into(),
                hooks: HookPolicy::Execute,
                signing: SigningPreference::Unsigned,
            }),
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(commit.title, "Review commit");
    assert!(commit.summary.contains("Signer: UNSIGNED"));
    assert!(commit
        .content
        .contains("\"Exact reviewed message\\n\\nBody\\n\""));
    assert!(commit.content.contains("Commit identity"));
    assert!(commit.content.contains("file.txt — WorkingTree"));
    assert!(!commit.content.contains("\"patch\": ["));
}

#[test]
fn native_window_placement_survives_restart_and_invalid_values_preserve_saved_state() {
    let f = Fixture::new();
    let dir = f.root.join("state");
    let placement = WindowPlacement {
        x: -1200,
        y: 60,
        width: 1200,
        height: 800,
    };
    let mut app = Application::open(&dir, fixture_options(&f)).unwrap();
    assert_eq!(app.window_placement().unwrap(), None);
    app.save_window_placement(placement).unwrap();
    assert!(app
        .save_window_placement(WindowPlacement {
            width: 0,
            ..placement
        })
        .is_err());
    assert_eq!(app.window_placement().unwrap(), Some(placement));
    drop(app);
    let mut app = Application::open(&dir, fixture_options(&f)).unwrap();
    assert_eq!(app.window_placement().unwrap(), Some(placement));
}

#[test]
fn activity_deduplicates_linked_worktrees_and_audit_filters_real_operation_phases() {
    let (f, mut app, ws, wt) = setup();
    f.git(&[
        "worktree",
        "add",
        "-qb",
        "linked",
        f.repo.join("linked").to_str().unwrap(),
    ]);
    app.request(
        app.epoch(),
        Request::Scan {
            workspace: ws.clone(),
        },
    )
    .unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let Data::Activity(activity) = app
        .request(
            app.epoch(),
            Request::Activity {
                workspace: ws.clone(),
                worktree: None,
                since: now - 100,
                until: now + 100,
                reference: None,
                author: None,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(activity.complete);
    assert_eq!(activity.items.len(), 1);
    trust(&mut app, &ws, &wt);
    fs::write(f.repo.join("file.txt"), b"audit selection\n").unwrap();
    let Data::Review(review) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws.clone(),
                worktree: wt.clone(),
                request: WriteRequest::Stage {
                    paths: vec![NativePath::from_path(Path::new("file.txt"))],
                    hooks: HookPolicy::Execute,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    app.cancel_review(&review.id).unwrap();
    let query = |operation_kind| Request::Audit {
        workspace: ws.clone(),
        query: AuditQuery {
            offset: 0,
            worktree: Some(wt.clone()),
            operation_kind: Some(operation_kind),
            phase: Some(Phase::CancelledBeforePublication),
            since_ms: None,
            until_ms: None,
        },
    };
    let Data::Audit(audit) = app
        .request(app.epoch(), query("stage".into()))
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(audit.items.len(), 1);
    assert!(audit.items[0].events_complete);
    assert!(audit.items[0].events.len() >= 2);
    assert_eq!(
        audit.items[0].events.last().unwrap().phase,
        Phase::CancelledBeforePublication
    );
    assert_eq!(
        audit.items[0].outcome.phase,
        Phase::CancelledBeforePublication
    );
    let Data::Audit(audit) = app
        .request(app.epoch(), query("commit".into()))
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(audit.items.is_empty());
    assert!(app.request(app.epoch(), query("shell".into())).is_err());
}

#[test]
fn untrusted_remote_inspection_is_scoped_and_includes_host_configuration_without_helper_execution()
{
    let (f, mut app, ws, wt) = setup();
    f.git(&["config", "remote.origin.url", "alias:repo"]);
    fs::write(
        f.root.join("global.config"),
        b"[url \"https://example.invalid/\"]\n insteadOf = alias:\n",
    )
    .unwrap();
    let before = tree_bytes(&f.repo);
    let Data::Remotes(result) = app
        .request(
            app.epoch(),
            Request::Remotes {
                workspace: ws.clone(),
                worktree: wt.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        result.remotes[0].fetch[0].display,
        "https://example.invalid/repo"
    );
    assert_eq!(tree_bytes(&f.repo), before);
    assert!(app
        .request(
            app.epoch(),
            Request::Remotes {
                workspace: "other".into(),
                worktree: wt
            }
        )
        .is_err());
    assert!(app
        .request(
            app.epoch(),
            Request::Remotes {
                workspace: ws,
                worktree: "other".into()
            }
        )
        .is_err());
}

#[test]
fn bulk_composer_state_is_durable_workspace_scoped_and_does_not_change_git() {
    use std::collections::BTreeMap;
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    let state = BulkComposerState {
        version: 1,
        message: "Shared subject\n\nExact body 🛰\n".into(),
        configurations: BTreeMap::from([(
            wt.clone(),
            BatchConfiguration {
                mode: BatchMode::IndexOnly,
                excluded: vec![NativePath::from_path(Path::new("file.txt"))],
                hooks: HookPolicy::BypassAll,
                signing: SigningPreference::Unsigned,
            },
        )]),
    };
    app.request(
        app.epoch(),
        Request::SaveBulkComposer {
            workspace: ws.clone(),
            state: state.clone(),
        },
    )
    .unwrap();
    let Data::BulkComposer(loaded) = app
        .request(
            app.epoch(),
            Request::LoadBulkComposer {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&state).unwrap()
    );
    let other = Fixture::new();
    other.commit_file();
    let picker = app.register_directory(&other.repo).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "Other workspace".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(workspaces) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    let active = workspaces.active.unwrap();
    assert_ne!(active, ws);
    let Data::BulkComposer(empty) = app
        .request(app.epoch(), Request::LoadBulkComposer { workspace: active })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(empty.message.is_empty() && empty.configurations.is_empty());
    assert!(app
        .request(
            app.epoch(),
            Request::SaveBulkComposer {
                workspace: ws.clone(),
                state: BulkComposerState::default()
            }
        )
        .is_err());
    app.request(
        app.epoch(),
        Request::Activate {
            workspace: ws.clone(),
        },
    )
    .unwrap();
    drop(app);
    let mut app = Application::open(&f.root.join("state"), fixture_options(&f)).unwrap();
    let Data::BulkComposer(restored) = app
        .request(app.epoch(), Request::LoadBulkComposer { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn invalid_bulk_preferences_do_not_replace_the_last_valid_draft() {
    use std::collections::BTreeMap;
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    let valid = BulkComposerState {
        message: "Keep this draft".into(),
        ..BulkComposerState::default()
    };
    app.request(
        app.epoch(),
        Request::SaveBulkComposer {
            workspace: ws.clone(),
            state: valid.clone(),
        },
    )
    .unwrap();
    let config = |excluded| BatchConfiguration {
        mode: BatchMode::ReviewAll,
        excluded,
        hooks: HookPolicy::Execute,
        signing: SigningPreference::Configured,
    };
    for state in [
        BulkComposerState {
            version: 2,
            ..valid.clone()
        },
        BulkComposerState {
            message: "x".repeat(512 * 1024 + 1),
            ..valid.clone()
        },
        BulkComposerState {
            configurations: BTreeMap::from([(
                wt.clone(),
                config(vec![NativePath::Utf8("../escape".into())]),
            )]),
            ..valid.clone()
        },
        BulkComposerState {
            configurations: BTreeMap::from([(
                wt,
                config(vec![
                    NativePath::Utf8("file.txt".into()),
                    NativePath::UnixBytes(b"file.txt".to_vec()),
                ]),
            )]),
            ..valid.clone()
        },
    ] {
        assert!(app
            .request(
                app.epoch(),
                Request::SaveBulkComposer {
                    workspace: ws.clone(),
                    state
                }
            )
            .is_err());
    }
    let mut forged = serde_json::to_value(Request::SaveBulkComposer {
        workspace: ws.clone(),
        state: valid.clone(),
    })
    .unwrap();
    forged["state"]["approved"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Request>(forged).is_err());
    let Data::BulkComposer(loaded) = app
        .request(app.epoch(), Request::LoadBulkComposer { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(loaded.message, valid.message);
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn final_presentation_snapshot_is_atomic_durable_and_never_grants_authority() {
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    let mut navigation = DesktopState::default();
    navigation
        .drafts
        .insert(wt.clone(), "Latest individual draft\n\nExact body 🛰".into());
    let bulk = BulkComposerState {
        message: "Latest bulk draft\n\nExact body".into(),
        ..BulkComposerState::default()
    };
    app.save_presentation(app.epoch(), &ws, &navigation, &bulk)
        .unwrap();
    drop(app);
    let mut app = Application::open(&f.root.join("state"), fixture_options(&f)).unwrap();
    let state = app
        .request(
            app.epoch(),
            Request::LoadState {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data;
    let Data::State(state) = state else { panic!() };
    assert_eq!(state.drafts, navigation.drafts);
    let Data::BulkComposer(state) = app
        .request(
            app.epoch(),
            Request::LoadBulkComposer {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(state.message, bulk.message);
    assert!(app
        .save_presentation(app.epoch() + 1, &ws, &navigation, &bulk)
        .is_err());
    assert!(app
        .save_presentation(app.epoch(), "foreign", &navigation, &bulk)
        .is_err());
    let mut bad_bulk = bulk.clone();
    bad_bulk.version = 2;
    let mut changed = navigation.clone();
    changed.drafts.insert(wt.clone(), "Must roll back".into());
    assert!(app
        .save_presentation(app.epoch(), &ws, &changed, &bad_bulk)
        .is_err());
    let db = rusqlite::Connection::open(f.root.join("state/state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_bulk_snapshot BEFORE INSERT ON state WHEN NEW.key LIKE 'bulk-composer:%' BEGIN SELECT RAISE(ABORT,'fixture write failure'); END;").unwrap();
    assert!(app
        .save_presentation(app.epoch(), &ws, &changed, &bulk)
        .is_err());
    db.execute_batch("DROP TRIGGER fail_bulk_snapshot;")
        .unwrap();
    let Data::State(saved) = app
        .request(
            app.epoch(),
            Request::LoadState {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(saved.drafts, navigation.drafts);
    assert_eq!(
        db.query_row("SELECT count(*) FROM write_trust", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM write_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(tree_bytes(&f.repo), before);
}

fn unified_audit(
    app: &mut Application,
    workspace: &str,
    query: OperationAuditQuery,
) -> OperationAuditPage {
    let Data::OperationAudit(page) = app
        .request(
            app.epoch(),
            Request::OperationAudit {
                workspace: workspace.into(),
                query,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    page
}
fn all_audit() -> OperationAuditQuery {
    OperationAuditQuery {
        offset: 0,
        worktree: None,
        operation_kind: None,
        result: None,
        since_ms: None,
        until_ms: None,
    }
}
#[test]
fn unified_audit_filters_real_local_batch_and_remote_records_without_mutation_or_authority() {
    let (f, mut app, ws, wt) = setup();
    f.git(&[
        "remote",
        "add",
        "origin",
        "https://example.invalid/repo.git",
    ]);
    fs::write(f.repo.join("file.txt"), b"unified audit fixture\n").unwrap();
    trust(&mut app, &ws, &wt);
    let before = tree_bytes(&f.repo);
    let Data::Review(stage) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws.clone(),
                worktree: wt.clone(),
                request: WriteRequest::Stage {
                    paths: vec![NativePath::from_path(Path::new("file.txt"))],
                    hooks: HookPolicy::Execute,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    app.cancel_review(&stage.id).unwrap();
    let Data::Review(batch) = app
        .request(
            app.epoch(),
            Request::ReviewBatch {
                workspace: ws.clone(),
                selections: vec![BatchSelection {
                    worktree: wt.clone(),
                    message: "audit batch".into(),
                    mode: BatchMode::ReviewAll,
                    excluded: vec![],
                    hooks: HookPolicy::Execute,
                    signing: SigningPreference::Unsigned,
                }],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    app.cancel_review(&batch.id).unwrap();
    let Data::Review(observation) = app
        .request(
            app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: ws.clone(),
                worktree: wt.clone(),
                remote: "origin".into(),
                reference: "refs/heads/main".into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let pending = unified_audit(
        &mut app,
        &ws,
        OperationAuditQuery {
            result: Some(OperationAuditResult::Pending),
            ..all_audit()
        },
    );
    assert_eq!(pending.items.len(), 1);
    assert_eq!(
        pending.items[0].operation_kind,
        OperationAuditKind::RemoteObservation
    );
    app.cancel_review(&observation.id).unwrap();
    let all = unified_audit(&mut app, &ws, all_audit());
    assert_eq!(all.items.len(), 4);
    assert!(all
        .items
        .windows(2)
        .all(|rows| rows[0].created_at_ms >= rows[1].created_at_ms));
    assert!(all
        .items
        .iter()
        .all(|item| item.result == OperationAuditResult::Cancelled));
    for kind in [
        OperationAuditKind::Stage,
        OperationAuditKind::Commit,
        OperationAuditKind::BatchCommit,
        OperationAuditKind::RemoteObservation,
    ] {
        let page = unified_audit(
            &mut app,
            &ws,
            OperationAuditQuery {
                operation_kind: Some(kind),
                worktree: Some(wt.clone()),
                ..all_audit()
            },
        );
        assert_eq!(page.items.len(), 1, "{kind:?}");
        assert_eq!(page.items[0].operation_kind, kind);
        match &page.items[0].record {
            OperationAuditRecord::Local(record) => {
                assert!(!record.events.is_empty());
                assert!(!record.outcome.published);
            }
            OperationAuditRecord::Batch(view) => {
                assert_eq!(view.outcomes.len(), 1);
                assert_eq!(view.record.phase, BatchPhase::Cancelled);
            }
            OperationAuditRecord::PushPublication(_)
            | OperationAuditRecord::PushCheck(_)
            | OperationAuditRecord::FetchCapture(_)
            | OperationAuditRecord::FetchImport(_)
            | OperationAuditRecord::UpstreamSetup(_) => {
                panic!("No fetch was prepared by this fixture")
            }
            OperationAuditRecord::RemoteObservation(record) => assert_eq!(
                record.phase,
                gitfleet_core::remote::observation::ObservationPhase::Cancelled
            ),
        }
    }
    assert!(unified_audit(
        &mut app,
        &ws,
        OperationAuditQuery {
            since_ms: Some(u64::MAX >> 1),
            ..all_audit()
        }
    )
    .items
    .is_empty());
    assert!(app
        .request(
            app.epoch(),
            Request::OperationAudit {
                workspace: ws.clone(),
                query: OperationAuditQuery {
                    worktree: Some("foreign".into()),
                    ..all_audit()
                }
            }
        )
        .is_err());
    assert!(app
        .request(
            app.epoch(),
            Request::OperationAudit {
                workspace: "foreign".into(),
                query: all_audit()
            }
        )
        .is_err());
    assert!(app.begin_confirmation(&observation.id).is_err());
    assert!(!f.root.join("state/network").exists());
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn unified_audit_pages_across_tied_sources_and_projects_orphans_without_rewriting_journals() {
    let (f, mut app, ws, wt) = setup();
    f.git(&[
        "remote",
        "add",
        "origin",
        "https://example.invalid/repo.git",
    ]);
    trust(&mut app, &ws, &wt);
    let Data::Review(review) = app
        .request(
            app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: ws.clone(),
                worktree: wt.clone(),
                remote: "origin".into(),
                reference: "refs/heads/main".into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    app.cancel_review(&review.id).unwrap();
    let db = rusqlite::Connection::open(f.root.join("state/state.db")).unwrap();
    let raw: String = db
        .query_row(
            "SELECT value FROM state WHERE key LIKE 'remote-observation:%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let template: serde_json::Value = serde_json::from_str(&raw).unwrap();
    db.execute(
        "DELETE FROM state WHERE key LIKE 'remote-observation:%'",
        [],
    )
    .unwrap();
    // Synthetic historical records exercise pagination and crash projections only.
    for n in 0..55 {
        let mut stored = template.clone();
        let id = format!("00000000-0000-4000-8000-{n:012}");
        stored["value"]["plan"]["id"] = json!(id);
        stored["value"]["plan"]["created_at_ms"] = json!(100);
        stored["value"]["phase"] = json!(if n % 2 == 0 { "running" } else { "proposed" });
        db.execute(
            "INSERT INTO state(key,value) VALUES(?1,?2)",
            rusqlite::params![format!("remote-observation:{id}"), stored.to_string()],
        )
        .unwrap();
    }
    let batch_id = "10000000-0000-4000-8000-000000000000";
    let batch = BatchRecord {
        version: 1,
        id: batch_id.into(),
        workspace: WorkspaceId(ws.clone()),
        created_at_ms: 100,
        phase: BatchPhase::Executing,
        issue: None,
        items: vec![BatchItem {
            worktree: WorktreeId(wt.clone()),
            path: NativePath::from_path(&f.repo),
            dependencies: vec![],
            operation: None,
            issue: None,
        }],
    };
    db.execute(
        "INSERT INTO state(key,value) VALUES(?1,?2)",
        rusqlite::params![
            format!("batch:{batch_id}"),
            serde_json::to_string(&batch).unwrap()
        ],
    )
    .unwrap();
    let mut foreign = template.clone();
    foreign["value"]["plan"]["workspace"] = json!("other-workspace");
    foreign["value"]["phase"] = json!("running");
    db.execute(
        "INSERT INTO state(key,value) VALUES('remote-observation:foreign',?1)",
        [foreign.to_string()],
    )
    .unwrap();
    let stored_before: Vec<String> = {
        let mut stmt = db
            .prepare("SELECT value FROM state WHERE key LIKE 'remote-observation:%' ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let first = unified_audit(
        &mut app,
        &ws,
        OperationAuditQuery {
            result: Some(OperationAuditResult::Interrupted),
            since_ms: Some(100),
            until_ms: Some(100),
            ..all_audit()
        },
    );
    assert_eq!(first.items.len(), 50);
    assert_eq!(first.next, Some(50));
    let second = unified_audit(
        &mut app,
        &ws,
        OperationAuditQuery {
            offset: 50,
            result: Some(OperationAuditResult::Interrupted),
            ..all_audit()
        },
    );
    assert_eq!(second.items.len(), 6);
    assert_eq!(second.next, None);
    let ids = first
        .items
        .iter()
        .chain(&second.items)
        .map(|i| i.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 56);
    assert!(first
        .items
        .iter()
        .chain(&second.items)
        .all(|i| match &i.record {
            OperationAuditRecord::RemoteObservation(r) =>
                r.phase == gitfleet_core::remote::observation::ObservationPhase::Interrupted,
            OperationAuditRecord::Batch(b) => b.record.phase == BatchPhase::Interrupted,
            _ => false,
        }));
    assert_eq!(first.items[0].id, batch_id);

    assert!(unified_audit(
        &mut app,
        &ws,
        OperationAuditQuery {
            result: Some(OperationAuditResult::Pending),
            ..all_audit()
        }
    )
    .items
    .is_empty());
    let after: Vec<String> = {
        let mut stmt = db
            .prepare("SELECT value FROM state WHERE key LIKE 'remote-observation:%' ORDER BY key")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    assert_eq!(stored_before, after);
    for query in [
        OperationAuditQuery {
            offset: 1_000_001,
            ..all_audit()
        },
        OperationAuditQuery {
            since_ms: Some(101),
            until_ms: Some(100),
            ..all_audit()
        },
        OperationAuditQuery {
            until_ms: Some(u64::MAX),
            ..all_audit()
        },
    ] {
        assert_eq!(
            app.request(
                app.epoch(),
                Request::OperationAudit {
                    workspace: ws.clone(),
                    query
                }
            )
            .unwrap_err()
            .kind,
            ErrorKind::InvalidRequest
        );
    }
    let key = "remote-observation:00000000-0000-4000-8000-000000000000";
    db.execute("UPDATE state SET value=json_set(value,'$.version',2,'$.value.plan.created_at_ms',200) WHERE key=?1",[key]).unwrap();
    assert_eq!(
        app.request(
            app.epoch(),
            Request::OperationAudit {
                workspace: ws.clone(),
                query: all_audit()
            }
        )
        .unwrap_err()
        .kind,
        ErrorKind::NewerSchema
    );
    db.execute(
        "UPDATE state SET value=json_set(value,'$.version',1,'$.padding',?2) WHERE key=?1",
        rusqlite::params![key, "x".repeat(4 * 1024 * 1024)],
    )
    .unwrap();
    assert_eq!(
        app.request(
            app.epoch(),
            Request::OperationAudit {
                workspace: ws.clone(),
                query: all_audit()
            }
        )
        .unwrap_err()
        .kind,
        ErrorKind::OutputLimit
    );
    assert!(serde_json::from_value::<Request>(
        json!({"kind":"operation_audit","workspace":ws,"query":{"offset":0,"result":"execute"}})
    )
    .is_err());
    assert!(serde_json::from_value::<Request>(
        json!({"kind":"operation_audit","workspace":ws,"query":{"offset":0,"approved":true}})
    )
    .is_err());
}

#[test]
fn g01_hostile_review_text_preserves_exact_approved_message_and_index_blob() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    let path = "review\u{202e}name\n.txt";
    let staged = b"<script>fixture_only()</script>\n\x1b[31m staged\n";
    fs::write(f.repo.join(path), staged).unwrap();
    f.git(&["add", "--", path]);
    fs::write(f.repo.join(path), b"unapproved working bytes\n").unwrap();
    fs::write(f.repo.join("file.txt"), b"excluded staged\n").unwrap();
    f.git(&["add", "--", "file.txt"]);
    let index_before = fs::read(f.repo.join(".git/index")).unwrap();
    let parent = f.git(&["rev-parse", "HEAD"]);
    let message = "<img src=x onerror=fixture_only()>\n\nExact \u{202e}RTL\u{2066}isolate\u{2069}\u{1b} bytes\n";
    let Data::Review(review) = app
        .request(
            app.epoch(),
            Request::Propose {
                workspace: ws,
                worktree: wt,
                request: WriteRequest::Commit {
                    selected: vec![SelectedPath {
                        path: NativePath::from_path(Path::new(path)),
                        source: ContentSource::Index,
                    }],
                    message: message.into(),
                    hooks: HookPolicy::Execute,
                    signing: SigningPreference::Configured,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    for text in [&review.summary, &review.content] {
        assert!(!text.contains(['\u{202e}', '\u{2066}', '\u{2069}', '\u{1b}']));
    }
    assert!(review
        .content
        .contains("<img src=x onerror=fixture_only()>"));
    assert!(review.content.contains("\\u{202e}") || review.content.contains("\\u202e"));
    assert_eq!(fs::read(f.repo.join(".git/index")).unwrap(), index_before);
    assert_eq!(f.git(&["rev-parse", "HEAD"]), parent);
    let (capability, _) = app.begin_confirmation(&review.id).unwrap();
    let result = app
        .confirm(capability, true, Cancellation::default())
        .unwrap();
    assert_eq!(result["phase"], "complete");
    // Git independently inspects actual published bytes rather than trusting the result DTO.
    let commit = f.git(&["cat-file", "commit", "HEAD"]);
    let separator = commit
        .windows(2)
        .position(|bytes| bytes == b"\n\n")
        .unwrap();
    assert_eq!(&commit[separator + 2..], message.as_bytes());
    assert_eq!(f.git(&["show", &format!("HEAD:{path}")]), staged);
    assert_eq!(
        fs::read(f.repo.join(path)).unwrap(),
        b"unapproved working bytes\n"
    );
    assert_eq!(f.git(&["show", ":file.txt"]), b"excluded staged\n");
    assert_eq!(f.git(&["rev-parse", "HEAD^"]), parent);
    assert!(app.review(&review.id).is_err());
}

#[test]
fn g01_replaced_or_closed_review_cannot_use_an_already_issued_confirmation() {
    let (f, mut app, ws, wt) = setup();
    let before = tree_bytes(&f.repo);
    let request = || Request::ReviewTrust {
        workspace: ws.clone(),
        worktrees: vec![wt.clone()],
    };
    let Data::Review(first) = app.request(app.epoch(), request()).unwrap().data else {
        panic!()
    };
    let (old_capability, _) = app.begin_confirmation(&first.id).unwrap();
    let Data::Review(replacement) = app.request(app.epoch(), request()).unwrap().data else {
        panic!()
    };
    assert!(app
        .confirm(old_capability, true, Cancellation::default())
        .is_err());
    // The rejected old callback must not consume or approve the replacement.
    assert!(app.review(&replacement.id).is_ok());
    let (closed_capability, _) = app.begin_confirmation(&replacement.id).unwrap();
    app.cancel_review(&replacement.id).unwrap();
    assert!(app
        .confirm(closed_capability, true, Cancellation::default())
        .is_err());
    let Data::Trust(status) = app
        .request(app.epoch(), Request::TrustStatus { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(status.iter().all(|entry| entry.grant == "untrusted"));
    assert_eq!(tree_bytes(&f.repo), before);
}

#[test]
fn g01_g02_shutdown_or_reset_revokes_issued_commit_confirmation_without_running_hooks() {
    use std::os::unix::fs::PermissionsExt;
    for shutdown in [false, true] {
        let (f, mut app, ws, wt) = setup();
        let hook = f.repo.join(".git/hooks/pre-commit");
        fs::create_dir_all(hook.parent().unwrap()).unwrap();
        // A relative marker avoids embedding fixture paths in executable shell text.
        fs::write(
            &hook,
            b"#!/bin/sh\nprintf 'unexpected hook execution' > g01-hook-marker\n",
        )
        .unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
        trust(&mut app, &ws, &wt);
        fs::write(f.repo.join("file.txt"), b"reviewed working content\n").unwrap();
        let Data::Review(review) = app
            .request(
                app.epoch(),
                Request::Propose {
                    workspace: ws.clone(),
                    worktree: wt,
                    request: WriteRequest::Commit {
                        selected: vec![SelectedPath {
                            path: NativePath::from_path(Path::new("file.txt")),
                            source: ContentSource::WorkingTree,
                        }],
                        message: "Must not publish after context loss\n".into(),
                        hooks: HookPolicy::Execute,
                        signing: SigningPreference::Configured,
                    },
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let (capability, _) = app.begin_confirmation(&review.id).unwrap();
        let before = tree_bytes(&f.repo);
        if shutdown {
            app.shutdown().unwrap();
        } else {
            app.reset_presentation().unwrap();
        }
        assert!(app
            .confirm(capability, true, Cancellation::default())
            .is_err());
        assert_eq!(tree_bytes(&f.repo), before);
        assert!(!f.repo.join("g01-hook-marker").exists());
        drop(app);
        let mut reopened = Application::open(&f.root.join("state"), fixture_options(&f)).unwrap();
        let Data::Operations(page) = reopened
            .request(
                reopened.epoch(),
                Request::Operations {
                    workspace: ws,
                    offset: 0,
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].phase, Phase::CancelledBeforePublication);
        assert_eq!(tree_bytes(&f.repo), before);
    }
}

#[test]
fn identity_recovery_keeps_drafts_and_trust_scoped_and_invalidates_pending_review() {
    let (f, mut app, ws, wt) = setup();
    trust(&mut app, &ws, &wt);
    let mut state = DesktopState::default();
    state.drafts.insert(wt.clone(), "Historical draft".into());
    state.active_tab = Some(wt.clone());
    state.tabs.push(RepositoryTab {
        worktree: wt.clone(),
        ..RepositoryTab::default()
    });
    app.request(
        app.epoch(),
        Request::SaveState {
            workspace: ws.clone(),
            state: state.clone(),
        },
    )
    .unwrap();
    let Data::Review(review) = app
        .request(
            app.epoch(),
            Request::ReviewTrust {
                workspace: ws.clone(),
                worktrees: vec![wt.clone()],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&review.id).unwrap();
    fs::rename(f.repo.join(".git"), f.root.join("old-git")).unwrap();
    f.git(&["init", "-q"]);
    let before = tree_bytes(&f.repo);
    let Data::Inventory(inventory) = app
        .request(
            app.epoch(),
            Request::Scan {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let current = inventory
        .entries
        .iter()
        .find(|e| e.superseded_by.is_none())
        .unwrap();
    assert_ne!(current.id.0, wt);
    assert!(app.confirm(cap, true, Cancellation::default()).is_err());
    let Data::Trust(states) = app
        .request(
            app.epoch(),
            Request::TrustStatus {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        states
            .iter()
            .find(|s| s.worktree == current.id.0)
            .unwrap()
            .grant,
        "untrusted"
    );
    assert!(app
        .request(
            app.epoch(),
            Request::Refresh {
                workspace: ws.clone(),
                worktree: wt.clone()
            }
        )
        .is_err());
    let Data::State(restored) = app
        .request(
            app.epoch(),
            Request::LoadState {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(restored.drafts, state.drafts);
    assert_eq!(restored.active_tab, Some(wt));
    assert!(!restored.drafts.contains_key(&current.id.0));
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn recovered_historical_identities_do_not_degrade_periodic_monitoring() {
    let (f, mut app, ws) = fast_setup();
    fs::rename(f.repo.join(".git"), f.root.join("old-git")).unwrap();
    f.git(&["init", "-q"]);
    app.request(
        app.epoch(),
        Request::Scan {
            workspace: ws.clone(),
        },
    )
    .unwrap();
    let Data::Inventory(initial) = app
        .request(
            app.epoch(),
            Request::Inventory {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let initial_degradation = initial.monitor.degraded;
    let before = tree_bytes(&f.repo);
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(1400) {
        app.reconcile().unwrap();
        let Data::Inventory(inventory) = app
            .request(
                app.epoch(),
                Request::Inventory {
                    workspace: ws.clone(),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        assert_eq!(
            inventory
                .entries
                .iter()
                .filter(|e| e.superseded_by.is_some())
                .count(),
            1
        );
        assert_eq!(inventory.monitor.degraded, initial_degradation);
        std::thread::sleep(std::time::Duration::from_millis(80));
    }
    assert_eq!(before, tree_bytes(&f.repo));
}

#[test]
fn inventory_exposes_scan_level_and_repository_inspection_failures_and_cache_skips() {
    let (f, mut app, ws, _) = setup();
    let broken = f.repo.join("broken");
    fs::create_dir(&broken).unwrap();
    fs::write(broken.join(".git"), b"broken").unwrap();
    let cache = f.repo.join("cache");
    fs::create_dir(&cache).unwrap();
    fs::write(
        cache.join("CACHEDIR.TAG"),
        b"Signature: 8a477f597d28d172789f06886806bc55",
    )
    .unwrap();
    fs::write(cache.join(".git"), b"").unwrap();
    f.git(&["config", "filter.block.clean", "must-not-run"]);
    let Data::Inventory(view) = app
        .request(
            app.epoch(),
            Request::Scan {
                workspace: ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let scan = view.scan.unwrap();
    assert!(!scan.complete);
    assert_eq!(scan.issues.len(), 2);
    assert!(scan
        .issues
        .iter()
        .any(|i| i.path.to_path().unwrap() == fs::canonicalize(&broken).unwrap()));
    assert!(scan
        .issues
        .iter()
        .any(|i| i.path.to_path().unwrap() == fs::canonicalize(&f.repo).unwrap()));
    assert_eq!(scan.skipped_caches.len(), 1);
    let Data::Inventory(view) = app
        .request(app.epoch(), Request::Inventory { workspace: ws })
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(view.scan.unwrap().issues.len(), 2);
}
