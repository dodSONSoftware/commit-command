mod support;
use gitfleet_core::{
    discovery::{self, DiscoveryOptions, RelationshipKind},
    domain::WorkspaceId,
    workspace::{Availability, WorkspaceService},
    ErrorKind,
};
use std::fs;
use std::process::Command;
use support::{tree_bytes, Fixture};

#[test]
fn saved_workspaces_restart_activate_rename_update_remove_and_lock() {
    let f = Fixture::new();
    f.commit_file();
    let state = f.root.join("state");
    let mut service = WorkspaceService::open(&state).unwrap();
    let first = service.create("First", &f.repo).unwrap();
    let second = service.create("Second", &f.repo).unwrap();
    service.activate(&first.id).unwrap();
    service.activate(&second.id).unwrap();
    assert_eq!(service.active().unwrap(), Some(second.id.clone()));
    assert_eq!(
        WorkspaceService::open(&state).err().unwrap().kind,
        ErrorKind::Busy
    );
    service
        .update(
            &first.id,
            Some("Renamed"),
            None,
            Some(DiscoveryOptions {
                excluded_names: vec!["generated".into()],
                excluded_patterns: vec!["scratch/*".into()],
            }),
        )
        .unwrap();
    drop(service);
    let mut service = WorkspaceService::open(&state).unwrap();
    assert_eq!(service.list().len(), 2);
    assert_eq!(service.list()[0].name, "Renamed");
    assert_eq!(service.active().unwrap(), Some(second.id.clone()));
    service.remove(&second.id).unwrap();
    assert!(service.active().unwrap().is_none());
    drop(service);
    assert_eq!(WorkspaceService::open(&state).unwrap().list().len(), 1);
}

#[test]
fn identities_survive_restart_and_verified_move_but_not_copies_or_replacement() {
    let f = Fixture::new();
    f.commit_file();
    let state = f.root.join("state");
    let mut service = WorkspaceService::open(&state).unwrap();
    let workspace = service.create("Project", &f.repo).unwrap();
    let before = service.scan(&workspace.id).unwrap().entries.remove(0);
    drop(service);
    let mut service = WorkspaceService::open(&state).unwrap();
    assert_eq!(
        service.scan(&workspace.id).unwrap().entries[0].id,
        before.id
    );
    let moved = f.root.join("moved");
    fs::rename(&f.repo, &moved).unwrap();
    service
        .update(&workspace.id, None, Some(&moved), None)
        .unwrap();
    let after = service.scan(&workspace.id).unwrap().entries;
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, before.id);
    assert!(after[0].untrusted);
    let backup = f.root.join("old-git");
    fs::rename(moved.join(".git"), &backup).unwrap();
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    let output = command
        .current_dir(&moved)
        .args(["-c", "init.templateDir=", "init", "-q"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = service.scan(&workspace.id).unwrap();
    assert_eq!(report.entries.len(), 2);
    let replacement = report
        .entries
        .iter()
        .find(|entry| matches!(entry.availability, Availability::Available))
        .unwrap();
    assert_ne!(replacement.id, before.id);
    assert_ne!(replacement.repository_id, before.repository_id);
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.id == before.id && matches!(entry.availability, Availability::Missing)));
}

#[test]
fn clones_with_identical_head_have_distinct_identities() {
    let f = Fixture::new();
    f.commit_file();
    let clone = f.root.join("copy");
    f.git(&[
        "clone",
        "-q",
        "--no-hardlinks",
        f.repo.to_str().unwrap(),
        clone.to_str().unwrap(),
    ]);
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let one = service.create("one", &f.repo).unwrap();
    let two = service.create("two", &clone).unwrap();
    let a = service.scan(&one.id).unwrap().entries.remove(0);
    let b = service.scan(&two.id).unwrap().entries.remove(0);
    assert_eq!(
        a.status.as_ref().unwrap().head,
        b.status.as_ref().unwrap().head
    );
    assert_ne!(a.id, b.id);
    assert_ne!(a.repository_id, b.repository_id);
}

#[test]
fn mixed_inventory_classifies_nested_submodule_and_linked_worktrees() {
    let f = Fixture::new();
    f.commit_file();
    let source = Fixture::new();
    source.commit_file();
    let nested = f.repo.join("nested");
    fs::create_dir(&nested).unwrap();
    f.git(&[
        "-C",
        nested.to_str().unwrap(),
        "-c",
        "init.templateDir=",
        "init",
        "-q",
    ]);
    f.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        source.repo.to_str().unwrap(),
        "module",
    ]);
    let linked = f.repo.join("linked");
    f.git(&["worktree", "add", "--detach", linked.to_str().unwrap()]);
    let external = f.root.join("external");
    f.git(&["worktree", "add", "--detach", external.to_str().unwrap()]);
    let before = tree_bytes(&f.root);
    let report = discovery::scan(&f.repo, &DiscoveryOptions::default()).unwrap();
    assert!(report.complete, "{:?}", report.issues);
    assert_eq!(report.repositories.len(), 4);
    let relation = |kind: fn(&RelationshipKind) -> bool, path: &str| {
        report.relationships.iter().any(|r| {
            kind(&r.kind)
                && r.child.to_path().unwrap() == fs::canonicalize(f.repo.join(path)).unwrap()
        })
    };
    assert!(relation(
        |k| matches!(k, RelationshipKind::NestedIndependent),
        "nested"
    ));
    assert!(relation(
        |k| matches!(k, RelationshipKind::Submodule),
        "module"
    ));
    assert!(relation(
        |k| matches!(k, RelationshipKind::LinkedWorktree),
        "linked"
    ));
    assert!(report
        .relationships
        .iter()
        .any(|r| r.state == "outside_workspace"));
    assert_eq!(before, tree_bytes(&f.root));
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("mixed", &f.repo).unwrap();
    let entries = service.scan(&workspace.id).unwrap().entries;
    let a = entries
        .iter()
        .find(|entry| entry.metadata.root.to_path().unwrap() == fs::canonicalize(&f.repo).unwrap())
        .unwrap();
    let b = entries
        .iter()
        .find(|entry| entry.metadata.root.to_path().unwrap() == fs::canonicalize(&linked).unwrap())
        .unwrap();
    assert_eq!(a.repository_id, b.repository_id);
    assert_ne!(a.id, b.id);
}

#[test]
fn exclusions_partial_scans_and_unavailable_entries_are_explicit() {
    let f = Fixture::new();
    f.commit_file();
    let nested = f.repo.join("generated");
    fs::create_dir(&nested).unwrap();
    f.git(&["-C", nested.to_str().unwrap(), "init", "-q"]);
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("workspace", &f.repo).unwrap();
    assert_eq!(service.scan(&workspace.id).unwrap().entries.len(), 2);
    service
        .update(
            &workspace.id,
            None,
            None,
            Some(DiscoveryOptions {
                excluded_names: vec!["generated".into()],
                excluded_patterns: vec![],
            }),
        )
        .unwrap();
    let report = service.scan(&workspace.id).unwrap();
    assert_eq!(report.scan.repositories.len(), 1);
    assert!(report
        .entries
        .iter()
        .any(|entry| matches!(entry.availability, Availability::Missing)));
    f.git(&["config", "filter.block.clean", "echo must-not-run"]);
    let report = service.scan(&workspace.id).unwrap();
    assert!(!report.scan.complete);
    assert!(!report.scan.issues.is_empty());
    assert!(report
        .entries
        .iter()
        .all(|entry| matches!(entry.availability, Availability::Unavailable)));
    fs::rename(&f.repo, f.root.join("gone")).unwrap();
    let report = service.scan(&workspace.id).unwrap();
    assert!(!report.scan.complete);
    assert_eq!(report.entries.len(), 2);
}

#[test]
fn targeted_refresh_does_not_rediscover_and_replacement_requires_rescan() {
    let f = Fixture::new();
    f.commit_file();
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("workspace", &f.repo).unwrap();
    let entry = service.scan(&workspace.id).unwrap().entries.remove(0);
    let bad = f.repo.join("new-broken-repo");
    fs::create_dir(&bad).unwrap();
    fs::write(bad.join(".git"), "broken").unwrap();
    fs::write(f.repo.join("file.txt"), b"external edit\n").unwrap();
    assert_eq!(
        service
            .refresh(&workspace.id, &entry.id)
            .unwrap()
            .status
            .unwrap()
            .counts
            .unstaged,
        1
    );
    fs::rename(f.repo.join(".git"), f.root.join("previous-git")).unwrap();
    f.git(&["init", "-q"]);
    assert_eq!(
        service.refresh(&workspace.id, &entry.id).unwrap_err().kind,
        ErrorKind::ConcurrentChange
    );
}

#[test]
fn invalid_and_newer_configs_and_schemas_are_preserved() {
    let f = Fixture::new();
    let state = f.root.join("state");
    fs::create_dir(&state).unwrap();
    for text in ["not valid [", "version = 999\nworkspaces = []\n"] {
        fs::write(state.join("config.toml"), text).unwrap();
        assert!(WorkspaceService::open(&state).is_err());
        assert_eq!(fs::read_to_string(state.join("config.toml")).unwrap(), text);
    }
    fs::remove_file(state.join("config.toml")).unwrap();
    let database = rusqlite::Connection::open(state.join("state.db")).unwrap();
    database.execute_batch("PRAGMA user_version=999;").unwrap();
    drop(database);
    let before = fs::read(state.join("state.db")).unwrap();
    assert_eq!(
        WorkspaceService::open(&state).err().unwrap().kind,
        ErrorKind::NewerSchema
    );
    assert_eq!(fs::read(state.join("state.db")).unwrap(), before);
}

#[test]
fn startup_reconciles_authoritative_toml_after_database_update_was_interrupted() {
    let f = Fixture::new();
    let state = f.root.join("state");
    let mut service = WorkspaceService::open(&state).unwrap();
    let workspace = service.create("workspace", &f.repo).unwrap();
    service.activate(&workspace.id).unwrap();
    service.scan(&workspace.id).unwrap();
    drop(service);
    // Simulate a committed TOML rename/removal followed by interruption before SQLite reconcile.
    fs::write(state.join("config.toml"), "version = 1\nworkspaces = []\n").unwrap();
    let service = WorkspaceService::open(&state).unwrap();
    assert!(service.list().is_empty());
    assert!(service.active().unwrap().is_none());
    drop(service);
    let database = rusqlite::Connection::open(state.join("state.db")).unwrap();
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM membership", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn state_inside_checkout_and_invalid_workspace_definitions_are_rejected() {
    let f = Fixture::new();
    f.commit_file();
    let before = tree_bytes(&f.repo);
    assert_eq!(
        WorkspaceService::open(&f.repo.join("state"))
            .err()
            .unwrap()
            .kind,
        ErrorKind::InvalidPath
    );
    assert_eq!(before, tree_bytes(&f.repo));
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    assert!(service.create(" ", &f.repo).is_err());
    assert!(service.create("file", &f.repo.join("file.txt")).is_err());
    assert!(service.create("contains state", &f.root).is_err());
    assert!(service.activate(&WorkspaceId("unknown".into())).is_err());
}

#[test]
fn cli_and_service_workspace_results_agree_and_preserve_repository_bytes() {
    let f = Fixture::new();
    f.commit_file();
    let before = tree_bytes(&f.repo);
    let state = f.root.join("state");
    let output = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
        .arg("--data-dir")
        .arg(&state)
        .args(["workspace", "create", "CLI"])
        .arg(&f.repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let created: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let service = WorkspaceService::open(&state).unwrap();
    assert_eq!(serde_json::to_value(&service.list()[0]).unwrap(), created);
    assert_eq!(before, tree_bytes(&f.repo));
}

#[cfg(unix)]
#[test]
fn partial_permission_scan_and_glob_exclusions_preserve_known_identity() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let f = Fixture::new();
    f.commit_file();
    let blocked = f.repo.join("blocked");
    fs::create_dir(&blocked).unwrap();
    let scratch = f.repo.join("scratch");
    fs::create_dir(&scratch).unwrap();
    let excluded = scratch.join("generated");
    fs::create_dir(&excluded).unwrap();
    f.git(&["-C", excluded.to_str().unwrap(), "init", "-q"]);
    let alias = f.root.join("alias");
    symlink(&f.repo, &alias).unwrap();
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("alias", &alias).unwrap();
    assert_eq!(workspace.display_root.to_path().unwrap(), alias);
    let initial = service.scan(&workspace.id).unwrap();
    let id = initial
        .entries
        .iter()
        .find(|e| e.metadata.root.to_path().unwrap() == fs::canonicalize(&f.repo).unwrap())
        .unwrap()
        .id
        .clone();
    let options = DiscoveryOptions {
        excluded_names: vec![],
        excluded_patterns: vec!["scratch/*".into()],
    };
    service
        .update(&workspace.id, None, None, Some(options))
        .unwrap();
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).unwrap();
    let result = service.scan(&workspace.id);
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755)).unwrap();
    let report = result.unwrap();
    assert!(!report.scan.complete);
    assert!(!report.scan.issues.is_empty());
    assert_eq!(report.scan.repositories.len(), 1);
    assert!(report.entries.iter().any(|e| e.id == id));
    assert!(report
        .entries
        .iter()
        .any(|e| matches!(e.availability, Availability::Unavailable)));
}
#[cfg(unix)]
#[test]
fn failed_atomic_config_save_preserves_previous_toml_and_memory() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let state = f.root.join("state");
    let mut service = WorkspaceService::open(&state).unwrap();
    let workspace = service.create("original", &f.repo).unwrap();
    let before = fs::read(state.join("config.toml")).unwrap();
    fs::set_permissions(&state, fs::Permissions::from_mode(0o500)).unwrap();
    let result = service.update(&workspace.id, Some("must-not-save"), None, None);
    fs::set_permissions(&state, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read(state.join("config.toml")).unwrap(), before);
    assert_eq!(service.list()[0].name, "original");
}
#[test]
fn unversioned_unknown_database_is_not_migrated_and_empty_database_is_backed_up() {
    let f = Fixture::new();
    let state = f.root.join("state");
    fs::create_dir(&state).unwrap();
    let db = rusqlite::Connection::open(state.join("state.db")).unwrap();
    db.execute_batch("CREATE TABLE unknown(value TEXT); INSERT INTO unknown VALUES('preserve');")
        .unwrap();
    drop(db);
    assert!(WorkspaceService::open(&state).is_err());
    let db = rusqlite::Connection::open(state.join("state.db")).unwrap();
    assert_eq!(
        db.query_row("SELECT value FROM unknown", [], |row| row
            .get::<_, String>(0))
            .unwrap(),
        "preserve"
    );
    drop(db);
    fs::remove_file(state.join("state.db")).unwrap();
    let db = rusqlite::Connection::open(state.join("state.db")).unwrap();
    db.execute_batch("PRAGMA user_version=0;").unwrap();
    drop(db);
    let service = WorkspaceService::open(&state).unwrap();
    assert!(fs::read_dir(&state).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with("state.pre-v1-")));
    drop(service);
}

#[test]
fn submodule_gitlinks_and_child_dirt_are_reported_without_recursive_helpers() {
    let f = Fixture::new();
    f.commit_file();
    let source = Fixture::new();
    source.commit_file();
    f.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        source.repo.to_str().unwrap(),
        "module",
    ]);
    let parent_status = gitfleet_core::inspection::snapshot(&f.repo).unwrap();
    assert!(parent_status.changes.iter().any(|c| c.path
        == gitfleet_core::domain::NativePath::from_path(std::path::Path::new("module"))
        && c.index_mode == "160000"));
    let staged =
        gitfleet_core::diff::inspect(&f.repo, gitfleet_core::diff::DiffMode::Staged, None).unwrap();
    assert!(staged.files.iter().any(|file| file.new_mode == "160000"));
    fs::write(f.repo.join("module/file.txt"), b"child dirty\n").unwrap();
    let before = tree_bytes(&f.root);
    let report = discovery::scan(&f.repo, &DiscoveryOptions::default()).unwrap();
    let relation = report
        .relationships
        .iter()
        .find(|r| matches!(r.kind, RelationshipKind::Submodule))
        .unwrap();
    assert_eq!(relation.child_dirty, Some(true));
    assert_eq!(relation.gitlink_matches_head, Some(true));
    assert!(relation.child_head.is_some());
    assert_eq!(before, tree_bytes(&f.root));
}

#[test]
fn changing_workspace_root_invalidates_old_inventory_and_refresh_scope() {
    let f = Fixture::new();
    f.commit_file();
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("workspace", &f.repo).unwrap();
    let entry = service.scan(&workspace.id).unwrap().entries.remove(0);
    let other = f.root.join("other-root");
    fs::create_dir(&other).unwrap();
    service
        .update(&workspace.id, None, Some(&other), None)
        .unwrap();
    assert!(service.last_scan(&workspace.id).unwrap().is_none());
    assert!(matches!(
        service.inventory(&workspace.id).unwrap()[0].availability,
        Availability::Unavailable
    ));
    let before = tree_bytes(&f.repo);
    assert_eq!(
        service.refresh(&workspace.id, &entry.id).unwrap_err().kind,
        ErrorKind::InvalidPath
    );
    assert_eq!(before, tree_bytes(&f.repo));
}
