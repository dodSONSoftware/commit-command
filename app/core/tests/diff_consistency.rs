mod support;
use gitfleet_core::{
    diff::{self, DiffMode},
    inspection::ReadOptions,
    ErrorKind,
};
use std::{fs, process::Command};
use support::Fixture;

#[test]
fn whole_repository_diff_detects_change_during_patch() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("file.txt"), b"before patch\n").unwrap();
    let before = support::tree_bytes(&f.repo);
    let baseline = diff::inspect(&f.repo, DiffMode::Unstaged, None).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
        .args(["diff", f.repo.to_str().unwrap(), "unstaged"])
        .output()
        .unwrap();
    assert!(result.status.success());
    let cli: diff::Diff = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(baseline.files, cli.files);
    assert_eq!(baseline.patch, cli.patch);
    assert!(cli.complete);
    assert_eq!(support::tree_bytes(&f.repo), before);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bin = f.root.join("bin");
        fs::create_dir(&bin).unwrap();
        let wrapper = bin.join("git");
        fs::write(&wrapper,"#!/bin/sh\ncase \" $* \" in *' --patch '*) echo 'changed during patch' > file.txt;; esac\nPATH=${PATH#*:} exec git \"$@\"\n").unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        let path = std::env::var_os("PATH").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .args(["diff", f.repo.to_str().unwrap(), "unstaged"])
            .env(
                "PATH",
                std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&path)))
                    .unwrap(),
            )
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("concurrent_change"));
    }
    let options = ReadOptions::default();
    options.cancellation.cancel();
    assert_eq!(
        diff::inspect_with_options(&f.repo, DiffMode::Unstaged, None, &options)
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
}
