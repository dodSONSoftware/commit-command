mod support;
use gitfleet_core::{
    diff::{self, DiffMode},
    domain::{NativePath, ObjectFormat, ObjectId},
    history::{self, HistoryScope},
    inspection::{self, ReadOptions},
    ErrorKind,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use support::{tree_bytes, Fixture};
fn oid(f: &Fixture) -> ObjectId {
    let value = String::from_utf8(f.git(&["rev-parse", "HEAD"])).unwrap();
    ObjectId::parse(
        value.trim(),
        &inspection::repository_metadata(&f.repo)
            .unwrap()
            .object_format,
    )
    .unwrap()
}
fn path(text: &str) -> NativePath {
    NativePath::from_path(Path::new(text))
}
#[test]
fn structured_status_distinguishes_partial_staging_rename_binary_modes_and_conflicts() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("file.txt"), b"staged\n").unwrap();
    f.git(&["add", "file.txt"]);
    fs::write(f.repo.join("file.txt"), b"unstaged\n").unwrap();
    fs::write(f.repo.join("binary.bin"), [0, 255, 10]).unwrap();
    let status = inspection::snapshot(&f.repo).unwrap();
    assert_eq!(status.counts.staged, 1);
    assert_eq!(status.counts.unstaged, 1);
    assert_eq!(status.counts.untracked, 1);
    assert_eq!(
        status
            .changes
            .iter()
            .find(|c| c.path == path("file.txt"))
            .map(|c| (c.index, c.worktree)),
        Some(('M', 'M'))
    );
    assert!(status.last_commit.is_some());
    f.git(&["reset", "--hard", "-q"]);
    fs::rename(f.repo.join("file.txt"), f.repo.join("new name.txt")).unwrap();
    f.git(&["add", "-A"]);
    let status = inspection::snapshot(&f.repo).unwrap();
    let rename = status
        .changes
        .iter()
        .find(|c| c.original_path.is_some())
        .unwrap();
    assert_eq!(rename.original_path, Some(path("file.txt")));
    assert_eq!(rename.path, path("new name.txt"));
}
#[test]
fn local_upstream_comparison_never_claims_remote_freshness() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    f.git(&[
        "config",
        "remote.origin.url",
        "https://must-not-contact.invalid/repo",
    ]);
    f.git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    f.git(&["config", "branch.main.remote", "origin"]);
    f.git(&["config", "branch.main.merge", "refs/heads/main"]);
    fs::write(f.repo.join("file.txt"), b"next\n").unwrap();
    f.git(&["commit", "-qam", "ahead"]);
    let before = tree_bytes(&f.root);
    let tracking = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert_eq!(tracking.ahead, Some(1));
    assert_eq!(tracking.behind, Some(0));
    assert!(tracking.fetch_freshness.is_none());
    assert_eq!(tracking.comparison, "local_tracking_refs_only");
    assert_eq!(before, tree_bytes(&f.root));
}
#[test]
fn history_dag_merge_parents_tags_pagination_and_stale_cursors() {
    let f = Fixture::new();
    f.commit_file();
    let root = oid(&f);
    f.git(&["checkout", "-qb", "feature"]);
    fs::write(f.repo.join("feature"), b"feature\n").unwrap();
    f.git(&["add", "feature"]);
    f.git(&["commit", "-qm", "feature"]);
    let feature = oid(&f);
    f.git(&["checkout", "-q", "main"]);
    fs::write(f.repo.join("main"), b"main\n").unwrap();
    f.git(&["add", "main"]);
    f.git(&["commit", "-qm", "main"]);
    let main = oid(&f);
    f.git(&["merge", "--no-ff", "-qm", "merge", "feature"]);
    let merge = oid(&f);
    f.git(&["tag", "-a", "v1", "-m", "tag"]);
    let before = tree_bytes(&f.root);
    let page = history::page(&f.repo, HistoryScope::LocalBranches, 1, None).unwrap();
    assert_eq!(page.commits[0].oid, merge);
    assert_eq!(page.commits[0].parents, vec![main.clone(), feature.clone()]);
    assert!(page
        .references
        .iter()
        .any(|r| r.name == "refs/tags/v1" && r.peeled.as_ref() == Some(&merge)));
    let next = history::page(
        &f.repo,
        HistoryScope::LocalBranches,
        100,
        page.next.as_ref(),
    )
    .unwrap();
    let mut ids = next
        .commits
        .iter()
        .map(|c| c.oid.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&root));
    ids.push(merge.clone());
    assert_eq!(
        ids.iter()
            .map(|id| &id.hex)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(before, tree_bytes(&f.root));
    let diff = diff::inspect(
        &f.repo,
        DiffMode::Historical {
            commit: merge.clone(),
            parent: Some(main),
        },
        None,
    )
    .unwrap();
    assert!(diff.files.iter().any(|c| c.path == path("feature")));
    assert_eq!(
        diff::inspect(
            &f.repo,
            DiffMode::Historical {
                commit: merge,
                parent: None
            },
            None
        )
        .unwrap_err()
        .kind,
        ErrorKind::InvalidRequest
    );
    f.git(&["branch", "new"]);
    assert_eq!(
        history::page(&f.repo, HistoryScope::LocalBranches, 1, page.next.as_ref())
            .unwrap_err()
            .kind,
        ErrorKind::StaleCursor
    );
}
#[test]
fn root_commit_diff_and_staged_unstaged_blobs_have_independent_oracles() {
    let f = Fixture::new();
    f.commit_file();
    let root = oid(&f);
    let before = tree_bytes(&f.root);
    let root_diff = diff::inspect(
        &f.repo,
        DiffMode::Historical {
            commit: root,
            parent: None,
        },
        None,
    )
    .unwrap();
    assert_eq!(root_diff.files.len(), 1);
    assert_eq!(root_diff.files[0].change, "A");
    assert_eq!(
        diff::blob(&f.repo, root_diff.files[0].new_oid.as_ref().unwrap()).unwrap(),
        b"original\n"
    );
    assert_eq!(before, tree_bytes(&f.root));
    fs::write(f.repo.join("file.txt"), b"staged\n").unwrap();
    f.git(&["add", "file.txt"]);
    fs::write(f.repo.join("file.txt"), b"unstaged\n").unwrap();
    let before = tree_bytes(&f.root);
    let staged = diff::inspect(&f.repo, DiffMode::Staged, Some(&path("file.txt"))).unwrap();
    let unstaged = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    assert_eq!(
        diff::blob(&f.repo, staged.files[0].new_oid.as_ref().unwrap()).unwrap(),
        b"staged\n"
    );
    assert!(String::from_utf8(staged.patch).unwrap().contains("+staged"));
    assert!(String::from_utf8(unstaged.patch)
        .unwrap()
        .contains("+unstaged"));
    assert_eq!(
        diff::preview(&f.repo, &path("file.txt")).unwrap().bytes,
        b"unstaged\n"
    );
    assert_eq!(before, tree_bytes(&f.root));
}
#[test]
fn literal_newline_and_binary_previews_preserve_scope_and_bytes() {
    let f = Fixture::new();
    f.commit_file();
    for name in [":(glob)*.txt", "*.txt", "line\nbreak"] {
        fs::write(f.repo.join(name), b"literal\n").unwrap();
    }
    f.git(&["add", "-A"]);
    fs::write(f.repo.join("file.txt"), b"excluded\n").unwrap();
    fs::write(f.repo.join("binary.bin"), [0, 255, 10]).unwrap();
    let before = tree_bytes(&f.root);
    for name in [":(glob)*.txt", "*.txt", "line\nbreak"] {
        let diff = diff::inspect(&f.repo, DiffMode::Staged, Some(&path(name))).unwrap();
        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, path(name));
    }
    let binary = diff::preview(&f.repo, &path("binary.bin")).unwrap();
    assert!(binary.binary);
    assert_eq!(binary.bytes, [0, 255, 10]);
    assert_eq!(before, tree_bytes(&f.root));
}
#[cfg(unix)]
#[test]
fn helper_sentinels_are_suppressed_for_every_history_diff_and_failure_path() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.commit_file();
    let commit = oid(&f);
    let helper = f.root.join("helper");
    fs::write(&helper, b"#!/bin/sh\nprintf ran > helper-ran\nexit 1\n").unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    for key in [
        "core.fsmonitor",
        "diff.external",
        "diff.sentinel.command",
        "diff.sentinel.textconv",
        "gpg.program",
        "core.sshCommand",
        "credential.helper",
    ] {
        f.git(&["config", key, helper.to_str().unwrap()]);
    }
    fs::write(f.repo.join(".gitattributes"), b"file.txt diff=sentinel\n").unwrap();
    fs::write(f.repo.join("file.txt"), b"changed\n").unwrap();
    let before = tree_bytes(&f.root);
    history::page(&f.repo, HistoryScope::LocalBranches, 10, None).unwrap();
    history::commit(&f.repo, &commit).unwrap();
    diff::inspect(&f.repo, DiffMode::Staged, None).unwrap();
    diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    diff::inspect(
        &f.repo,
        DiffMode::Historical {
            commit: commit.clone(),
            parent: None,
        },
        None,
    )
    .unwrap();
    diff::preview(&f.repo, &path("file.txt")).unwrap();
    let missing = ObjectId::parse(&"1".repeat(40), &ObjectFormat::Sha1).unwrap();
    assert!(history::commit(&f.repo, &missing).is_err());
    assert!(diff::blob(&f.repo, &missing).is_err());
    assert_eq!(before, tree_bytes(&f.root));
    for verb in ["history", "status", "scan"] {
        let output = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .arg(verb)
            .arg(&f.repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(before, tree_bytes(&f.root));
}
#[cfg(unix)]
#[test]
fn symlink_leaf_is_not_dereferenced_and_parent_escape_and_fifo_are_rejected() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    f.commit_file();
    let outside = f.root.join("secret");
    fs::write(&outside, b"not-previewed").unwrap();
    symlink(&outside, f.repo.join("leaf")).unwrap();
    symlink(&f.root, f.repo.join("escape")).unwrap();
    let before = tree_bytes(&f.root);
    let preview = diff::preview(&f.repo, &path("leaf")).unwrap();
    assert_eq!(preview.kind, "symlink");
    assert_ne!(preview.bytes, b"not-previewed");
    assert_eq!(
        diff::preview(&f.repo, &path("escape/secret"))
            .unwrap_err()
            .kind,
        ErrorKind::InvalidPath
    );
    assert!(diff::preview(&f.repo, &path("../secret")).is_err());
    assert_eq!(before, tree_bytes(&f.root));
    let fifo = f.repo.join("fifo");
    let name = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert_eq!(
        diff::preview(&f.repo, &path("fifo")).unwrap_err().kind,
        ErrorKind::InvalidPath
    );
    fs::remove_file(fifo).unwrap();
}
#[test]
fn sha256_repository_inspection_and_blob_ids_are_format_aware() {
    let f = Fixture::new();
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
    f.git(&["config", "user.name", "Fixture"]);
    f.git(&["config", "user.email", "fixture@example.invalid"]);
    f.commit_file();
    let before = tree_bytes(&f.root);
    let status = inspection::snapshot(&f.repo).unwrap();
    assert_eq!(
        status.head_oid.as_ref().unwrap().format,
        ObjectFormat::Sha256
    );
    assert_eq!(status.head.as_ref().unwrap().len(), 64);
    let commit = history::page(&f.repo, HistoryScope::LocalBranches, 10, None)
        .unwrap()
        .commits
        .remove(0);
    let diff = diff::inspect(
        &f.repo,
        DiffMode::Historical {
            commit: commit.oid,
            parent: None,
        },
        None,
    )
    .unwrap();
    assert_eq!(
        diff::blob(&f.repo, diff.files[0].new_oid.as_ref().unwrap()).unwrap(),
        b"original\n"
    );
    assert_eq!(before, tree_bytes(&f.root));
}
#[test]
fn split_index_is_read_only_and_sparse_configuration_fails_closed() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["update-index", "--split-index"]);
    let before = tree_bytes(&f.root);
    inspection::snapshot(&f.repo).unwrap();
    diff::inspect(&f.repo, DiffMode::Staged, None).unwrap();
    assert_eq!(before, tree_bytes(&f.root));
    f.git(&["config", "index.sparse", "true"]);
    let before = tree_bytes(&f.root);
    assert_eq!(
        inspection::snapshot(&f.repo).unwrap_err().kind,
        ErrorKind::UnsupportedConfiguration
    );
    assert_eq!(before, tree_bytes(&f.root));
}
#[test]
fn cancelled_deadline_output_and_preview_limits_are_typed_without_repo_mutation() {
    let f = Fixture::new();
    f.commit_file();
    let before = tree_bytes(&f.root);
    let mut options = ReadOptions::default();
    options.cancellation.cancel();
    assert_eq!(
        inspection::snapshot_with_options(&f.repo, &options)
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    options = ReadOptions {
        timeout: Duration::ZERO,
        ..ReadOptions::default()
    };
    assert_eq!(
        inspection::snapshot_with_options(&f.repo, &options)
            .unwrap_err()
            .kind,
        ErrorKind::Timeout
    );
    options = ReadOptions {
        max_output: 8,
        ..ReadOptions::default()
    };
    assert_eq!(
        inspection::snapshot_with_options(&f.repo, &options)
            .unwrap_err()
            .kind,
        ErrorKind::OutputLimit
    );
    assert_eq!(before, tree_bytes(&f.root));
    fs::write(f.repo.join("large"), vec![b'x'; 2 * 1024 * 1024 + 1]).unwrap();
    assert_eq!(
        diff::preview(&f.repo, &path("large")).unwrap_err().kind,
        ErrorKind::OutputLimit
    );
}
#[test]
fn byte_safe_parsers_roundtrip_non_utf8_names_and_reject_malformed_records() {
    let record = b"? invalid-\xff\n*\0";
    let (_, changes) = inspection::parse_status(record).unwrap();
    let encoded = serde_json::to_string(&changes[0].path).unwrap();
    let decoded: NativePath = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, changes[0].path);
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        assert_eq!(
            decoded.to_path().unwrap().as_os_str().as_bytes(),
            b"invalid-\xff\n*"
        );
    }
    assert!(inspection::parse_status(b"2 malformed\0").is_err());
    assert!(ObjectId::parse("HEAD", &ObjectFormat::Sha1).is_err());
}
#[test]
fn unborn_history_is_empty_and_history_limits_are_checked() {
    let f = Fixture::new();
    assert!(
        history::page(&f.repo, HistoryScope::LocalBranches, 100, None)
            .unwrap()
            .commits
            .is_empty()
    );
    assert!(history::page(&f.repo, HistoryScope::LocalBranches, 501, None).is_err());
    assert_eq!(
        diff::inspect(&f.repo, DiffMode::Staged, None)
            .unwrap()
            .files
            .len(),
        0
    );
}

#[test]
fn conflict_stages_and_merge_state_are_structured_without_mutation() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["checkout", "-qb", "other"]);
    fs::write(f.repo.join("file.txt"), b"other\n").unwrap();
    f.git(&["commit", "-qam", "other"]);
    f.git(&["checkout", "-q", "main"]);
    fs::write(f.repo.join("file.txt"), b"main\n").unwrap();
    f.git(&["commit", "-qam", "main"]);
    assert!(!f.git_output(&["merge", "other"]).status.success());
    let before = tree_bytes(&f.root);
    let status = inspection::snapshot(&f.repo).unwrap();
    assert_eq!(status.counts.conflicted, 1);
    assert!(status.changes[0].conflict);
    assert!(status.operations.iter().any(|op| op == "merge"));
    assert_eq!(before, tree_bytes(&f.root));
}
#[cfg(unix)]
#[test]
fn executable_mode_and_deletion_diffs_are_explicit() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.commit_file();
    fs::set_permissions(f.repo.join("file.txt"), fs::Permissions::from_mode(0o755)).unwrap();
    let before = tree_bytes(&f.root);
    let changed = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    assert_eq!(changed.files[0].old_mode, "100644");
    assert_eq!(changed.files[0].new_mode, "100755");
    assert_eq!(before, tree_bytes(&f.root));
    fs::remove_file(f.repo.join("file.txt")).unwrap();
    let before = tree_bytes(&f.root);
    let changed = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    assert_eq!(changed.files[0].change, "D");
    assert_eq!(before, tree_bytes(&f.root));
}
#[test]
fn missing_upstream_and_shallow_boundary_are_explicit() {
    let f = Fixture::new();
    f.commit_file();
    let first = oid(&f);
    fs::write(f.repo.join("file.txt"), b"next\n").unwrap();
    f.git(&["commit", "-qam", "next"]);
    let second = oid(&f);
    fs::write(f.repo.join(".git/shallow"), format!("{}\n", second.hex)).unwrap();
    let before = tree_bytes(&f.root);
    let page = history::page(&f.repo, HistoryScope::LocalBranches, 10, None).unwrap();
    assert!(page.shallow);
    assert_eq!(page.shallow_boundaries, vec![second]);
    assert_eq!(page.commits.len(), 1);
    assert_eq!(page.commits[0].parents, vec![first]);
    assert_eq!(before, tree_bytes(&f.root));
    f.git(&[
        "config",
        "remote.origin.url",
        "https://must-not-contact.invalid/repo",
    ]);
    f.git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    f.git(&["config", "branch.main.remote", "origin"]);
    f.git(&["config", "branch.main.merge", "refs/heads/main"]);
    let tracking = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert!(tracking.oid.is_none());
    assert!(tracking.ahead.is_none());
    assert!(tracking.behind.is_none());
}

#[test]
fn cli_and_library_inspection_contracts_agree_for_all_read_modes() {
    fn normalize(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("observed_at_ms");
                for value in map.values_mut() {
                    normalize(value);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    normalize(value);
                }
            }
            _ => {}
        }
    }
    let f = Fixture::new();
    f.commit_file();
    let commit = oid(&f);
    fs::write(f.repo.join("file.txt"), b"staged\n").unwrap();
    f.git(&["add", "file.txt"]);
    fs::write(f.repo.join("file.txt"), b"unstaged\n").unwrap();
    let before = tree_bytes(&f.root);
    let expected = vec![
        (
            vec!["status".to_owned()],
            serde_json::to_value(inspection::snapshot(&f.repo).unwrap()).unwrap(),
        ),
        (
            vec!["history".to_owned()],
            serde_json::to_value(
                history::page(&f.repo, HistoryScope::LocalBranches, 100, None).unwrap(),
            )
            .unwrap(),
        ),
        (
            vec!["commit".to_owned(), commit.hex.clone()],
            serde_json::to_value(history::commit(&f.repo, &commit).unwrap()).unwrap(),
        ),
        (
            vec!["diff".to_owned(), "staged".into()],
            serde_json::to_value(diff::inspect(&f.repo, DiffMode::Staged, None).unwrap()).unwrap(),
        ),
        (
            vec!["diff".to_owned(), "unstaged".into()],
            serde_json::to_value(diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap())
                .unwrap(),
        ),
        (
            vec!["preview".to_owned(), "file.txt".into()],
            serde_json::to_value(diff::preview(&f.repo, &path("file.txt")).unwrap()).unwrap(),
        ),
    ];
    for (arguments, mut expected) in expected {
        let output = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .arg(&arguments[0])
            .arg(&f.repo)
            .args(&arguments[1..])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        normalize(&mut actual);
        normalize(&mut expected);
        assert_eq!(actual, expected);
    }
    assert_eq!(before, tree_bytes(&f.root));
}

#[cfg(unix)]
#[test]
fn clean_unstaged_diff_preserves_an_index_with_unpopulated_stat_cache() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("tab\tfile.txt"), b"baseline\n").unwrap();
    f.git(&["add", "--", "tab\tfile.txt"]);
    f.git(&["commit", "-qm", "Second tracked file"]);
    f.git(&["config", "diff.autoRefreshIndex", "true"]);
    // A reconciled index may have correct tree entries without working-file stat data.
    f.git(&["read-tree", "HEAD"]);
    let index = f.repo.join(".git/index");
    fs::set_permissions(&index, fs::Permissions::from_mode(0o600)).unwrap();
    let before = tree_bytes(&f.repo);
    let result = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    assert!(result.files.is_empty());
    assert!(result.patch.is_empty());
    assert_eq!(
        before,
        tree_bytes(&f.repo),
        "read-only diff refreshed the index"
    );
    fs::write(f.repo.join("tab\tfile.txt"), b"real content change\n").unwrap();
    let before = tree_bytes(&f.repo);
    let result = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[0].path, path("tab\tfile.txt"));
    assert!(String::from_utf8_lossy(&result.patch).contains("+real content change"));
    assert_eq!(before, tree_bytes(&f.repo));
}
