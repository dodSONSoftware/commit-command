mod support;

use gitfleet_core::{discovery, inspection, ErrorKind};
use std::fs;
use std::process::Command;
use support::{tree_bytes, Fixture};

fn assert_unchanged(f: &Fixture) {
    let before = tree_bytes(&f.root);
    inspection::snapshot(&f.repo).unwrap();
    discovery::discover(&f.repo).unwrap();
    for verb in ["status", "discover"] {
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
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
    assert_eq!(
        before,
        tree_bytes(&f.root),
        "inspection changed fixture bytes, modes or entries"
    );
}

#[test]
fn inspection_and_cli_preserve_dirty_repository_bytes() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("file.txt"), b"staged\n").unwrap();
    f.git(&["add", "--", "file.txt"]);
    fs::write(f.repo.join("file.txt"), b"staged\nunstaged\n").unwrap();
    fs::write(f.repo.join("untracked.bin"), [0, 255, 10]).unwrap();
    let state = inspection::snapshot(&f.repo).unwrap();
    assert!(state.status.contains("MM file.txt"));
    assert!(state.status.contains("?? untracked.bin"));
    assert_unchanged(&f);
}

#[cfg(unix)]
#[test]
fn configured_helper_sentinels_are_not_executed() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.commit_file();
    let helper = f.root.join("helper");
    fs::write(&helper, b"#!/bin/sh\nprintf ran > helper-ran\nexit 1\n").unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    let hooks = f.root.join("hooks");
    fs::create_dir(&hooks).unwrap();
    for name in [
        "pre-commit",
        "post-index-change",
        "reference-transaction",
        "fsmonitor-watchman",
    ] {
        fs::copy(&helper, hooks.join(name)).unwrap();
    }
    for key in [
        "core.fsmonitor",
        "diff.external",
        "diff.sentinel.command",
        "diff.sentinel.textconv",
        "core.sshCommand",
        "credential.helper",
        "gpg.program",
    ] {
        f.git(&["config", "--local", key, helper.to_str().unwrap()]);
    }
    f.git(&[
        "config",
        "--local",
        "core.hooksPath",
        hooks.to_str().unwrap(),
    ]);
    f.git(&["config", "--local", "core.untrackedCache", "true"]);
    fs::write(f.repo.join(".gitattributes"), "file.txt diff=sentinel\n").unwrap();
    fs::write(f.repo.join("file.txt"), b"changed\n").unwrap();
    assert_unchanged(&f);
    assert!(!f.repo.join("helper-ran").exists());
}

#[test]
fn content_filters_are_rejected_without_mutation() {
    for key in [
        "filter.sentinel.clean",
        "filter.sentinel.process",
        "filter.sentinel.smudge",
    ] {
        let f = Fixture::new();
        f.commit_file();
        f.git(&["config", "--local", key, "echo ran > helper-ran"]);
        fs::write(f.repo.join(".gitattributes"), "file.txt filter=sentinel\n").unwrap();
        fs::write(f.repo.join("file.txt"), b"changed\n").unwrap();
        let before = tree_bytes(&f.root);
        let error = inspection::snapshot(&f.repo).unwrap_err();
        assert_eq!(error.kind, ErrorKind::UnsupportedConfiguration);
        assert!(discovery::discover(&f.repo).is_err());
        let output = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .arg("status")
            .arg(&f.repo)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(before, tree_bytes(&f.root));
    }
}

#[test]
fn included_filter_configuration_is_rejected() {
    let f = Fixture::new();
    f.commit_file();
    let included = f.root.join("included.config");
    fs::write(
        &included,
        "[filter \"hidden\"]\nclean = echo ran > helper-ran\n",
    )
    .unwrap();
    f.git(&[
        "config",
        "--local",
        "include.path",
        included.to_str().unwrap(),
    ]);
    let before = tree_bytes(&f.root);
    assert_eq!(
        inspection::snapshot(&f.repo).unwrap_err().kind,
        ErrorKind::UnsupportedConfiguration
    );
    assert_eq!(before, tree_bytes(&f.root));
}

#[test]
fn worktree_filter_configuration_is_rejected() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["config", "extensions.worktreeConfig", "true"]);
    f.git(&[
        "config",
        "--worktree",
        "filter.hidden.clean",
        "echo ran > helper-ran",
    ]);
    let before = tree_bytes(&f.root);
    assert_eq!(
        inspection::snapshot(&f.repo).unwrap_err().kind,
        ErrorKind::UnsupportedConfiguration
    );
    assert_eq!(before, tree_bytes(&f.root));
}

#[test]
fn unborn_and_detached_head_can_be_inspected() {
    let f = Fixture::new();
    let state = inspection::snapshot(&f.repo).unwrap();
    assert!(state.head.is_none());
    assert_eq!(state.branch.as_deref(), Some("main"));
    assert_unchanged(&f);
    f.commit_file();
    f.git(&["checkout", "--detach", "-q"]);
    assert!(inspection::snapshot(&f.repo).unwrap().branch.is_none());
    assert_unchanged(&f);
}

#[test]
fn unmerged_index_can_be_inspected_without_write_tree() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["checkout", "-qb", "other"]);
    fs::write(f.repo.join("file.txt"), b"other\n").unwrap();
    f.git(&["commit", "-qam", "other"]);
    f.git(&["checkout", "-q", "main"]);
    fs::write(f.repo.join("file.txt"), b"main\n").unwrap();
    f.git(&["commit", "-qam", "main"]);
    assert!(!f.git_output(&["merge", "other"]).status.success());
    assert!(inspection::snapshot(&f.repo)
        .unwrap()
        .status
        .contains("UU file.txt"));
    assert_unchanged(&f);
}

#[test]
fn nested_directory_is_not_accepted_as_worktree_root() {
    let f = Fixture::new();
    f.commit_file();
    let nested = f.repo.join("nested");
    fs::create_dir(&nested).unwrap();
    assert_eq!(
        inspection::snapshot(&nested).unwrap_err().kind,
        ErrorKind::InvalidPath
    );
}

#[cfg(unix)]
#[test]
fn newline_filename_remains_git_quoted_without_mutation() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("line\nbreak"), b"newline\n").unwrap();
    let state = inspection::snapshot(&f.repo).unwrap();
    assert!(state.status.contains(r"line\nbreak"));
    assert_unchanged(&f);
}

#[cfg(unix)]
#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "M0 host filesystem rejects non-UTF-8 filename creation (EPERM); requires a supporting filesystem"
)]
fn non_utf8_filename_remains_git_quoted_without_mutation() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join(OsStr::from_bytes(b"non-utf8-\xff")), b"bytes\n").unwrap();
    assert!(inspection::snapshot(&f.repo)
        .unwrap()
        .status
        .contains(r"non-utf8-\377"));
    assert_unchanged(&f);
}

#[test]
fn cli_does_not_inherit_git_repository_or_config_overrides() {
    let f = Fixture::new();
    f.commit_file();
    let other = Fixture::new();
    other.commit_file();
    fs::write(f.repo.join("chosen.txt"), b"chosen\n").unwrap();
    let before = tree_bytes(&f.root);
    let other_before = tree_bytes(&other.root);
    let output = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
        .arg("status")
        .arg(&f.repo)
        .env("GIT_DIR", other.repo.join(".git"))
        .env("GIT_WORK_TREE", &other.repo)
        .env("GIT_INDEX_FILE", other.repo.join(".git/index"))
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "filter.injected.clean")
        .env("GIT_CONFIG_VALUE_0", "echo ran > helper-ran")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value["status"].as_str().unwrap().contains("?? chosen.txt"));
    assert_eq!(before, tree_bytes(&f.root));
    assert_eq!(other_before, tree_bytes(&other.root));
}

#[cfg(unix)]
#[test]
fn missing_promisor_object_cannot_launch_a_remote_helper() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    f.commit_file();
    let helper = f.root.join("remote-helper");
    fs::write(&helper, b"#!/bin/sh\nprintf ran > helper-ran\nexit 1\n").unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    f.git(&["config", "core.repositoryformatversion", "1"]);
    f.git(&["config", "extensions.partialClone", "origin"]);
    f.git(&["config", "remote.origin.promisor", "true"]);
    f.git(&[
        "config",
        "remote.origin.url",
        &format!("ext::{}", helper.display()),
    ]);
    f.git(&["config", "protocol.ext.allow", "always"]);
    let tree = String::from_utf8(f.git(&["rev-parse", "HEAD^{tree}"])).unwrap();
    let tree = tree.trim();
    fs::remove_file(
        f.repo
            .join(".git/objects")
            .join(&tree[..2])
            .join(&tree[2..]),
    )
    .unwrap();
    let before = tree_bytes(&f.root);
    assert_eq!(
        inspection::snapshot(&f.repo).unwrap_err().kind,
        ErrorKind::InspectionFailed
    );
    assert_eq!(before, tree_bytes(&f.root));
}

#[test]
fn remote_presence_and_tracking_counts_are_local_and_backward_compatible() {
    let f = Fixture::new();
    f.commit_file();
    let initial = inspection::snapshot(&f.repo).unwrap();
    assert_eq!(initial.remotes, Some(vec![]));
    let mut legacy = serde_json::to_value(&initial).unwrap();
    legacy.as_object_mut().unwrap().remove("remotes");
    let restored: inspection::Snapshot = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.remotes, None);

    // An unreachable URL demonstrates that inspection needs no remote connection.
    f.git(&[
        "remote",
        "add",
        "origin",
        "https://example.invalid/unreachable.git",
    ]);
    let state = inspection::snapshot(&f.repo).unwrap();
    assert_eq!(state.remotes, Some(vec!["origin".into()]));
    assert!(state.tracking.is_none());
    f.git(&["config", "branch.main.remote", "origin"]);
    f.git(&["config", "branch.main.merge", "refs/heads/main"]);
    let missing = inspection::snapshot(&f.repo).unwrap();
    assert!(missing.tracking.unwrap().behind.is_none());
    f.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let equal = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert_eq!((equal.ahead, equal.behind), (Some(0), Some(0)));
    f.git(&["commit", "--allow-empty", "-qm", "local"]);
    let ahead = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert_eq!((ahead.ahead, ahead.behind), (Some(1), Some(0)));
    f.git(&["checkout", "-qb", "server", "HEAD~1"]);
    f.git(&["commit", "--allow-empty", "-qm", "remote"]);
    f.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    f.git(&["checkout", "-q", "main"]);
    let diverged = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert_eq!((diverged.ahead, diverged.behind), (Some(1), Some(1)));
    f.git(&["update-ref", "refs/heads/main", "HEAD~1"]);
    let behind = inspection::snapshot(&f.repo).unwrap().tracking.unwrap();
    assert_eq!((behind.ahead, behind.behind), (Some(0), Some(1)));
    assert_unchanged(&f);
}
