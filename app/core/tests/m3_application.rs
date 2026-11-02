mod support;
use gitfleet_core::{application::*, domain::*, inspection::Cancellation, writes::*, ErrorKind};
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
