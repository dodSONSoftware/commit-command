use super::child_evidence::{commit_link, setup_evidence};
use super::*;
use gitfleet_core::remote::push::*;
fn preview_for(t: &mut Test, id: &str, observation: &str) -> PushPreview {
    let Data::PushPreview(value) = t
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
    value
}
fn selection(
    t: &mut Test,
    id: &str,
    observation: &str,
    children: Vec<ChildReachabilitySelection>,
) -> PushPlanningSelection {
    PushPlanningSelection {
        worktree: id.into(),
        observation: observation.into(),
        source_oid: preview_for(t, id, observation).source_oid,
        children,
    }
}
fn plan(
    t: &mut Test,
    selections: Vec<PushPlanningSelection>,
) -> gitfleet_core::Result<PushDependencyPlan> {
    let Data::PushDependencyPlan(value) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushDependencyPlan {
                workspace: t.workspace.clone(),
                selections,
            },
        )?
        .data
    else {
        panic!()
    };
    Ok(value)
}
fn pending_child(sha256: bool) -> (Test, ChildReachabilitySelection) {
    let (mut t, mut binding) = setup_evidence(sha256, false);
    let child = t.f.repo.join("child");
    fs::write(child.join("new.txt"), b"Selected outgoing child commit\n").unwrap();
    t.f.git(&["-C", child.to_str().unwrap(), "add", "new.txt"]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "commit",
        "-qm",
        "Pending child publication",
    ]);
    let oid = String::from_utf8(t.f.git(&["-C", child.to_str().unwrap(), "rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    commit_link(&t, &oid);
    let p = push_preview(&mut t, &binding.parent_observation);
    let r = &p
        .outgoing_gitlinks
        .iter()
        .find(|r| r.oid.hex == oid)
        .unwrap();
    binding.source_oid = p.source_oid;
    binding.child_oid = r.oid.clone();
    binding.gitmodules_blob = r.mappings[0].gitmodules_blob.clone().unwrap();
    (t, binding)
}
fn all_bindings(
    t: &mut Test,
    id: &str,
    observation: &str,
    child_id: &str,
    child_observation: &str,
) -> Vec<ChildReachabilitySelection> {
    let p = preview_for(t, id, observation);
    p.outgoing_gitlinks
        .iter()
        .flat_map(|r| {
            r.mappings.iter().map(|m| ChildReachabilitySelection {
                parent_observation: observation.into(),
                source_oid: p.source_oid.clone(),
                path: r.path.clone(),
                child_oid: r.oid.clone(),
                gitmodules_blob: m.gitmodules_blob.clone().unwrap(),
                child_worktree: child_id.into(),
                child_observation: child_observation.into(),
            })
        })
        .collect()
}
#[test]
fn planning_never_adds_unselected_children_and_requires_remote_evidence() {
    let (mut t, b) = setup_evidence(false, false);
    let parent = t.worktree.clone();
    let s = selection(&mut t, &parent, &b.parent_observation, vec![b.clone()]);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let child_remote = tree_bytes(&t.f.root.join("child-remote.git"));
    let value = plan(&mut t, vec![s.clone()]).unwrap();
    assert_eq!(value.order, vec![parent.clone()]);
    assert_eq!(value.items.len(), 1);
    assert!(value
        .dependencies
        .iter()
        .all(|e| e.state == DependencyState::AlreadyReachable && !e.child_selected));
    let mut missing = s.clone();
    missing.children.clear();
    let value = plan(&mut t, vec![missing]).unwrap();
    assert!(value.order.is_empty());
    assert_eq!(value.dependencies[0].state, DependencyState::Blocked);
    let mut bad = s.clone();
    bad.children.push(b.clone());
    assert_eq!(
        plan(&mut t, vec![bad]).unwrap_err().kind,
        ErrorKind::InvalidRequest
    );
    assert_eq!(
        plan(&mut t, vec![s.clone(), s.clone()]).unwrap_err().kind,
        ErrorKind::InvalidRequest
    );
    let mut extra = s.clone();
    extra.children[0].path = b"not-a-requirement".to_vec();
    assert_eq!(
        plan(&mut t, vec![extra]).unwrap_err().kind,
        ErrorKind::InvalidRequest
    );
    let mut drift = s.clone();
    drift.source_oid.hex = "a".repeat(40);
    assert_eq!(
        plan(&mut t, vec![drift]).unwrap_err().kind,
        ErrorKind::StaleReview
    );
    let token = Cancellation::default();
    token.cancel();
    assert_eq!(
        t.app
            .request_with_cancellation(
                t.app.epoch(),
                Request::PushDependencyPlan {
                    workspace: t.workspace.clone(),
                    selections: vec![s.clone()]
                },
                token
            )
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(
        plan(&mut t, vec![s; 33]).unwrap_err().kind,
        ErrorKind::InvalidRequest
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), child_remote);
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(
        db.query_row("select count(*) from write_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.query_row(
            "select count(*) from state where key like 'remote-observation:%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    assert!(t.app.begin_confirmation(&b.parent_observation).is_err());
}
#[test]
fn selected_pending_child_is_conditional_and_unselected_pending_child_blocks() {
    for sha256 in [false, true] {
        let (mut t, b) = pending_child(sha256);
        let parent = t.worktree.clone();
        let bindings = all_bindings(
            &mut t,
            &parent,
            &b.parent_observation,
            &b.child_worktree,
            &b.child_observation,
        );
        let a = selection(&mut t, &parent, &b.parent_observation, bindings);
        let c = selection(&mut t, &b.child_worktree, &b.child_observation, vec![]);
        let before = tree_bytes(&t.f.repo);
        let remotes = (
            tree_bytes(&t.remote),
            tree_bytes(&t.f.root.join("child-remote.git")),
        );
        let blocked = plan(&mut t, vec![a.clone()]).unwrap();
        assert!(blocked.order.is_empty());
        assert!(blocked
            .dependencies
            .iter()
            .any(|e| e.state == DependencyState::Blocked && !e.child_selected));
        let value = plan(&mut t, vec![a.clone(), c.clone()]).unwrap();
        assert_eq!(value.order, vec![b.child_worktree.clone(), parent]);
        assert!(value
            .dependencies
            .iter()
            .any(|e| e.state == DependencyState::SelectedChildRequired));
        let reverse = plan(&mut t, vec![c, a]).unwrap();
        assert_eq!(reverse.order, value.order);
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.remote), remotes.0);
        assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remotes.1);
    }
}
#[test]
fn selected_child_without_required_commit_cannot_satisfy_parent() {
    let (mut t, b) = setup_evidence(false, true);
    let parent = t.worktree.clone();
    let a = selection(&mut t, &parent, &b.parent_observation, vec![b.clone()]);
    let c = selection(&mut t, &b.child_worktree, &b.child_observation, vec![]);
    let result = plan(&mut t, vec![a, c]).unwrap();
    assert!(result.order.is_empty());
    assert_eq!(result.dependencies[0].state, DependencyState::Blocked);
    assert!(result.dependencies[0]
        .issue
        .as_deref()
        .unwrap()
        .contains("does not contain"));
}
#[test]
fn selected_two_repository_cycle_returns_closed_witness_and_no_order() {
    let (mut t, b) = setup_evidence(false, false);
    let parent = t.worktree.clone();
    let child = t.f.repo.join("child");
    let base = String::from_utf8(t.f.git(&["-C", t.remote.to_str().unwrap(), "rev-parse", "main"]))
        .unwrap()
        .trim()
        .to_owned();
    // A parent-repository commit is a valid gitlink target even without a checkout.
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "fetch",
        "-q",
        t.remote.to_str().unwrap(),
        "main",
    ]);
    fs::write(
        child.join(".gitmodules"),
        format!(
            "[submodule \"back\"]\npath=back\nurl={}\n",
            t.remote.display()
        ),
    )
    .unwrap();
    t.f.git(&["-C", child.to_str().unwrap(), "add", ".gitmodules"]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{base},back"),
    ]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "commit",
        "-qm",
        "Historical dependency back to parent",
    ]);
    let a = selection(&mut t, &parent, &b.parent_observation, vec![b.clone()]);
    let back = all_bindings(
        &mut t,
        &b.child_worktree,
        &b.child_observation,
        &parent,
        &b.parent_observation,
    );
    let c = selection(&mut t, &b.child_worktree, &b.child_observation, back);
    let before = tree_bytes(&t.f.repo);
    let result = plan(&mut t, vec![a, c]).unwrap();
    assert!(result.order.is_empty());
    assert_eq!(result.cycle.first(), result.cycle.last());
    assert_eq!(result.cycle.len(), 3);
    assert!(result.cycle.contains(&parent) && result.cycle.contains(&b.child_worktree));
    assert_eq!(tree_bytes(&t.f.repo), before);
}
#[test]
fn multilevel_selected_chain_is_ordered_grandchild_child_parent() {
    let (mut t, ab) = setup_evidence(false, false);
    let parent = t.worktree.clone();
    let child = t.f.repo.join("child");
    let grand = child.join("grand");
    fs::create_dir(&grand).unwrap();
    t.f.git(&["init", "-q", "-b", "main", grand.to_str().unwrap()]);
    for (k, v) in [
        ("user.name", "Grandchild fixture"),
        ("user.email", "grand@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        t.f.git(&["-C", grand.to_str().unwrap(), "config", k, v]);
    }
    fs::write(grand.join("file.txt"), b"Grand baseline\n").unwrap();
    t.f.git(&["-C", grand.to_str().unwrap(), "add", "file.txt"]);
    t.f.git(&[
        "-C",
        grand.to_str().unwrap(),
        "commit",
        "-qm",
        "Grand baseline",
    ]);
    let grand_remote = t.f.root.join("grand-remote.git");
    t.f.git(&[
        "clone",
        "-q",
        "--bare",
        grand.to_str().unwrap(),
        grand_remote.to_str().unwrap(),
    ]);
    t.f.git(&[
        "-C",
        grand.to_str().unwrap(),
        "remote",
        "add",
        "origin",
        grand_remote.to_str().unwrap(),
    ]);
    fs::write(grand.join("file.txt"), b"Grand selected outgoing\n").unwrap();
    t.f.git(&["-C", grand.to_str().unwrap(), "add", "file.txt"]);
    t.f.git(&[
        "-C",
        grand.to_str().unwrap(),
        "commit",
        "-qm",
        "Grand outgoing",
    ]);
    let grand_oid =
        String::from_utf8(t.f.git(&["-C", grand.to_str().unwrap(), "rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
    fs::write(
        child.join(".gitmodules"),
        format!(
            "[submodule \"grand\"]\npath=grand\nurl={}\n",
            grand_remote.display()
        ),
    )
    .unwrap();
    t.f.git(&["-C", child.to_str().unwrap(), "add", ".gitmodules"]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{grand_oid},grand"),
    ]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "commit",
        "-qm",
        "Child requires selected grandchild",
    ]);
    let child_oid =
        String::from_utf8(t.f.git(&["-C", child.to_str().unwrap(), "rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
    commit_link(&t, &child_oid);
    t.app
        .request(
            t.app.epoch(),
            Request::Scan {
                workspace: t.workspace.clone(),
            },
        )
        .unwrap();
    let Data::Inventory(inventory) = t
        .app
        .request(
            t.app.epoch(),
            Request::Inventory {
                workspace: t.workspace.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let grand_id = inventory
        .entries
        .iter()
        .find(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&grand).unwrap())
        .unwrap()
        .id
        .0
        .clone();
    t.worktree = grand_id.clone();
    trust(&mut t);
    let grand_observation = observed(&mut t);
    t.worktree = parent.clone();
    let a_bindings = all_bindings(
        &mut t,
        &parent,
        &ab.parent_observation,
        &ab.child_worktree,
        &ab.child_observation,
    );
    let b_bindings = all_bindings(
        &mut t,
        &ab.child_worktree,
        &ab.child_observation,
        &grand_id,
        &grand_observation.plan.id,
    );
    let a = selection(&mut t, &parent, &ab.parent_observation, a_bindings);
    let b = selection(
        &mut t,
        &ab.child_worktree,
        &ab.child_observation,
        b_bindings,
    );
    let c = selection(&mut t, &grand_id, &grand_observation.plan.id, vec![]);
    let before = tree_bytes(&t.f.repo);
    let remotes = (
        tree_bytes(&t.remote),
        tree_bytes(&t.f.root.join("child-remote.git")),
        tree_bytes(&grand_remote),
    );
    let value = plan(&mut t, vec![a.clone(), b.clone(), c]).unwrap();
    assert_eq!(
        value.order,
        vec![grand_id.clone(), ab.child_worktree.clone(), parent]
    );
    assert_eq!(
        value
            .dependencies
            .iter()
            .filter(|e| e.state == DependencyState::SelectedChildRequired)
            .count(),
        2
    );
    let missing = plan(&mut t, vec![a, b]).unwrap();
    assert!(missing.order.is_empty());
    assert_eq!(missing.items.len(), 2);
    assert!(missing
        .dependencies
        .iter()
        .any(|e| e.child_worktree.as_deref() == Some(grand_id.as_str())
            && !e.child_selected
            && e.state == DependencyState::Blocked));
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remotes.0);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remotes.1);
    assert_eq!(tree_bytes(&grand_remote), remotes.2);
}

#[test]
fn planning_blocks_non_fast_forward_and_expired_root_without_children() {
    let mut t = setup(true);
    trust(&mut t);
    let base = String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    advance_local(&t);
    t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "fetch",
        "-q",
        t.f.repo.to_str().unwrap(),
        "refs/heads/main:refs/heads/main",
    ]);
    let observation = observed(&mut t);
    t.f.git(&["update-ref", "refs/heads/main", &base]);
    let parent = t.worktree.clone();
    let s = selection(&mut t, &parent, &observation.plan.id, vec![]);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let value = plan(&mut t, vec![s.clone()]).unwrap();
    assert!(value.order.is_empty());
    assert!(value.dependencies.is_empty());
    assert!(!value.issues.is_empty());
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let key = format!("remote-observation:{}", observation.plan.id);
    let raw: String = db
        .query_row("select value from state where key=?1", [&key], |r| r.get(0))
        .unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["value"]["observed_at_ms"] = serde_json::json!(0);
    db.execute(
        "update state set value=?1 where key=?2",
        rusqlite::params![v.to_string(), key],
    )
    .unwrap();
    assert_eq!(
        plan(&mut t, vec![s]).unwrap_err().kind,
        ErrorKind::StaleReview
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}
#[test]
fn planning_refuses_two_selected_sources_for_one_destination_branch() {
    let (mut t, b) = setup_evidence(false, false);
    let parent = t.worktree.clone();
    let child = t.f.repo.join("child");
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "remote",
        "set-url",
        "origin",
        t.remote.to_str().unwrap(),
    ]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "fetch",
        "-q",
        t.remote.to_str().unwrap(),
        "main",
    ]);
    t.worktree = b.child_worktree.clone();
    trust(&mut t);
    let child_observation = observed(&mut t);
    t.worktree = parent.clone();
    let a = selection(&mut t, &parent, &b.parent_observation, vec![]);
    let c = selection(
        &mut t,
        &b.child_worktree,
        &child_observation.plan.id,
        vec![],
    );
    let before = tree_bytes(&t.f.repo);
    let error = plan(&mut t, vec![a, c]).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidRequest);
    assert!(error.to_string().contains("distinct destination branches"));
    assert_eq!(tree_bytes(&t.f.repo), before);
}
