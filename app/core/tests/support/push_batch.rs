use super::*;
use gitfleet_core::{
    remote::{push::*, push_batch::*, push_publication::*},
    writes::HookPolicy,
};
fn preview(t: &mut Test, id: &str, observation: &str) -> PushPreview {
    let Data::PushPreview(p) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: t.workspace.clone(),
                worktree: id.into(),
                observation: observation.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    p
}
fn fixture(sha256: bool) -> (Test, PushPlanningSelection, PushPlanningSelection) {
    let (mut t, b) = super::push_dependencies::pending_child(sha256);
    let parent = t.worktree.clone();
    let bindings = super::push_dependencies::all_bindings(
        &mut t,
        &parent,
        &b.parent_observation,
        &b.child_worktree,
        &b.child_observation,
    );
    let child = PushPlanningSelection {
        worktree: b.child_worktree.clone(),
        observation: b.child_observation.clone(),
        source_oid: preview(&mut t, &b.child_worktree, &b.child_observation).source_oid,
        children: vec![],
    };
    let parent = PushPlanningSelection {
        worktree: parent,
        observation: b.parent_observation,
        source_oid: b.source_oid,
        children: bindings,
    };
    (t, parent, child)
}
fn create(
    t: &mut Test,
    selections: Vec<PushPlanningSelection>,
) -> gitfleet_core::Result<PushBatch> {
    let Data::PushBatch(b) = t
        .app
        .request(
            t.app.epoch(),
            Request::CreatePushBatch {
                workspace: t.workspace.clone(),
                selections: selections
                    .into_iter()
                    .map(|selection| PushBatchSelection {
                        selection,
                        hooks: HookPolicy::Execute,
                    })
                    .collect(),
            },
        )?
        .data
    else {
        panic!()
    };
    Ok(b)
}
fn get(t: &mut Test, id: &str) -> PushBatch {
    let Data::PushBatch(b) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushBatch {
                workspace: t.workspace.clone(),
                id: id.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    b
}
fn review_item(
    t: &mut Test,
    id: &str,
    selection: PushPlanningSelection,
) -> gitfleet_core::Result<ReviewDisplay> {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushBatchItem {
                workspace: t.workspace.clone(),
                batch: id.into(),
                selection,
            },
        )?
        .data
    else {
        panic!()
    };
    Ok(r)
}
fn finish(
    t: &mut Test,
    r: &ReviewDisplay,
    accepted: bool,
) -> gitfleet_core::Result<PushPublication> {
    let (cap, _) = t.app.begin_confirmation(&r.id)?;
    serde_json::from_value(t.app.confirm(cap, accepted, Cancellation::default())?)
        .map_err(Into::into)
}
fn fresh_child(t: &mut Test, parent: &mut PushPlanningSelection, child: &PushPlanningSelection) {
    let old = t.worktree.clone();
    t.worktree = child.worktree.clone();
    let observation = observed(t);
    t.worktree = old;
    for b in &mut parent.children {
        b.child_observation = observation.plan.id.clone();
    }
}
#[test]
fn guided_queue_publishes_child_then_freshly_reviewed_parent_without_managed_changes() {
    for sha256 in [false, true] {
        let (mut t, mut parent, child) = fixture(sha256);
        let before = tree_bytes(&t.f.repo);
        let parent_remote = tree_bytes(&t.remote);
        let batch = create(&mut t, vec![parent.clone(), child.clone()]).unwrap();
        let Data::PushBatches(page) = t
            .app
            .request(
                t.app.epoch(),
                Request::PushBatches {
                    workspace: t.workspace.clone(),
                    offset: 0,
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].plan.id, batch.plan.id);
        assert_eq!(page.items[0].phase, PushBatchPhase::Ready);

        assert_eq!(
            batch
                .items
                .iter()
                .map(|i| i.worktree.0.clone())
                .collect::<Vec<_>>(),
            vec![child.worktree.clone(), parent.worktree.clone()]
        );
        assert_eq!(
            review_item(&mut t, &batch.plan.id, parent.clone())
                .unwrap_err()
                .kind,
            ErrorKind::StaleReview
        );
        assert_eq!(tree_bytes(&t.remote), parent_remote);
        let r = review_item(&mut t, &batch.plan.id, child.clone()).unwrap();
        assert!(r.summary.contains("only this item"));
        let p: PushPublicationPlan = serde_json::from_str(&r.content).unwrap();
        assert_eq!(p.batch.unwrap().id, batch.plan.id);
        let child_result = finish(&mut t, &r, true).unwrap();
        assert!(child_result.published);
        let current = get(&mut t, &batch.plan.id);
        assert_eq!(current.phase, PushBatchPhase::Ready);
        assert_eq!(current.items[0].phase, PushBatchItemPhase::Complete);
        assert_eq!(current.items[1].phase, PushBatchItemPhase::Pending);
        assert!(t.app.begin_confirmation(&r.id).is_err());
        assert_eq!(
            review_item(&mut t, &batch.plan.id, parent.clone())
                .unwrap_err()
                .kind,
            ErrorKind::StaleReview,
            "Old child observation must not become success evidence"
        );
        fresh_child(&mut t, &mut parent, &child);
        let r = review_item(&mut t, &batch.plan.id, parent.clone()).unwrap();
        let result = finish(&mut t, &r, true).unwrap();
        assert!(result.published);
        let current = get(&mut t, &batch.plan.id);
        assert_eq!(current.phase, PushBatchPhase::Complete);
        assert!(current
            .items
            .iter()
            .all(|i| i.phase == PushBatchItemPhase::Complete));
        assert_eq!(
            String::from_utf8(t.f.git(&[
                "-C",
                t.remote.to_str().unwrap(),
                "rev-parse",
                "refs/heads/main"
            ]))
            .unwrap()
            .trim(),
            parent.source_oid.hex
        );
        assert_eq!(
            String::from_utf8(t.f.git(&[
                "-C",
                t.f.root.join("child-remote.git").to_str().unwrap(),
                "rev-parse",
                "refs/heads/main"
            ]))
            .unwrap()
            .trim(),
            child.source_oid.hex
        );
        assert_eq!(tree_bytes(&t.f.repo), before);
        let Data::OperationAudit(page) = t
            .app
            .request(
                t.app.epoch(),
                Request::OperationAudit {
                    workspace: t.workspace.clone(),
                    query: OperationAuditQuery {
                        offset: 0,
                        worktree: Some(parent.worktree.clone()),
                        operation_kind: Some(OperationAuditKind::PushBatch),
                        result: Some(OperationAuditResult::Complete),
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
        assert_eq!(page.items.len(), 1);
        assert!(matches!(
            page.items[0].record,
            OperationAuditRecord::PushBatch(_)
        ));
        reopen(&mut t);
        assert_eq!(get(&mut t, &batch.plan.id).phase, PushBatchPhase::Complete);
        assert_eq!(tree_bytes(&t.f.repo), before);
    }
}
#[test]
fn decline_stops_guided_queue_and_never_implicitly_publishes_parent() {
    let (mut t, parent, child) = fixture(false);
    let before = tree_bytes(&t.f.repo);
    let parent_remote = tree_bytes(&t.remote);
    let child_remote = tree_bytes(&t.f.root.join("child-remote.git"));
    let batch = create(&mut t, vec![parent.clone(), child.clone()]).unwrap();
    let r = review_item(&mut t, &batch.plan.id, child).unwrap();
    assert_eq!(
        finish(&mut t, &r, false).unwrap_err().kind,
        ErrorKind::Cancelled
    );
    let current = get(&mut t, &batch.plan.id);
    assert_eq!(current.phase, PushBatchPhase::Stopped);
    assert_eq!(current.items[0].phase, PushBatchItemPhase::Cancelled);
    assert!(review_item(&mut t, &batch.plan.id, parent).is_err());
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), parent_remote);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), child_remote);
}
#[test]
fn unselected_missing_child_blocks_queue_creation_and_never_enrolls_child() {
    let (mut t, parent, _child) = fixture(false);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    assert_eq!(
        create(&mut t, vec![parent]).unwrap_err().kind,
        ErrorKind::UnsupportedRepositoryState
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
    let c = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(c.query_row("select count(*) from state where key like 'push-batch:%' or key like 'push-publication:%'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
#[test]
fn restart_interrupts_unfinished_queue_without_restoring_item_authority() {
    let (mut t, parent, child) = fixture(false);
    let batch = create(&mut t, vec![parent.clone(), child.clone()]).unwrap();
    let r = review_item(&mut t, &batch.plan.id, child.clone()).unwrap();
    let child_result = finish(&mut t, &r, true).unwrap();
    assert!(child_result.published);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let child_remote = tree_bytes(&t.f.root.join("child-remote.git"));
    reopen(&mut t);
    let current = get(&mut t, &batch.plan.id);
    assert_eq!(current.phase, PushBatchPhase::Interrupted);
    assert_eq!(current.items[0].phase, PushBatchItemPhase::Complete);
    assert_eq!(current.items[1].phase, PushBatchItemPhase::Interrupted);
    assert!(review_item(&mut t, &batch.plan.id, parent).is_err());
    assert!(t.app.begin_confirmation(&r.id).is_err());
    let Data::Workspaces(list) = t
        .app
        .request(t.app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(list.push_batch_recovery.len(), 1);
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), child_remote);
}

#[test]
fn observed_current_child_is_skipped_without_publication_or_a_second_child_approval() {
    let (mut t, mut parent, mut child) = fixture(false);
    let child_remote = t.f.root.join("child-remote.git");
    t.f.git(&[
        "-C",
        child_remote.to_str().unwrap(),
        "-c",
        "protocol.file.allow=always",
        "fetch",
        "-q",
        t.f.repo.join("child").to_str().unwrap(),
        "refs/heads/main:refs/heads/main",
    ]);
    let old = t.worktree.clone();
    t.worktree = child.worktree.clone();
    let fresh = observed(&mut t);
    t.worktree = old;
    child.observation = fresh.plan.id.clone();
    for b in &mut parent.children {
        b.child_observation = fresh.plan.id.clone();
    }
    let child_before = tree_bytes(&child_remote);
    let managed = tree_bytes(&t.f.repo);
    let batch = create(&mut t, vec![parent.clone(), child]).unwrap();
    assert_eq!(
        batch.items[0].phase,
        PushBatchItemPhase::SkippedObservedCurrent
    );
    assert!(batch.items[0].publication.is_none());
    let r = review_item(&mut t, &batch.plan.id, parent).unwrap();
    assert!(finish(&mut t, &r, true).unwrap().published);
    assert_eq!(get(&mut t, &batch.plan.id).phase, PushBatchPhase::Complete);
    assert_eq!(tree_bytes(&child_remote), child_before);
    assert_eq!(tree_bytes(&t.f.repo), managed);
    let c = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(
        c.query_row(
            "select count(*) from state where key like 'push-publication:%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}
#[test]
fn later_server_rejection_preserves_child_success_and_requires_fresh_retry_queue() {
    use std::os::unix::fs::PermissionsExt;
    let (mut t, mut parent, child) = fixture(false);
    let batch = create(&mut t, vec![parent.clone(), child.clone()]).unwrap();
    let r = review_item(&mut t, &batch.plan.id, child.clone()).unwrap();
    let completed = finish(&mut t, &r, true).unwrap();
    assert!(completed.published);
    fresh_child(&mut t, &mut parent, &child);
    let hook = t.remote.join("hooks/pre-receive");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(&hook, b"#!/bin/sh\nexit 1\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    let managed = tree_bytes(&t.f.repo);
    let child_before = tree_bytes(&t.f.root.join("child-remote.git"));
    let parent_tip = t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main",
    ]);
    let r = review_item(&mut t, &batch.plan.id, parent.clone()).unwrap();
    let failed = finish(&mut t, &r, true).unwrap();
    assert_eq!(failed.phase, PushPublicationPhase::Unconfirmed);
    let stopped = get(&mut t, &batch.plan.id);
    assert_eq!(stopped.phase, PushBatchPhase::Stopped);
    assert_eq!(stopped.items[0].phase, PushBatchItemPhase::Complete);
    assert_eq!(stopped.items[1].phase, PushBatchItemPhase::Unconfirmed);
    assert!(review_item(&mut t, &batch.plan.id, parent.clone()).is_err());
    assert_eq!(
        t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ]),
        parent_tip
    );
    fs::remove_file(hook).unwrap();
    let fresh = observed(&mut t);
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushPublicationRecovery {
                workspace: t.workspace.clone(),
                operation: failed.plan.id,
                observation: fresh.plan.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    t.app.confirm(cap, true, Cancellation::default()).unwrap();
    assert_eq!(
        get(&mut t, &batch.plan.id).phase,
        PushBatchPhase::Stopped,
        "Observation recovery never advances the queue"
    );
    parent.observation = fresh.plan.id.clone();
    for b in &mut parent.children {
        b.parent_observation = fresh.plan.id.clone();
    }
    let retry = create(&mut t, vec![parent.clone()]).unwrap();
    let r = review_item(&mut t, &retry.plan.id, parent).unwrap();
    assert!(finish(&mut t, &r, true).unwrap().published);
    assert_eq!(get(&mut t, &retry.plan.id).phase, PushBatchPhase::Complete);
    assert_eq!(get(&mut t, &batch.plan.id).phase, PushBatchPhase::Stopped);
    assert_eq!(tree_bytes(&t.f.repo), managed);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), child_before);
}

#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "subprocess-only disposable guided queue SIGKILL worker"]
fn guided_queue_crash_worker() {
    let marker =
        PathBuf::from(std::env::var_os("GITFLEET_GUIDED_CRASH_MARKER").expect("worker marker"));
    let owner = marker.parent().unwrap().canonicalize().unwrap();
    assert!(owner.starts_with(fs::canonicalize(std::env::temp_dir()).unwrap()));
    assert_eq!(marker.file_name().unwrap(), "worker.json");
    assert_eq!(
        fs::read_to_string(owner.join(".guided-worker-owner")).unwrap(),
        "Disposable guided queue test\n"
    );
    let (mut t, parent, child) = fixture(false);
    let batch = create(&mut t, vec![parent.clone(), child.clone()]).unwrap();
    let r = review_item(&mut t, &batch.plan.id, child.clone()).unwrap();
    assert!(finish(&mut t, &r, true).unwrap().published);
    fs::write(
        marker,
        serde_json::to_vec(&(
            t.f.root.clone(),
            t.workspace.clone(),
            parent,
            child,
            batch.plan.id,
        ))
        .unwrap(),
    )
    .unwrap();
    unsafe {
        libc::kill(libc::getpid(), libc::SIGKILL);
    }
    panic!("Expected owned worker SIGKILL");
}
#[cfg(feature = "write-test-support")]
#[test]
fn actual_process_loss_between_items_preserves_success_and_never_starts_parent() {
    use std::os::unix::process::ExitStatusExt;
    let owner = Fixture::new();
    fs::write(
        owner.root.join(".guided-worker-owner"),
        b"Disposable guided queue test\n",
    )
    .unwrap();
    let marker = owner.root.join("worker.json");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "push_batch::guided_queue_crash_worker",
            "--ignored",
            "--nocapture",
        ])
        .env("GITFLEET_GUIDED_CRASH_MARKER", &marker)
        .output()
        .unwrap();
    assert_eq!(
        output.status.signal(),
        Some(libc::SIGKILL),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let (root, workspace, parent, child, id): (
        PathBuf,
        String,
        PushPlanningSelection,
        PushPlanningSelection,
        String,
    ) = serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
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
    let managed = tree_bytes(&f.repo);
    let remote = tree_bytes(&root.join("remote.git"));
    let child_remote = tree_bytes(&root.join("child-remote.git"));
    let mut t = Test {
        f,
        app: Application::open(&root.join("state"), options.clone()).unwrap(),
        workspace,
        worktree: parent.worktree.clone(),
        remote: root.join("remote.git"),
        options,
    };
    let batch = get(&mut t, &id);
    assert_eq!(batch.phase, PushBatchPhase::Interrupted);
    assert_eq!(batch.items[0].worktree.0, child.worktree);
    assert_eq!(batch.items[0].phase, PushBatchItemPhase::Complete);
    assert_eq!(batch.items[1].phase, PushBatchItemPhase::Interrupted);
    assert!(review_item(&mut t, &id, parent).is_err());
    assert_eq!(tree_bytes(&t.f.repo), managed);
    assert_eq!(tree_bytes(&t.remote), remote);
    assert_eq!(tree_bytes(&root.join("child-remote.git")), child_remote);
    let c = rusqlite::Connection::open(root.join("state/state.db")).unwrap();
    assert_eq!(
        c.query_row(
            "select count(*) from state where key like 'push-publication:%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn revoked_selected_worktree_blocks_queue_before_any_publication_record() {
    let (mut t, _parent, child) = fixture(false);
    t.app
        .request(
            t.app.epoch(),
            Request::Revoke {
                workspace: t.workspace.clone(),
                worktree: child.worktree.clone(),
            },
        )
        .unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.f.root.join("child-remote.git"));
    assert_eq!(
        create(&mut t, vec![child]).unwrap_err().kind,
        ErrorKind::UntrustedRepository
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remote);
    let c = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(c.query_row("select count(*) from state where key like 'push-batch:%' or key like 'push-publication:%'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
#[test]
fn a_saved_queue_cannot_broaden_its_frozen_source() {
    let (mut t, parent, mut child) = fixture(false);
    let batch = create(&mut t, vec![child.clone()]).unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.f.root.join("child-remote.git"));
    child.source_oid = parent.source_oid;
    assert_eq!(
        review_item(&mut t, &batch.plan.id, child).unwrap_err().kind,
        ErrorKind::StaleReview
    );
    assert_eq!(
        get(&mut t, &batch.plan.id).items[0].phase,
        PushBatchItemPhase::Pending
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remote);
    let c = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(
        c.query_row(
            "select count(*) from state where key like 'push-publication:%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}
#[test]
fn guided_request_rejects_approval_force_and_hook_policy_overrides() {
    let request = Request::ReviewPushBatchItem {
        workspace: "w".into(),
        batch: "queue".into(),
        selection: PushPlanningSelection {
            worktree: "t".into(),
            observation: "o".into(),
            source_oid: gitfleet_core::domain::ObjectId {
                format: gitfleet_core::domain::ObjectFormat::Sha1,
                hex: "a".repeat(40),
            },
            children: vec![],
        },
    };
    for field in ["force", "approved", "hooks"] {
        let mut raw = serde_json::to_value(&request).unwrap();
        raw[field] = serde_json::json!(true);
        assert!(serde_json::from_value::<Request>(raw).is_err());
    }
}
