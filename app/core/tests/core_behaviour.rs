use gitfleet_core::{discovery, git, lab::Fixture};
use std::fs;

#[test]
fn discover_nested_repositories_and_gitfile_marker() {
    let f = Fixture::new().unwrap();
    let nested = f.repo.join("nested");
    fs::create_dir_all(&nested).unwrap();
    git::run(&nested, &["init", "-q"], None).unwrap();
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 2);
    // Discovery returns canonical paths. On macOS /var typically resolves to /private/var.
    let expected_root = fs::canonicalize(&f.repo).unwrap();
    let expected_nested = fs::canonicalize(&nested).unwrap();
    assert!(found.iter().any(|r| r.path == expected_root));
    assert!(found.iter().any(|r| r.path == expected_nested));
}

#[cfg(unix)]
#[test]
fn discovery_canonicalizes_symlinked_root() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new().unwrap();
    let alias = f.root.join("repository-alias");
    symlink(&f.repo, &alias).unwrap();
    let found = discovery::discover(&alias).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, fs::canonicalize(&f.repo).unwrap());
}

#[test]
fn discover_linked_worktree_with_git_file() {
    let f = Fixture::new().unwrap();
    let linked = f.repo.join("linked-worktree");
    let linked_arg = linked.to_str().unwrap();
    git::run(&f.repo, &["worktree", "add", "--detach", linked_arg], None).unwrap();
    assert!(linked.join(".git").is_file());
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|r| r.path == fs::canonicalize(&f.repo).unwrap()));
    assert!(found.iter().any(|r| r.path == fs::canonicalize(&linked).unwrap()));
}

#[test]
fn prevent_parent_path_escape() {
    let f = Fixture::new().unwrap();
    assert!(git::validate_path(&f.repo, "../outside.txt").is_err());
    assert!(git::validate_path(&f.repo, "/tmp/outside.txt").is_err());
    assert!(git::validate_path(&f.repo, ".git/config").is_err());
    assert!(git::validate_path(&f.repo, "src//nested.rs").is_err());
    assert!(git::validate_path(&f.repo, "valid file.rs").is_ok());
}

#[cfg(unix)]
#[test]
fn discovery_does_not_follow_symlinked_directories() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new().unwrap();
    symlink(&f.root, f.repo.join("loop")).unwrap();
    let found = discovery::discover(&f.repo).unwrap();
    assert_eq!(found.len(), 1);
}

#[test]
fn read_only_status_is_available() {
    let f = Fixture::new().unwrap();
    f.stage("full.txt", "staged\n").unwrap();
    let state = git::log_state(&f.repo).unwrap();
    assert!(state["status"].as_str().unwrap().contains("M  full.txt"));
}
