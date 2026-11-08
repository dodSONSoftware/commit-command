use super::*;
use gitfleet_core::remote::{push::*, push_check::*};
use gitfleet_core::writes::HookPolicy;
use std::os::unix::fs::PermissionsExt;

pub(super) fn ready(sha256: bool) -> (Test, PushPlanningSelection) {
    let mut t = setup_format(true, sha256);
    trust(&mut t);
    let observation = observed(&mut t);
    fs::write(t.f.repo.join("outgoing.txt"), b"Pending source\n").unwrap();
    t.f.git(&["add", "outgoing.txt"]);
    t.f.git(&["commit", "-qm", "Outgoing check fixture"]);
    let Data::PushPreview(p) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.plan.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let s = PushPlanningSelection {
        worktree: t.worktree.clone(),
        observation: observation.plan.id,
        source_oid: p.source_oid,
        children: vec![],
    };
    (t, s)
}
fn install(t: &Test, body: &str) {
    let path = t.f.repo.join(".git/hooks/pre-push");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn propose(
    t: &mut Test,
    s: PushPlanningSelection,
    hooks: HookPolicy,
) -> gitfleet_core::Result<ReviewDisplay> {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushCheck {
                workspace: t.workspace.clone(),
                selection: s,
                hooks,
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
    token: Cancellation,
) -> gitfleet_core::Result<PushCheck> {
    let (cap, _) = t.app.begin_confirmation(&r.id)?;
    serde_json::from_value(t.app.confirm(cap, accepted, token)?).map_err(Into::into)
}
fn audit(t: &mut Test, result: Option<OperationAuditResult>) -> OperationAuditPage {
    let Data::OperationAudit(p) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: Some(t.worktree.clone()),
                    operation_kind: Some(OperationAuditKind::PushCheck),
                    result,
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
    p
}
#[test]
fn native_capability_runs_exact_pre_push_payload_once_without_publication() {
    for sha256 in [false, true] {
        let (mut t, s) = ready(sha256);
        let args = t.f.root.join("args");
        let input = t.f.root.join("stdin");
        install(
            &t,
            &format!(
                "printf '%s\\n' \"$PWD\" \"$@\" > '{}'\ncat > '{}'",
                args.display(),
                input.display()
            ),
        );
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.remote);
        let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
        let p: PushCheckPlan = serde_json::from_str(&r.content).unwrap();
        assert!(r.summary.contains("no publication authority"));
        assert!(!args.exists());
        assert_eq!(
            audit(&mut t, Some(OperationAuditResult::Pending))
                .items
                .len(),
            1
        );
        let check = finish(&mut t, &r, true, Cancellation::default()).unwrap();
        assert_eq!(check.phase, PushCheckPhase::Complete);
        assert!(check.hook_passed);
        assert!(!check.published);
        assert_eq!(
            fs::read_to_string(args).unwrap(),
            format!(
                "{}\norigin\n{}\n",
                fs::canonicalize(&t.f.repo).unwrap().display(),
                p.preview.destination
            )
        );
        assert_eq!(fs::read_to_string(input).unwrap(), p.hook_stdin);
        assert_eq!(
            p.hook_stdin,
            format!(
                "{} {} {} {}\n",
                p.preview.source_ref,
                p.preview.source_oid.hex,
                p.preview.destination_ref,
                p.preview.observed_oid.hex
            )
        );
        assert_eq!(
            audit(&mut t, Some(OperationAuditResult::Complete))
                .items
                .len(),
            1
        );
        assert!(t.app.begin_confirmation(&r.id).is_err());
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.remote), remote);
    }
}
#[test]
fn decline_bypass_and_unsupported_manager_never_run_hook() {
    let (mut t, s) = ready(false);
    let marker = t.f.root.join("marker");
    install(
        &t,
        &format!(
            "# husky unsupported fixture\nprintf run >> '{}'",
            marker.display()
        ),
    );
    assert_eq!(
        propose(&mut t, s.clone(), HookPolicy::Execute)
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedRepositoryState
    );
    let r = propose(&mut t, s.clone(), HookPolicy::BypassAll).unwrap();
    assert!(r.summary.contains("BypassAll"));
    assert_eq!(
        finish(&mut t, &r, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    let r = propose(&mut t, s, HookPolicy::BypassAll).unwrap();
    let value = finish(&mut t, &r, true, Cancellation::default()).unwrap();
    assert!(!value.hook_passed);
    assert!(!value.published);
    assert!(!marker.exists());
}
#[test]
fn changed_hook_or_source_invalidates_approved_check_before_hook() {
    for source in [false, true] {
        let (mut t, s) = ready(false);
        let marker = t.f.root.join("marker");
        install(&t, &format!("printf run >> '{}'", marker.display()));
        let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
        if source {
            fs::write(t.f.repo.join("drift"), b"Source drift").unwrap();
            t.f.git(&["add", "drift"]);
            t.f.git(&["commit", "-qm", "Drift"]);
        } else {
            install(&t, &format!("printf changed >> '{}'", marker.display()));
        }
        assert_eq!(
            finish(&mut t, &r, true, Cancellation::default())
                .unwrap_err()
                .kind,
            ErrorKind::StaleReview
        );
        assert!(!marker.exists());
        assert_eq!(
            audit(&mut t, Some(OperationAuditResult::Blocked))
                .items
                .len(),
            1
        );
    }
}
#[test]
fn rejected_or_precancelled_check_keeps_no_raw_hook_output_or_authority() {
    let (mut t, s) = ready(false);
    install(
        &t,
        "printf 'private-hook-output'\nprintf 'private-hook-error' >&2\nexit 3",
    );
    let remote = tree_bytes(&t.remote);
    let r = propose(&mut t, s.clone(), HookPolicy::Execute).unwrap();
    let error = finish(&mut t, &r, true, Cancellation::default()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::HookRejected);
    assert!(!error.to_string().contains("private-hook"));
    assert!(!serde_json::to_string(&audit(&mut t, None))
        .unwrap()
        .contains("private-hook"));
    let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
    let token = Cancellation::default();
    token.cancel();
    assert_eq!(
        finish(&mut t, &r, true, token).unwrap_err().kind,
        ErrorKind::Cancelled
    );
    assert_eq!(tree_bytes(&t.remote), remote);
}
#[test]
fn hook_source_mutation_blocks_completion_without_rolling_back_hook_effects() {
    let (mut t, s) = ready(false);
    let old = String::from_utf8(t.f.git(&["rev-parse", "HEAD^"])).unwrap();
    install(
        &t,
        &format!("git update-ref refs/heads/main {}", old.trim()),
    );
    let remote = tree_bytes(&t.remote);
    let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
    assert_eq!(
        finish(&mut t, &r, true, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::StaleReview
    );
    let p = audit(&mut t, Some(OperationAuditResult::Blocked));
    let OperationAuditRecord::PushCheck(v) = &p.items[0].record else {
        panic!()
    };
    assert!(!v.published);
    assert!(v.hook_passed);
    assert_eq!(tree_bytes(&t.remote), remote);
}
#[test]
fn conditional_selected_child_is_not_remote_success_for_parent_check() {
    let (mut t, b) = super::push_dependencies::pending_child(false);
    let s = PushPlanningSelection {
        worktree: t.worktree.clone(),
        observation: b.parent_observation.clone(),
        source_oid: b.source_oid.clone(),
        children: vec![b.clone()],
    };
    let before = tree_bytes(&t.remote);
    assert_eq!(
        propose(&mut t, s, HookPolicy::Execute).unwrap_err().kind,
        ErrorKind::UnsupportedRepositoryState
    );
    assert_eq!(tree_bytes(&t.remote), before);
    assert!(audit(&mut t, None).items.is_empty());
}
#[test]
fn restart_projects_orphaned_checks_as_interrupted_without_replaying_hooks() {
    let (mut t, s) = ready(false);
    let marker = t.f.root.join("marker");
    install(&t, &format!("printf run >> '{}'", marker.display()));
    let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
    let plan: PushCheckPlan = serde_json::from_str(&r.content).unwrap();
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let key = format!("push-check:{}", plan.id);
    let raw: String = db
        .query_row("select value from state where key=?1", [&key], |r| r.get(0))
        .unwrap();
    let mut stored: serde_json::Value = serde_json::from_str(&raw).unwrap();
    stored["value"]["phase"] = serde_json::json!("running");
    db.execute(
        "update state set value=?1 where key=?2",
        rusqlite::params![stored.to_string(), key],
    )
    .unwrap();
    drop(db);
    reopen(&mut t);
    assert!(t.app.begin_confirmation(&r.id).is_err());
    let page = audit(&mut t, Some(OperationAuditResult::Interrupted));
    assert_eq!(page.items.len(), 1);
    let OperationAuditRecord::PushCheck(v) = &page.items[0].record else {
        panic!()
    };
    assert_eq!(v.phase, PushCheckPhase::Interrupted);
    assert!(!v.published);
    assert!(!marker.exists());
}

#[test]
fn running_hook_cancellation_stops_check_and_cannot_replay() {
    let (mut t, s) = ready(false);
    let started = t.f.root.join("hook-started");
    let finished = t.f.root.join("hook-finished");
    install(
        &t,
        &format!(
            "printf started > '{}'\nsleep 30\nprintf finished > '{}'",
            started.display(),
            finished.display()
        ),
    );
    let r = propose(&mut t, s, HookPolicy::Execute).unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let token = Cancellation::default();
    let cancel = token.clone();
    let marker = started.clone();
    let waiter = std::thread::spawn(move || {
        for _ in 0..1500 {
            if marker.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        cancel.cancel();
    });
    let result = finish(&mut t, &r, true, token);
    waiter.join().unwrap();
    assert!(
        started.exists(),
        "Cancellation must reach a proven running hook"
    );
    assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
    assert!(!finished.exists());
    assert!(t.app.begin_confirmation(&r.id).is_err());
    assert_eq!(
        audit(&mut t, Some(OperationAuditResult::Cancelled))
            .items
            .len(),
        1
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}

#[test]
fn child_gate_requires_new_observation_after_fixture_remote_advances() {
    let (mut t, mut b) = super::push_dependencies::pending_child(false);
    let parent = t.worktree.clone();
    let bindings = super::push_dependencies::all_bindings(
        &mut t,
        &parent,
        &b.parent_observation,
        &b.child_worktree,
        &b.child_observation,
    );
    let mut selection = PushPlanningSelection {
        worktree: parent.clone(),
        observation: b.parent_observation.clone(),
        source_oid: b.source_oid.clone(),
        children: bindings,
    };
    assert!(propose(&mut t, selection.clone(), HookPolicy::Execute).is_err());
    // Fixture seeding uses fetch into a disposable bare remote; no Git push runs.
    let child = t.f.repo.join("child");
    let remote = t.f.root.join("child-remote.git");
    t.f.git(&[
        "-C",
        remote.to_str().unwrap(),
        "-c",
        "protocol.file.allow=always",
        "fetch",
        "-q",
        child.to_str().unwrap(),
        "refs/heads/main:refs/heads/main",
    ]);
    assert!(
        propose(&mut t, selection.clone(), HookPolicy::Execute).is_err(),
        "Remote movement alone cannot change old observed evidence"
    );
    t.worktree = b.child_worktree.clone();
    let fresh = observed(&mut t);
    t.worktree = parent.clone();
    b.child_observation = fresh.plan.id;
    selection.children = super::push_dependencies::all_bindings(
        &mut t,
        &parent,
        &b.parent_observation,
        &b.child_worktree,
        &b.child_observation,
    );
    let before = tree_bytes(&t.f.repo);
    let remote_before = tree_bytes(&remote);
    let r = propose(&mut t, selection, HookPolicy::Execute).unwrap();
    let value = finish(&mut t, &r, true, Cancellation::default()).unwrap();
    assert_eq!(value.phase, PushCheckPhase::Complete);
    assert!(value
        .plan
        .dependencies
        .iter()
        .all(|d| d.state == DependencyState::AlreadyReachable));
    assert!(!value.published);
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&remote), remote_before);
}
