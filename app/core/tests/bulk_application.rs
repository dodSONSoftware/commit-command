mod support;
use gitfleet_core::{application::*, domain::*, inspection::Cancellation, writes::*, ErrorKind};
use std::{fs, path::Path};
use support::{tree_bytes, Fixture};
struct Test {
    f: Fixture,
    app: Application,
    ws: String,
    entries: Vec<gitfleet_core::workspace::StoredWorktree>,
}
fn setup(count: usize) -> Test {
    let f = Fixture::new();
    f.commit_file();
    for i in 1..count {
        let repo = f.repo.join(format!("child-{i}"));
        let p = repo.to_str().unwrap();
        f.git(&["-c", "init.templateDir=", "init", "-q", "-b", "main", p]);
        for (key, value) in [
            ("user.name", "Batch fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
        ] {
            f.git(&["-C", p, "config", key, value]);
        }
        fs::write(repo.join("file.txt"), b"original\n").unwrap();
        f.git(&["-C", p, "add", "file.txt"]);
        f.git(&[
            "-C",
            p,
            "-c",
            "core.hooksPath=",
            "commit",
            "-qm",
            "baseline",
        ]);
    }
    let global = f.root.join("global.config");
    fs::write(&global, b"").unwrap();
    let home = f.root.join("home");
    fs::create_dir(&home).unwrap();
    let mut app = Application::open(
        &f.root.join("state"),
        Options {
            global_config: Some(global),
            signer_home: Some(home),
            ..Options::default()
        },
    )
    .unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "Bulk fixture".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(list) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    let ws = list.active.unwrap();
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
    let mut entries = inventory.entries;
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    for entry in &entries {
        fs::write(
            entry.metadata.root.to_path().unwrap().join("file.txt"),
            b"changed\n",
        )
        .unwrap();
    }
    Test {
        f,
        app,
        ws,
        entries,
    }
}
fn trust(t: &mut Test, index: usize) {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewTrust {
                workspace: t.ws.clone(),
                worktrees: vec![t.entries[index].id.0.clone()],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    t.app.confirm(cap, true, Cancellation::default()).unwrap();
}
fn selection(t: &Test, index: usize) -> BatchSelection {
    BatchSelection {
        worktree: t.entries[index].id.0.clone(),
        message: format!("Batch item {index}\n"),
        mode: BatchMode::ReviewAll,
        excluded: Vec::new(),
        hooks: HookPolicy::Execute,
        signing: SigningPreference::Configured,
    }
}
fn review(t: &mut Test, selections: Vec<BatchSelection>) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewBatch {
                workspace: t.ws.clone(),
                selections,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
fn confirm(t: &mut Test, r: &ReviewDisplay) -> BatchView {
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap()
}
fn head(t: &Test, index: usize) -> Vec<u8> {
    t.f.git(&[
        "-C",
        t.entries[index]
            .metadata
            .root
            .to_path()
            .unwrap()
            .to_str()
            .unwrap(),
        "rev-parse",
        "HEAD",
    ])
}
#[test]
fn explicit_batch_commits_only_selected_repositories_and_never_nested_children_implicitly() {
    let mut t = setup(2);
    let parent = t
        .entries
        .iter()
        .position(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&t.f.repo).unwrap())
        .unwrap();
    let child = 1 - parent;
    trust(&mut t, parent);
    let child_before = tree_bytes(&t.entries[child].metadata.root.to_path().unwrap());
    let selections = vec![selection(&t, parent)];
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, selections);
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert!(r.summary.contains("1 explicitly selected"));
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::Complete);
    assert_eq!(result.outcomes.len(), 1);
    assert!(result.outcomes[0].published);
    assert_eq!(t.f.git(&["show", "HEAD:file.txt"]), b"changed\n");
    assert_eq!(
        child_before,
        tree_bytes(&t.entries[child].metadata.root.to_path().unwrap())
    );
}
#[test]
fn batch_stops_after_first_failure_preserves_success_and_cancels_unstarted_items() {
    let mut t = setup(3);
    let second = t.entries[1].metadata.root.to_path().unwrap();
    let hook = second.join(".git/hooks/pre-commit");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(&hook, b"#!/bin/sh\nexit 23\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    }
    for i in 0..3 {
        trust(&mut t, i);
    }
    let before = (0..3).map(|i| head(&t, i)).collect::<Vec<_>>();
    let selections = (0..3).map(|i| selection(&t, i)).collect();
    let r = review(&mut t, selections);
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::PartialFailure);
    assert!(result.outcomes[0].published);
    assert_eq!(result.outcomes[1].phase, Phase::Blocked);
    assert_eq!(result.outcomes[2].phase, Phase::CancelledBeforePublication);
    assert_ne!(head(&t, 0), before[0]);
    assert_eq!(head(&t, 1), before[1]);
    assert_eq!(head(&t, 2), before[2]);
    fs::remove_file(&hook).unwrap();
    trust(&mut t, 1);
    let retry = (1..3).map(|i| selection(&t, i)).collect();
    let r = review(&mut t, retry);
    let completed_first = head(&t, 0);
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::Complete);
    assert_eq!(result.outcomes.len(), 2);
    assert!(result.outcomes.iter().all(|o| o.published));
    assert_eq!(head(&t, 0), completed_first);
}
#[test]
fn known_untrusted_item_blocks_entire_batch_before_any_repository_write() {
    let mut t = setup(2);
    trust(&mut t, 0);
    let selections = (0..2).map(|i| selection(&t, i)).collect();
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, selections);
    assert!(r.summary.contains("1 blocked"));
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::Blocked);
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[test]
fn declined_batch_and_changed_selected_content_never_publish() {
    let mut t = setup(1);
    trust(&mut t, 0);
    let selections = vec![selection(&t, 0)];
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, selections);
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert_eq!(
        t.app
            .confirm(cap, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    let selections = vec![selection(&t, 0)];
    let r = review(&mut t, selections);
    let old_head = head(&t, 0);
    fs::write(t.f.repo.join("file.txt"), b"external newer bytes\n").unwrap();
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::NeedsReauthorization);
    assert_eq!(head(&t, 0), old_head);
    assert_eq!(
        fs::read(t.f.repo.join("file.txt")).unwrap(),
        b"external newer bytes\n"
    );
}
#[test]
fn index_only_batch_preserves_partial_staging_and_exclusions_are_exact() {
    let mut t = setup(1);
    trust(&mut t, 0);
    fs::write(t.f.repo.join("file.txt"), b"staged\n").unwrap();
    t.f.git(&["add", "file.txt"]);
    fs::write(t.f.repo.join("file.txt"), b"working\n").unwrap();
    let mut s = selection(&t, 0);
    s.mode = BatchMode::IndexOnly;
    let r = review(&mut t, vec![s]);
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::Complete);
    assert_eq!(t.f.git(&["show", "HEAD:file.txt"]), b"staged\n");
    assert_eq!(fs::read(t.f.repo.join("file.txt")).unwrap(), b"working\n");
    let mut s = selection(&t, 0);
    s.excluded = vec![NativePath::from_path(Path::new("not-a-change"))];
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, vec![s]);
    assert!(r.content.contains("excluded path"));
    t.app.cancel_review(&r.id).unwrap();
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[test]
fn duplicate_selection_rejected_and_restart_cannot_replay_an_old_batch_capability() {
    let mut t = setup(1);
    trust(&mut t, 0);
    let s = selection(&t, 0);
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewBatch {
                workspace: t.ws.clone(),
                selections: vec![s.clone(), s.clone()]
            }
        )
        .is_err());
    let r = review(&mut t, vec![s]);
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let before = tree_bytes(&t.f.repo);
    let options = Options {
        global_config: Some(t.f.root.join("global.config")),
        signer_home: Some(t.f.root.join("home")),
        ..Options::default()
    };
    let old = std::mem::replace(
        &mut t.app,
        Application::open(&t.f.root.join("other-state"), options.clone()).unwrap(),
    );
    drop(old);
    let reopened = Application::open(&t.f.root.join("state"), options).unwrap();
    t.app = reopened;
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(before, tree_bytes(&t.f.repo));
    let Data::Batches(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].record.phase, BatchPhase::Interrupted);
    assert!(page.items[0].outcomes.iter().all(|o| !o.published));
}

#[test]
fn real_submodule_commits_before_parent_and_changed_gitlink_requires_fresh_parent_review() {
    let mut t = setup(1);
    let source = t.f.root.join("source");
    let source_path = source.to_str().unwrap();
    t.f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "-q",
        "-b",
        "main",
        source_path,
    ]);
    for (key, value) in [
        ("user.name", "Submodule fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        t.f.git(&["-C", source_path, "config", key, value]);
    }
    fs::write(source.join("file.txt"), b"original child\n").unwrap();
    t.f.git(&["-C", source_path, "add", "file.txt"]);
    t.f.git(&[
        "-C",
        source_path,
        "-c",
        "core.hooksPath=",
        "commit",
        "-qm",
        "child baseline",
    ]);
    t.f.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        source_path,
        "modules/lib",
    ]);
    t.f.git(&["commit", "-qm", "Register submodule"]);
    let child = t.f.repo.join("modules/lib");
    let child_path = child.to_str().unwrap();
    for (key, value) in [
        ("user.name", "Submodule fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        t.f.git(&["-C", child_path, "config", key, value]);
    }
    fs::write(child.join("file.txt"), b"new child\n").unwrap();
    t.app
        .request(
            t.app.epoch(),
            Request::Scan {
                workspace: t.ws.clone(),
            },
        )
        .unwrap();
    let Data::Inventory(inventory) = t
        .app
        .request(
            t.app.epoch(),
            Request::Inventory {
                workspace: t.ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    t.entries = inventory.entries;
    t.entries.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(t.entries.len(), 2);
    let parent = t
        .entries
        .iter()
        .position(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&t.f.repo).unwrap())
        .unwrap();
    let child_index = 1 - parent;
    for i in 0..2 {
        trust(&mut t, i);
    }
    let parent_head = head(&t, parent);
    let child_head = head(&t, child_index);
    let index = fs::read(t.f.repo.join(".git/index")).unwrap();
    let selections = (0..2).map(|i| selection(&t, i)).collect();
    let r = review(&mut t, selections);
    let Data::Batches(prepared) = t
        .app
        .request(
            t.app.epoch(),
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        prepared.items[0].record.items[0].worktree,
        t.entries[child_index].id
    );
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::NeedsReauthorization);
    assert!(result.outcomes[0].published);
    assert!(!result.outcomes[1].published);
    assert_ne!(head(&t, child_index), child_head);
    assert_eq!(head(&t, parent), parent_head);
    assert_eq!(fs::read(t.f.repo.join(".git/index")).unwrap(), index);

    let published_child = head(&t, child_index);
    let before_diff = tree_bytes(&t.f.repo);
    t.app
        .request(
            t.app.epoch(),
            Request::Refresh {
                workspace: t.ws.clone(),
                worktree: t.entries[child_index].id.0.clone(),
            },
        )
        .unwrap();
    let Data::Diff(gitlink_diff) = t
        .app
        .request(
            t.app.epoch(),
            Request::Diff {
                workspace: t.ws.clone(),
                worktree: t.entries[parent].id.0.clone(),
                mode: gitfleet_core::diff::DiffMode::Unstaged,
                path: Some(NativePath::from_path(Path::new("modules/lib"))),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(gitlink_diff.files.len(), 1);
    assert_eq!(
        gitlink_diff.files[0].old_oid.as_ref().unwrap().hex,
        String::from_utf8(child_head.clone()).unwrap().trim()
    );
    assert_eq!(
        gitlink_diff.files[0].new_oid.as_ref().unwrap().hex,
        String::from_utf8(published_child.clone()).unwrap().trim()
    );
    let text = String::from_utf8(gitlink_diff.patch).unwrap();
    assert!(text.contains("Observed child HEAD:"));
    assert!(text.contains("Comparison uses the child's last local inspection."));
    assert_eq!(tree_bytes(&t.f.repo), before_diff);
    let mut index_only = selection(&t, parent);
    index_only.mode = BatchMode::IndexOnly;
    let index_review = review(&mut t, vec![index_only]);
    assert!(index_review.summary.contains("1 blocked"));
    let mut excluded = selection(&t, parent);
    excluded.excluded = vec![NativePath::from_path(Path::new("modules/lib"))];
    let excluded_review = review(&mut t, vec![excluded]);
    assert!(excluded_review.summary.contains("0 blocked"));
    assert!(!excluded_review
        .content
        .contains("diff --git a/modules/lib b/modules/lib"));
    let parent_selection = selection(&t, parent);
    let fresh = review(&mut t, vec![parent_selection]);
    assert!(fresh
        .content
        .contains("diff --git a/modules/lib b/modules/lib"));
    let completed = confirm(&mut t, &fresh);
    assert_eq!(completed.record.phase, BatchPhase::Complete);
    assert!(completed.outcomes[0].published);
    assert_eq!(head(&t, child_index), published_child);
    assert_eq!(t.f.git(&["rev-parse", "HEAD:modules/lib"]), published_child);
    assert!(t.f.git(&["status", "--porcelain"]).is_empty());
}

#[test]
fn malformed_submodule_relationships_stop_before_batch_proposals() {
    // Self/ancestor edges cannot form a dependency cycle through normalized child paths.
    let modules = [
        "[submodule \"self\"]\npath = .\n",
        "[submodule \"ancestor\"]\npath = ..\n",
        "[submodule \"escape\"]\npath = ../outside\n",
        "[submodule \"absolute\"]\npath = /outside\n",
        "[submodule \"traversal\"]\npath = child/../outside\n",
        "[submodule \"metadata\"]\npath = .git\n",
        "[submodule \"separator\"]\npath = child//outside\n",
        "[submodule \"malformed\"\npath = child\n",
    ];
    for content in modules {
        let mut t = setup(1);
        fs::write(t.f.repo.join(".gitmodules"), content).unwrap();
        let before = tree_bytes(&t.f.repo);
        let selected = selection(&t, 0);
        let error = t
            .app
            .request(
                t.app.epoch(),
                Request::ReviewBatch {
                    workspace: t.ws.clone(),
                    selections: vec![selected],
                },
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unavailable, "{content}");
        assert_eq!(
            error.to_string(),
            "Complete discovery is required for batch dependency planning"
        );
        for request in [
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
            Request::Operations {
                workspace: t.ws.clone(),
                offset: 0,
            },
        ] {
            match t.app.request(t.app.epoch(), request).unwrap().data {
                Data::Batches(page) => assert!(page.items.is_empty()),
                Data::Operations(page) => assert!(page.items.is_empty()),
                _ => panic!("Unexpected audit response"),
            }
        }
        assert_eq!(tree_bytes(&t.f.repo), before, "{content}");
    }
}

#[test]
fn three_level_submodules_require_fresh_review_at_each_changed_parent() {
    let mut t = setup(1);
    let leaf_source = Fixture::new();
    leaf_source.commit_file();
    let middle_source = Fixture::new();
    middle_source.commit_file();
    middle_source.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        leaf_source.repo.to_str().unwrap(),
        "leaf",
    ]);
    middle_source.git(&["commit", "-qm", "Register leaf"]);
    t.f.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        middle_source.repo.to_str().unwrap(),
        "middle",
    ]);
    t.f.git(&["commit", "-qm", "Register middle"]);
    t.f.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "update",
        "--init",
        "--recursive",
    ]);
    let paths = [
        t.f.repo.clone(),
        t.f.repo.join("middle"),
        t.f.repo.join("middle/leaf"),
    ];
    for path in &paths {
        // Recursive init checks out the leaf detached; controlled commits require a branch.
        t.f.git(&["-C", path.to_str().unwrap(), "checkout", "-q", "main"]);

        for (key, value) in [
            ("user.name", "Three-level fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
        ] {
            t.f.git(&["-C", path.to_str().unwrap(), "config", key, value]);
        }
        fs::write(path.join("file.txt"), b"three-level reviewed content\n").unwrap();
    }
    t.app
        .request(
            t.app.epoch(),
            Request::Scan {
                workspace: t.ws.clone(),
            },
        )
        .unwrap();
    let Data::Inventory(inventory) = t
        .app
        .request(
            t.app.epoch(),
            Request::Inventory {
                workspace: t.ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    t.entries = inventory.entries;
    assert_eq!(t.entries.len(), 3);
    let indices: Vec<_> = paths
        .iter()
        .map(|path| {
            t.entries
                .iter()
                .position(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(path).unwrap())
                .unwrap()
        })
        .collect();
    let (root, middle, leaf) = (indices[0], indices[1], indices[2]);
    for i in 0..3 {
        trust(&mut t, i);
    }
    let original_heads: Vec<_> = (0..3).map(|i| head(&t, i)).collect();
    let before = tree_bytes(&t.f.repo);
    let requested = [root, middle, leaf]
        .into_iter()
        .map(|i| selection(&t, i))
        .collect();
    let first = review(&mut t, requested);
    assert!(first.summary.contains("0 blocked"));
    assert_eq!(
        tree_bytes(&t.f.repo),
        before,
        "Preparation must not stage any worktree"
    );
    let Data::Batches(prepared) = t
        .app
        .request(
            t.app.epoch(),
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let items = &prepared.items[0].record.items;
    assert_eq!(
        items.iter().map(|i| &i.worktree).collect::<Vec<_>>(),
        vec![
            &t.entries[leaf].id,
            &t.entries[middle].id,
            &t.entries[root].id
        ]
    );
    assert!(items[0].dependencies.is_empty());
    assert_eq!(items[1].dependencies, vec![t.entries[leaf].id.clone()]);
    assert_eq!(items[2].dependencies, vec![t.entries[middle].id.clone()]);
    let first_result = confirm(&mut t, &first);
    assert_eq!(first_result.record.phase, BatchPhase::NeedsReauthorization);
    assert!(first_result.outcomes[0].published);
    assert!(first_result.outcomes[1..].iter().all(|o| !o.published));
    assert_ne!(head(&t, leaf), original_heads[leaf]);
    assert_eq!(head(&t, middle), original_heads[middle]);
    assert_eq!(head(&t, root), original_heads[root]);
    let leaf_after = tree_bytes(&paths[2]);
    let leaf_metadata = t.entries[leaf].metadata.git_dir.to_path().unwrap();
    let leaf_metadata_after = tree_bytes(&leaf_metadata);
    let middle_before = tree_bytes(&paths[1]);

    // Fresh middle approval still cannot authorize the root's future gitlink.
    let requested = [root, middle]
        .into_iter()
        .map(|i| selection(&t, i))
        .collect();
    let second = review(&mut t, requested);
    assert!(second.content.contains("diff --git a/leaf b/leaf"));
    assert_eq!(tree_bytes(&paths[1]), middle_before);
    let second_result = confirm(&mut t, &second);
    assert_eq!(second_result.record.phase, BatchPhase::NeedsReauthorization);
    assert!(second_result.outcomes[0].published);
    assert!(second_result.outcomes[1..].iter().all(|o| !o.published));
    assert_eq!(head(&t, root), original_heads[root]);
    assert_eq!(tree_bytes(&paths[2]), leaf_after);
    assert_eq!(tree_bytes(&leaf_metadata), leaf_metadata_after);
    let middle_after = tree_bytes(&paths[1]);
    let middle_metadata = t.entries[middle].metadata.git_dir.to_path().unwrap();
    let middle_metadata_after = tree_bytes(&middle_metadata);
    let root_selection = selection(&t, root);
    let final_review = review(&mut t, vec![root_selection]);
    assert!(final_review
        .content
        .contains("diff --git a/middle b/middle"));
    let final_result = confirm(&mut t, &final_review);
    assert_eq!(final_result.record.phase, BatchPhase::Complete);
    assert!(final_result.outcomes[0].published);
    assert_eq!(tree_bytes(&paths[1]), middle_after);
    assert_eq!(tree_bytes(&middle_metadata), middle_metadata_after);
    for (entry, original_head) in t.entries.iter().zip(&original_heads) {
        let path = entry.metadata.root.to_path().unwrap();
        let old = std::str::from_utf8(original_head).unwrap();
        assert_eq!(
            t.f.git(&[
                "-C",
                path.to_str().unwrap(),
                "rev-list",
                "--count",
                &format!("{}..HEAD", old.trim())
            ]),
            b"1\n"
        );
        assert_eq!(
            t.f.git(&["-C", path.to_str().unwrap(), "show", "HEAD:file.txt"]),
            b"three-level reviewed content\n"
        );
    }
    assert_eq!(t.f.git(&["rev-parse", "HEAD:middle"]), head(&t, middle));
    assert_eq!(
        t.f.git(&["-C", paths[1].to_str().unwrap(), "rev-parse", "HEAD:leaf"]),
        head(&t, leaf)
    );
}

fn slow_hook(t: &Test, index: usize, marker: &Path) {
    let hook = t.entries[index]
        .metadata
        .root
        .to_path()
        .unwrap()
        .join(".git/hooks/pre-commit");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    // Fixture paths are generated without shell metacharacters. Marker stays outside repositories.
    fs::write(
        &hook,
        format!(
            "#!/bin/sh\nprintf started > '{}'\nsleep 60\nexit 23\n",
            marker.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
}
fn wait_for_marker(marker: &Path) -> bool {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while std::time::Instant::now() < until {
        if marker.exists() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    false
}
#[test]
fn running_batch_cancellation_preserves_completed_item_and_never_starts_successors() {
    let mut t = setup(3);
    let marker = t.f.root.join("hook-started");
    slow_hook(&t, 1, &marker);
    for i in 0..3 {
        trust(&mut t, i);
    }
    let before = (0..3).map(|i| head(&t, i)).collect::<Vec<_>>();
    let selection = (0..3).map(|i| selection(&t, i)).collect();
    let r = review(&mut t, selection);
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let token = Cancellation::default();
    let cancel = token.clone();
    let waiter = std::thread::spawn(move || {
        let started = wait_for_marker(&marker);
        cancel.cancel();
        started
    });
    let result: BatchView =
        serde_json::from_value(t.app.confirm(cap, true, token).unwrap()).unwrap();
    assert!(
        waiter.join().unwrap(),
        "Cancellation must follow actual hook execution"
    );
    assert_eq!(result.record.phase, BatchPhase::Cancelled);
    assert!(result.outcomes[0].published);
    assert!(!result.outcomes[1].published);
    assert_eq!(result.outcomes[2].phase, Phase::CancelledBeforePublication);
    assert_ne!(head(&t, 0), before[0]);
    assert_eq!(head(&t, 1), before[1]);
    assert_eq!(head(&t, 2), before[2]);
}

#[test]
#[ignore = "subprocess worker invoked by the batch interruption test"]
fn batch_crash_worker() {
    let root = std::path::PathBuf::from(std::env::var_os("GITFLEET_BATCH_CRASH_ROOT").unwrap());
    let mut app = Application::open(
        &root.join("state"),
        Options {
            global_config: Some(root.join("global.config")),
            signer_home: Some(root.join("home")),
            ..Options::default()
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
    let workspace = workspaces.active.unwrap();
    let Data::Inventory(mut inventory) = app
        .request(
            app.epoch(),
            Request::Inventory {
                workspace: workspace.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    inventory.entries.sort_by(|a, b| a.id.cmp(&b.id));
    let selections = inventory
        .entries
        .into_iter()
        .map(|e| BatchSelection {
            worktree: e.id.0,
            message: "Crash batch\n".into(),
            mode: BatchMode::ReviewAll,
            excluded: vec![],
            hooks: HookPolicy::Execute,
            signing: SigningPreference::Configured,
        })
        .collect();
    let Data::Review(review) = app
        .request(
            app.epoch(),
            Request::ReviewBatch {
                workspace,
                selections,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = app.begin_confirmation(&review.id).unwrap();
    let result = app.confirm(cap, true, Cancellation::default());
    panic!("Worker should have been killed while hook ran: {result:?}");
}

#[test]
fn process_interruption_between_items_is_visible_and_restart_does_not_replay() {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    let mut t = setup(3);
    let marker = t.f.root.join("hook-started");
    slow_hook(&t, 1, &marker);
    for i in 0..3 {
        trust(&mut t, i);
    }
    let before = (0..3).map(|i| head(&t, i)).collect::<Vec<_>>();
    let options = Options {
        global_config: Some(t.f.root.join("global.config")),
        signer_home: Some(t.f.root.join("home")),
        ..Options::default()
    };
    // Release the single state owner before the subprocess acquires it.
    let old = std::mem::replace(
        &mut t.app,
        Application::open(&t.f.root.join("other-state"), options.clone()).unwrap(),
    );
    drop(old);
    let log = fs::File::create(t.f.root.join("worker.txt")).unwrap();
    let mut worker = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "batch_crash_worker", "--nocapture"])
        .env("GITFLEET_BATCH_CRASH_ROOT", &t.f.root)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .process_group(0)
        .spawn()
        .unwrap();
    let started = wait_for_marker(&marker);
    // Only this fresh fixture worker and its descendants belong to this new process group.
    let killed = Command::new("/bin/kill")
        .args(["-KILL", &format!("-{}", worker.id())])
        .status()
        .unwrap();
    let exit = worker.wait().unwrap();
    assert!(killed.success());
    assert!(
        started,
        "Worker never entered the second item's hook: {}",
        fs::read_to_string(t.f.root.join("worker.txt")).unwrap()
    );
    assert!(!exit.success());
    let interrupted = tree_bytes(&t.f.repo);
    t.app = Application::open(&t.f.root.join("state"), options).unwrap();
    let Data::Batches(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items[0].record.phase, BatchPhase::Interrupted);
    assert!(page.items[0].outcomes[0].published);
    assert!(!page.items[0].outcomes[1].published);
    assert!(!page.items[0].outcomes[2].published);
    assert_eq!(interrupted, tree_bytes(&t.f.repo));
    assert_ne!(head(&t, 0), before[0]);
    assert_eq!(head(&t, 1), before[1]);
    assert_eq!(head(&t, 2), before[2]);
}

#[test]
fn selected_linked_worktrees_share_locks_but_commit_only_their_own_branches() {
    let mut t = setup(1);
    let linked = t.f.repo.join("linked");
    t.f.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "linked-branch",
        linked.to_str().unwrap(),
    ]);
    fs::write(linked.join("file.txt"), b"linked change\n").unwrap();
    let Data::Inventory(inventory) = t
        .app
        .request(
            t.app.epoch(),
            Request::Scan {
                workspace: t.ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    t.entries = inventory.entries;
    t.entries.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(t.entries.len(), 2);
    assert_eq!(t.entries[0].repository_id, t.entries[1].repository_id);
    for i in 0..2 {
        trust(&mut t, i);
    }
    let before = (0..2).map(|i| head(&t, i)).collect::<Vec<_>>();
    let selections = (0..2).map(|i| selection(&t, i)).collect();
    let r = review(&mut t, selections);
    let result = confirm(&mut t, &r);
    assert_eq!(result.record.phase, BatchPhase::Complete);
    assert!(result
        .outcomes
        .iter()
        .all(|o| o.published && o.index_reconciled));
    for (i, old) in before.iter().enumerate() {
        assert_ne!(head(&t, i), *old);
        let content = t.f.git(&[
            "-C",
            t.entries[i]
                .metadata
                .root
                .to_path()
                .unwrap()
                .to_str()
                .unwrap(),
            "show",
            "HEAD:file.txt",
        ]);
        assert_eq!(
            content,
            if t.entries[i].metadata.root.to_path().unwrap() == fs::canonicalize(&linked).unwrap() {
                b"linked change\n".as_slice()
            } else {
                b"changed\n".as_slice()
            }
        );
    }
}

#[test]
fn revoked_trust_revokes_the_whole_batch_capability_and_cancels_proposals() {
    let mut t = setup(2);
    for i in 0..2 {
        trust(&mut t, i);
    }
    let selections = (0..2).map(|i| selection(&t, i)).collect();
    let review = review(&mut t, selections);
    let (cap, _) = t.app.begin_confirmation(&review.id).unwrap();
    let before = tree_bytes(&t.f.repo);
    t.app
        .request(
            t.app.epoch(),
            Request::Revoke {
                workspace: t.ws.clone(),
                worktree: t.entries[1].id.0.clone(),
            },
        )
        .unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    let Data::Batches(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::Batches {
                workspace: t.ws.clone(),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items[0].record.phase, BatchPhase::Cancelled);
    assert!(page.items[0]
        .outcomes
        .iter()
        .all(|o| o.phase == Phase::CancelledBeforePublication));
    assert_eq!(tree_bytes(&t.f.repo), before);
}

#[test]
fn linked_worktree_batch_requires_independent_trust_and_preserves_unselected_checkout() {
    let mut t = setup(2);
    let linked = t.f.repo.join("linked");
    t.f.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "bulk-linked",
        linked.to_str().unwrap(),
    ]);
    fs::write(linked.join("file.txt"), b"linked working content\n").unwrap();
    let Data::Inventory(inventory) = t
        .app
        .request(
            t.app.epoch(),
            Request::Scan {
                workspace: t.ws.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    t.entries = inventory.entries;
    let parent_root = fs::canonicalize(&t.f.repo).unwrap();
    let parent = t
        .entries
        .iter()
        .position(|e| e.metadata.root.to_path().unwrap() == parent_root)
        .unwrap();
    let child = t
        .entries
        .iter()
        .position(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&linked).unwrap())
        .unwrap();
    assert_eq!(
        t.entries[parent].repository_id,
        t.entries[child].repository_id
    );
    assert_ne!(t.entries[parent].id, t.entries[child].id);
    trust(&mut t, parent);
    let before = tree_bytes(&t.f.repo);
    let requested = vec![selection(&t, parent), selection(&t, child)];
    let blocked = review(&mut t, requested);
    assert!(blocked.summary.contains("1 blocked"));
    assert_eq!(confirm(&mut t, &blocked).record.phase, BatchPhase::Blocked);
    assert_eq!(before, tree_bytes(&t.f.repo));
    trust(&mut t, child);
    let control = tree_bytes(&t.f.repo.join("child-1"));
    let requested = vec![selection(&t, parent), selection(&t, child)];
    let approved = review(&mut t, requested);
    let result = confirm(&mut t, &approved);
    assert_eq!(result.record.phase, BatchPhase::Complete);
    assert_eq!(result.outcomes.len(), 2);
    assert!(result.outcomes.iter().all(|outcome| outcome.published));
    assert_ne!(head(&t, parent), head(&t, child));
    assert_eq!(t.f.git(&["show", "main:file.txt"]), b"changed\n");
    assert_eq!(
        t.f.git(&["show", "bulk-linked:file.txt"]),
        b"linked working content\n"
    );
    assert_eq!(control, tree_bytes(&t.f.repo.join("child-1")));
}
