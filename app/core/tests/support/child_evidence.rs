use super::*;
use gitfleet_core::remote::push::{ChildReachability, ChildReachabilitySelection};
pub(super) fn config_child(t: &Test, url: &str) {
    fs::write(
        t.f.repo.join(".gitmodules"),
        format!("[submodule \"fixture\"]\npath = child\nurl = {url}\n"),
    )
    .unwrap();
    t.f.git(&["add", ".gitmodules"]);
}
pub(super) fn commit_link(t: &Test, oid: &str) {
    t.f.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{oid},child"),
    ]);
    t.f.git(&["commit", "-qm", "Historical child requirement"]);
}
#[test]
fn historical_mappings_keep_url_changes_and_ignore_working_files_and_includes() {
    for sha256 in [false, true] {
        let mut t = setup_format(true, sha256);
        trust(&mut t);
        let observation = observed(&mut t);
        let child = "a".repeat(if sha256 { 64 } else { 40 });
        config_child(&t, "https://example.invalid/first.git");
        commit_link(&t, &child);
        config_child(&t, "https://example.invalid/second.git");
        commit_link(&t, &child);
        let include = t.f.root.join("untrusted-include");
        fs::write(
            &include,
            "[submodule \"extra\"]\npath=child\nurl=https://wrong.invalid/repo.git\n",
        )
        .unwrap();
        fs::write(t.f.repo.join(".gitmodules"), format!("[include]\npath={}\n[submodule \"fixture\"]\npath=child\nurl=https://example.invalid/second.git\n", include.display())).unwrap();
        t.f.git(&["add", ".gitmodules"]);
        t.f.git(&[
            "commit",
            "-qm",
            "Includes cannot supply destination authority",
        ]);
        fs::write(t.f.repo.join(".gitmodules"), "[broken working file").unwrap();
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.remote);
        let preview = push_preview(&mut t, &observation.plan.id);
        assert_eq!(preview.outgoing_gitlinks.len(), 1);
        let mappings = &preview.outgoing_gitlinks[0].mappings;
        assert_eq!(mappings.len(), 3);
        assert!(mappings
            .iter()
            .all(|m| m.issue.is_none() && m.gitmodules_blob.is_some()));
        assert!(mappings.iter().any(
            |m| m.destination.as_ref().unwrap().display == "https://example.invalid/first.git"
        ));
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.remote), remote);
    }
}
#[test]
fn historical_mappings_refuse_ambiguity_unsafe_relative_urls_secrets_and_nonregular_blobs() {
    let mut t = setup(true);
    trust(&mut t);
    let observation = observed(&mut t);
    let child = "a".repeat(40);
    for content in [
        "[submodule \"one\"]\npath=child\nurl=../../../../../../../../../../outside.git\n",
        "[submodule \"one\"]\npath=child\nurl=https://user:fixture-hidden-secret@example.invalid/repo.git?fixture-hidden-secret\n",
        "[submodule \"one\"]\npath=child\nurl=https://example.invalid/one.git\nurl=https://example.invalid/two.git\n",
        "[submodule \"one\"]\npath=child\nurl\nurl=https://example.invalid/one.git\n",
        "[submodule \"one\"]\npath=child\nurl=https://example.invalid/one.git\n[submodule \"two\"]\npath=child\nurl=https://example.invalid/two.git\n",
        "[invalid config",
    ] { fs::write(t.f.repo.join(".gitmodules"),content).unwrap(); t.f.git(&["add", ".gitmodules"]); commit_link(&t, &child); }
    // A committed symlink is classified, never opened as a configuration file.
    fs::remove_file(t.f.repo.join(".gitmodules")).unwrap();
    std::os::unix::fs::symlink(t.f.root.join("not-read"), t.f.repo.join(".gitmodules")).unwrap();
    // Git refuses staging symlink .gitmodules, so construct that malicious tree
    // through its object API without bypassing any application guard.
    let tree = t.f.git(&["write-tree"]);
    let mut raw =
        t.f.git(&["ls-tree", "-z", std::str::from_utf8(&tree).unwrap().trim()]);
    // Existing .gitmodules blob's content is reused as a link target; only mode differs.
    // Find the .gitmodules record explicitly; another ordinary file may precede it.
    let records = raw
        .split_mut(|b| *b == 0)
        .filter(|r| r.ends_with(b"\t.gitmodules"));
    for record in records {
        record[..6].copy_from_slice(b"120000");
    }
    use std::io::Write;
    let mut p = std::process::Command::new("git")
        .current_dir(&t.f.repo)
        .args(["mktree", "-z", "--missing"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    p.stdin.take().unwrap().write_all(&raw).unwrap();
    let output = p.wait_with_output().unwrap();
    assert!(output.status.success());
    let parent = t.f.git(&["rev-parse", "HEAD"]);
    let commit = t.f.git(&[
        "commit-tree",
        std::str::from_utf8(&output.stdout).unwrap().trim(),
        "-p",
        std::str::from_utf8(&parent).unwrap().trim(),
        "-m",
        "Reject symlink mapping",
    ]);
    t.f.git(&[
        "update-ref",
        "refs/heads/main",
        std::str::from_utf8(&commit).unwrap().trim(),
    ]);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let preview = push_preview(&mut t, &observation.plan.id);
    assert!(preview.outgoing_gitlinks[0]
        .mappings
        .iter()
        .all(|m| m.issue.is_some()));
    assert!(!serde_json::to_string(&preview)
        .unwrap()
        .contains("fixture-hidden-secret"));
    assert!(preview.outgoing_gitlinks[0]
        .mappings
        .iter()
        .any(|m| m.issue.as_deref() == Some("Historical .gitmodules is not a regular blob")));
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}
fn evidence(
    t: &mut Test,
    selection: ChildReachabilitySelection,
) -> gitfleet_core::Result<ChildReachability> {
    let data = t
        .app
        .request(
            t.app.epoch(),
            Request::ChildReachability {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                selection,
            },
        )?
        .data;
    let Data::ChildReachability(value) = data else {
        panic!()
    };
    Ok(*value)
}
pub(super) fn setup_evidence(sha256: bool, unrelated: bool) -> (Test, ChildReachabilitySelection) {
    let mut t = setup_format(true, sha256);
    let child = t.f.repo.join("child");
    t.f.git(&[
        "init",
        "-q",
        "-b",
        "main",
        if sha256 {
            "--object-format=sha256"
        } else {
            "--object-format=sha1"
        },
        child.to_str().unwrap(),
    ]);
    for (name, value) in [
        ("user.name", "Child fixture"),
        ("user.email", "child@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        t.f.git(&["-C", child.to_str().unwrap(), "config", name, value]);
    }
    fs::write(child.join("child.txt"), b"child fixture").unwrap();
    t.f.git(&["-C", child.to_str().unwrap(), "add", "child.txt"]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "commit",
        "-qm",
        "Child baseline",
    ]);
    let root_oid =
        String::from_utf8(t.f.git(&["-C", child.to_str().unwrap(), "rev-parse", "HEAD"]))
            .unwrap()
            .trim()
            .to_owned();
    fs::write(child.join("child.txt"), b"child next").unwrap();
    t.f.git(&["-C", child.to_str().unwrap(), "add", "child.txt"]);
    t.f.git(&["-C", child.to_str().unwrap(), "commit", "-qm", "Child tip"]);
    let child_remote = t.f.root.join("child-remote.git");
    t.f.git(&[
        "clone",
        "-q",
        "--bare",
        child.to_str().unwrap(),
        child_remote.to_str().unwrap(),
    ]);
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "remote",
        "add",
        "origin",
        child_remote.to_str().unwrap(),
    ]);
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
    let child_id = inventory
        .entries
        .iter()
        .find(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&child).unwrap())
        .unwrap()
        .id
        .0
        .clone();
    let parent_id = std::mem::replace(&mut t.worktree, child_id.clone());
    trust(&mut t);
    let child_observation = observed(&mut t);
    t.worktree = parent_id;
    trust(&mut t);
    let parent_observation = observed(&mut t);
    let required = if unrelated {
        let tree =
            t.f.git(&["-C", child.to_str().unwrap(), "rev-parse", "HEAD^{tree}"]);
        String::from_utf8(t.f.git(&[
            "-C",
            child.to_str().unwrap(),
            "commit-tree",
            std::str::from_utf8(&tree).unwrap().trim(),
            "-m",
            "Unreachable local child",
        ]))
        .unwrap()
        .trim()
        .to_owned()
    } else {
        root_oid
    };
    config_child(&t, child_remote.to_str().unwrap());
    commit_link(&t, &required);
    let preview = push_preview(&mut t, &parent_observation.plan.id);
    let r = &preview.outgoing_gitlinks[0];
    let selection = ChildReachabilitySelection {
        parent_observation: parent_observation.plan.id,
        source_oid: preview.source_oid.clone(),
        path: r.path.clone(),
        child_oid: r.oid.clone(),
        gitmodules_blob: r.mappings[0].gitmodules_blob.clone().unwrap(),
        child_worktree: child_id,
        child_observation: child_observation.plan.id,
    };
    (t, selection)
}
#[test]
fn selected_child_observation_proves_only_actual_ancestry_and_preserves_all_repositories() {
    for (sha256, unrelated) in [(false, false), (true, false), (false, true)] {
        let (mut t, selection) = setup_evidence(sha256, unrelated);
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.remote);
        let child_remote = tree_bytes(&t.f.root.join("child-remote.git"));
        let value = evidence(&mut t, selection.clone()).unwrap();
        assert_eq!(value.reachable, !unrelated);
        assert_eq!(value.selection.child_oid, selection.child_oid);
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.remote), remote);
        assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), child_remote);
        assert!(t
            .app
            .begin_confirmation(&selection.child_observation)
            .is_err());
    }
}
#[test]
fn child_evidence_rejects_mismatches_expiry_missing_objects_config_drift_and_cancellation() {
    let (mut t, selection) = setup_evidence(false, false);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let mut bad = selection.clone();
    bad.path = b"unselected".to_vec();
    assert!(evidence(&mut t, bad).is_err());
    let mut bad = selection.clone();
    bad.source_oid.hex = "a".repeat(40);
    assert!(evidence(&mut t, bad).is_err());
    let mut bad = selection.clone();
    bad.gitmodules_blob.hex = "a".repeat(40);
    assert!(evidence(&mut t, bad).is_err());
    let mut bad = selection.clone();
    bad.child_observation = selection.parent_observation.clone();
    assert!(evidence(&mut t, bad).is_err());
    let child = t.f.repo.join("child");
    let config = fs::read(child.join(".git/config")).unwrap();
    t.f.git(&[
        "-C",
        child.to_str().unwrap(),
        "config",
        "remote.origin.url",
        t.remote.to_str().unwrap(),
    ]);
    assert!(evidence(&mut t, selection.clone()).is_err());
    fs::write(child.join(".git/config"), config).unwrap();
    let tip =
        t.f.git(&["-C", child.to_str().unwrap(), "rev-parse", "HEAD"]);
    fs::write(child.join(".git/shallow"), &tip).unwrap();
    assert!(evidence(&mut t, selection.clone()).is_err());
    fs::remove_file(child.join(".git/shallow")).unwrap();
    let token = Cancellation::default();
    token.cancel();
    assert_eq!(
        t.app
            .request_with_cancellation(
                t.app.epoch(),
                Request::ChildReachability {
                    workspace: t.workspace.clone(),
                    worktree: t.worktree.clone(),
                    selection: selection.clone()
                },
                token
            )
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    let object = child
        .join(".git/objects")
        .join(&selection.child_oid.hex[..2])
        .join(&selection.child_oid.hex[2..]);
    let bytes = fs::read(&object).unwrap();
    let permissions = fs::metadata(&object).unwrap().permissions();
    fs::remove_file(&object).unwrap();
    assert!(evidence(&mut t, selection.clone()).is_err());
    fs::write(&object, bytes).unwrap();
    fs::set_permissions(&object, permissions).unwrap();
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let key = format!("remote-observation:{}", selection.child_observation);
    let raw: String = db
        .query_row("select value from state where key=?1", [&key], |r| r.get(0))
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    value["value"]["observed_at_ms"] = serde_json::json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - 300_001
    );
    db.execute(
        "update state set value=?1 where key=?2",
        rusqlite::params![value.to_string(), key],
    )
    .unwrap();
    assert!(evidence(&mut t, selection.clone()).is_err());
    value["value"]["observed_at_ms"] = serde_json::json!(u64::MAX);
    db.execute(
        "update state set value=?1 where key=?2",
        rusqlite::params![value.to_string(), key],
    )
    .unwrap();
    assert!(evidence(&mut t, selection).is_err());
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}

#[test]
fn historical_mapping_contexts_are_bounded_without_partial_success() {
    let mut t = setup(true);
    trust(&mut t);
    let observation = observed(&mut t);
    use std::io::Write;
    let mut input = Vec::new();
    for i in 0..1024 {
        input.extend_from_slice(format!("160000 {}\tchild-{i}\0", "a".repeat(40)).as_bytes());
    }
    let mut p = std::process::Command::new("git")
        .current_dir(&t.f.repo)
        .args(["update-index", "-z", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    p.stdin.take().unwrap().write_all(&input).unwrap();
    assert!(p.wait().unwrap().success());
    for i in 0..5 {
        fs::write(t.f.repo.join(".gitmodules"),format!("[submodule \"fixture\"]\npath=child-0\nurl=https://example.invalid/version-{i}.git\n")).unwrap();
        t.f.git(&["add", ".gitmodules"]);
        t.f.git(&["commit", "-qm", "New historical mapping"]);
        if i == 3 {
            let preview = push_preview(&mut t, &observation.plan.id);
            assert_eq!(
                preview
                    .outgoing_gitlinks
                    .iter()
                    .map(|r| r.mappings.len())
                    .sum::<usize>(),
                4096
            );
        }
    }
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let error = t
        .app
        .request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.plan.id,
            },
        )
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::OutputLimit);
    assert!(error.to_string().contains("4,096 mapping contexts"));
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}
#[test]
fn child_evidence_rejects_historical_destination_mismatch() {
    let (mut t, mut selection) = setup_evidence(false, false);
    config_child(&t, "https://different.invalid/child.git");
    commit_link(&t, &selection.child_oid.hex);
    let preview = push_preview(&mut t, &selection.parent_observation);
    selection.source_oid = preview.source_oid;
    selection.gitmodules_blob = preview.outgoing_gitlinks[0]
        .mappings
        .iter()
        .find(|m| {
            m.destination
                .as_ref()
                .is_some_and(|d| d.display == "https://different.invalid/child.git")
        })
        .unwrap()
        .gitmodules_blob
        .clone()
        .unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.f.root.join("child-remote.git"));
    assert_eq!(
        evidence(&mut t, selection).unwrap_err().kind,
        ErrorKind::StaleReview
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remote);
}

#[test]
fn relative_child_mapping_uses_selected_parent_destination_and_proves_observed_ancestry() {
    for sha256 in [false, true] {
        let (mut t, mut selection) = setup_evidence(sha256, false);
        config_child(&t, "../child-remote.git");
        commit_link(&t, &selection.child_oid.hex);
        let preview = push_preview(&mut t, &selection.parent_observation);
        let mapping = preview.outgoing_gitlinks[0]
            .mappings
            .iter()
            .find(|m| m.declared_url.as_deref() == Some("../child-remote.git"))
            .unwrap();
        assert!(mapping.issue.is_none());
        assert_eq!(
            mapping.resolution_base.as_deref(),
            Some(preview.destination.as_str())
        );
        selection.source_oid = preview.source_oid.clone();
        selection.gitmodules_blob = mapping.gitmodules_blob.clone().unwrap();
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.f.root.join("child-remote.git"));
        let value = evidence(&mut t, selection).unwrap();
        assert!(value.reachable);
        assert_eq!(
            value.mapping.declared_url.as_deref(),
            Some("../child-remote.git")
        );
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(tree_bytes(&t.f.root.join("child-remote.git")), remote);
    }
}

#[test]
fn real_agent_authenticated_child_evidence_resolves_relative_ssh_mapping_without_new_contact() {
    let (mut t, mut selection) = setup_evidence(false, false);
    let _server = super::ssh_transport_tests::server(&mut t);
    let parent_url = String::from_utf8(t.f.git(&["remote", "get-url", "origin"]))
        .unwrap()
        .trim()
        .to_owned();
    let child_remote = t.f.root.join("child-remote.git");
    let child_url = parent_url
        .strip_suffix(t.remote.to_str().unwrap())
        .unwrap()
        .to_owned()
        + child_remote.to_str().unwrap();
    fs::write(t.f.root.join("ssh-routing.json"),serde_json::to_vec(&serde_json::json!({t.remote.to_str().unwrap():t.remote.to_str().unwrap(),child_remote.to_str().unwrap():child_remote.to_str().unwrap()})).unwrap()).unwrap();
    t.f.git(&[
        "-C",
        t.f.repo.join("child").to_str().unwrap(),
        "remote",
        "set-url",
        "origin",
        &child_url,
    ]);
    config_child(&t, "../child-remote.git");
    commit_link(&t, &selection.child_oid.hex);
    let parent_id = std::mem::replace(&mut t.worktree, selection.child_worktree.clone());
    trust(&mut t);
    let observed_child = observed(&mut t);
    assert_eq!(observed_child.phase, ObservationPhase::Complete);
    t.worktree = parent_id;
    trust(&mut t);
    let observed_parent = observed(&mut t);
    assert_eq!(observed_parent.phase, ObservationPhase::Complete);
    let preview = push_preview(&mut t, &observed_parent.plan.id);
    let mapping = preview.outgoing_gitlinks[0]
        .mappings
        .iter()
        .find(|m| m.declared_url.as_deref() == Some("../child-remote.git"))
        .unwrap();
    assert_eq!(mapping.destination.as_ref().unwrap().display, child_url);
    selection.parent_observation = observed_parent.plan.id;
    selection.child_observation = observed_child.plan.id;
    selection.source_oid = preview.source_oid;
    selection.gitmodules_blob = mapping.gitmodules_blob.clone().unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&child_remote);
    let requests = fs::read(t.f.root.join("ssh-requests")).unwrap();
    assert!(evidence(&mut t, selection).unwrap().reachable);
    assert_eq!(fs::read(t.f.root.join("ssh-requests")).unwrap(), requests);
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&child_remote), remote);
}
