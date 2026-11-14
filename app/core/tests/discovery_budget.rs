mod support;
use gitfleet_core::{
    discovery::{self, DiscoveryOptions, Inventory},
    inspection::ReadOptions,
    workspace::WorkspaceService,
    ErrorKind,
};
use serde_json::Value;
use std::{
    fs,
    time::{Duration, Instant},
};
use support::{tree_bytes, Fixture};

fn stable(report: Inventory) -> Value {
    fn strip(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("observed_at_ms");
                for value in fields.values_mut() {
                    strip(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    strip(value);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(report).unwrap();
    strip(&mut value);
    value
}

#[test]
fn one_shot_and_workspace_agree_on_complete_excluded_and_unavailable_scans() {
    let f = Fixture::new();
    f.commit_file();
    for name in ["nested", "node_modules/ignored", "excluded/ignored"] {
        let path = f.repo.join(name);
        fs::create_dir_all(&path).unwrap();
        f.git(&[
            "-c",
            "init.templateDir=",
            "init",
            "-q",
            path.to_str().unwrap(),
        ]);
    }
    let options = DiscoveryOptions {
        excluded_patterns: vec!["excluded".into()],
        ..Default::default()
    };
    let before = tree_bytes(&f.repo);
    let state = f.root.join("state");
    let mut service = WorkspaceService::open(&state).unwrap();
    let workspace = service.create("Unified", &f.repo).unwrap();
    service
        .update(&workspace.id, None, None, Some(options.clone()))
        .unwrap();
    let one = discovery::scan(&f.repo, &options).unwrap();
    assert!(one.complete && one.repositories.len() == 2 && one.progress.is_none());
    let workspace_scan = service.scan(&workspace.id).unwrap().scan;
    assert_eq!(stable(one), stable(workspace_scan));
    assert_eq!(before, tree_bytes(&f.repo));
    let canonical_root = fs::canonicalize(&f.repo).unwrap();
    let moved = f.root.join("moved");
    fs::rename(&f.repo, &moved).unwrap();
    assert_eq!(
        stable(discovery::scan(&canonical_root, &options).unwrap()),
        stable(service.scan(&workspace.id).unwrap().scan)
    );
    fs::rename(moved, &f.repo).unwrap();
}

#[test]
fn large_directory_shares_deadline_but_only_workspace_retains_continuation() {
    let f = Fixture::new();
    let root = f.root.join("large");
    fs::create_dir(&root).unwrap();
    for i in 0..12000 {
        fs::create_dir(root.join(format!("entry-{i}"))).unwrap();
    }
    let read = ReadOptions {
        timeout: Duration::from_millis(1),
        ..Default::default()
    };
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("Large", &root).unwrap();
    let started = Instant::now();
    let one = discovery::scan_with_options(&root, &DiscoveryOptions::default(), &read).unwrap();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!one.complete && one.progress.is_none());
    assert!(one
        .issues
        .iter()
        .any(|issue| issue.error.kind == ErrorKind::Timeout));
    let started = Instant::now();
    let partial = service
        .scan_with_options(&workspace.id, &read)
        .unwrap()
        .scan;
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!partial.complete && partial.progress.is_some());
    assert!(service.can_continue_scan(&workspace.id));
    let cancelled = ReadOptions::default();
    cancelled.cancellation.cancel();
    assert_eq!(
        discovery::scan_with_options(&root, &DiscoveryOptions::default(), &cancelled)
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(
        service
            .continue_scan_with_options(&workspace.id, &cancelled)
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert!(service.can_continue_scan(&workspace.id));
    // Exclude the large fanout, invalidate the old cursor and finish a fresh scan.
    service
        .update(
            &workspace.id,
            None,
            None,
            Some(DiscoveryOptions {
                excluded_patterns: vec!["entry-*".into()],
                ..Default::default()
            }),
        )
        .unwrap();
    assert!(service.scan(&workspace.id).unwrap().scan.complete);
}

#[cfg(unix)]
#[test]
fn unreadable_child_has_the_same_partial_issue_in_both_entry_points() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let denied = fs::canonicalize(&f.repo).unwrap().join("denied");
    fs::create_dir(&denied).unwrap();
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("Denied", &f.repo).unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o0)).unwrap();
    let one = discovery::scan(&f.repo, &DiscoveryOptions::default());
    let saved = service.scan(&workspace.id);
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).unwrap();
    let one = one.unwrap();
    assert!(
        !one.complete
            && one
                .issues
                .iter()
                .any(|issue| issue.path.to_path().unwrap() == denied)
    );
    assert_eq!(stable(one), stable(saved.unwrap().scan));
}

#[cfg(unix)]
#[test]
fn slow_metadata_uses_caller_deadline_and_cancellation() {
    use std::{os::unix::fs::PermissionsExt, process::Command};
    const CHILD: &str = "GITFLEET_DISCOVERY_BUDGET_CHILD";
    let f = Fixture::new();
    if std::env::var_os(CHILD).is_none() {
        let bin = f.root.join("bin");
        fs::create_dir(&bin).unwrap();
        let wrapper = bin.join("git");
        fs::write(&wrapper,"#!/bin/sh\nif [ -f \"$PWD/slow-inspection\" ]; then echo started > \"$PWD/inspection-started\"; sleep 2; fi\nPATH=${PATH#*:} exec git \"$@\"\n").unwrap();
        fs::set_permissions(wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        let path = std::env::var_os("PATH").unwrap();
        let result = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "slow_metadata_uses_caller_deadline_and_cancellation",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env(
                "PATH",
                std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&path)))
                    .unwrap(),
            )
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    f.commit_file();
    let mut service = WorkspaceService::open(&f.root.join("state")).unwrap();
    let workspace = service.create("Slow", &f.repo).unwrap();
    fs::write(f.repo.join("slow-inspection"), "slow").unwrap();
    let read = ReadOptions {
        timeout: Duration::from_millis(100),
        ..Default::default()
    };
    for saved in [false, true] {
        let started = Instant::now();
        let report = if saved {
            service
                .scan_with_options(&workspace.id, &read)
                .unwrap()
                .scan
        } else {
            discovery::scan_with_options(&f.repo, &DiscoveryOptions::default(), &read).unwrap()
        };
        assert!(
            started.elapsed() < Duration::from_millis(800),
            "metadata ignored caller deadline: {:?}",
            started.elapsed()
        );
        assert!(
            !report.complete
                && report
                    .issues
                    .iter()
                    .any(|issue| issue.error.kind == ErrorKind::Timeout)
        );
        assert_eq!(report.progress.is_some(), saved);
    }
    for saved in [false, true] {
        let _ = fs::remove_file(f.repo.join("inspection-started"));
        let read = ReadOptions {
            timeout: Duration::from_secs(10),
            ..Default::default()
        };
        let token = read.cancellation.clone();
        let marker = f.repo.join("inspection-started");
        let cancel = std::thread::spawn(move || {
            let start = Instant::now();
            while !marker.exists() && start.elapsed() < Duration::from_secs(1) {
                std::thread::sleep(Duration::from_millis(5));
            }
            token.cancel();
        });
        let started = Instant::now();
        let result = if saved {
            service
                .scan_with_options(&workspace.id, &read)
                .map(|r| r.scan)
        } else {
            discovery::scan_with_options(&f.repo, &DiscoveryOptions::default(), &read)
        };
        cancel.join().unwrap();
        assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        assert!(started.elapsed() < Duration::from_millis(800));
    }
    fs::remove_file(f.repo.join("slow-inspection")).unwrap();
    assert!(
        service
            .continue_scan_with_options(&workspace.id, &ReadOptions::default())
            .unwrap()
            .scan
            .complete
    );
}

#[test]
fn cli_scan_and_workspace_scan_agree_and_discover_keeps_its_result_contract() {
    use std::process::Command;
    let f = Fixture::new();
    f.commit_file();
    let cli = |args: &[&std::ffi::OsStr]| {
        let result = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        serde_json::from_slice::<Value>(&result.stdout).unwrap()
    };
    let before = tree_bytes(&f.repo);
    let raw = cli(&["scan".as_ref(), f.repo.as_os_str()]);
    let state = f.root.join("cli-state");
    let created = cli(&[
        "--data-dir".as_ref(),
        state.as_os_str(),
        "workspace".as_ref(),
        "create".as_ref(),
        "CLI".as_ref(),
        f.repo.as_os_str(),
    ]);
    let id = created["id"].as_str().unwrap();
    let saved = cli(&[
        "--data-dir".as_ref(),
        state.as_os_str(),
        "workspace".as_ref(),
        "scan".as_ref(),
        id.as_ref(),
    ]);
    assert_eq!(
        stable(serde_json::from_value(raw).unwrap()),
        stable(serde_json::from_value(saved["scan"].clone()).unwrap())
    );
    let compatible = cli(&["discover".as_ref(), f.repo.as_os_str()]);
    assert_eq!(compatible.as_array().unwrap().len(), 1);
    assert!(compatible[0].get("path").is_some());
    assert_eq!(before, tree_bytes(&f.repo));
    let missing = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
        .arg("discover")
        .arg(f.root.join("missing"))
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("inspection_failed"));
}
