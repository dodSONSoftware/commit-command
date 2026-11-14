mod support;
use gitfleet_core::{
    activity,
    history::{self, HistoryScope},
    inspection::ReadOptions,
};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::{tree_bytes, Fixture};

#[test]
fn history_and_activity_bound_git_process_count() {
    const CHILD: &str = "GITFLEET_HISTORY_BATCH_ROOT";
    if let Some(root) = std::env::var_os(CHILD) {
        let options = ReadOptions {
            timeout: Duration::from_secs(60),
            ..Default::default()
        };
        let start = Instant::now();
        let page = history::page_with_options(
            root.as_ref(),
            HistoryScope::LocalBranches,
            500,
            None,
            &options,
        )
        .unwrap();
        assert_eq!(page.commits.len(), 500);
        assert!(page.next.is_none());
        assert_eq!(page.commits[0].message, b"commit-499\n");
        let history_elapsed = start.elapsed();
        let start = Instant::now();
        let page = activity::query(
            root.as_ref(),
            HistoryScope::LocalBranches,
            1699999999,
            1700000001,
            500,
            &options,
        )
        .unwrap();
        assert_eq!(page.commits.len(), 500);
        assert!(page.complete);
        assert_eq!(page.commits[0].subject, b"commit-499");
        println!(
            "history={history_elapsed:?}, activity={:?}",
            start.elapsed()
        );
        #[cfg(unix)]
        {
            let wrapper = std::path::PathBuf::from(&root)
                .parent()
                .unwrap()
                .join("bin/git");
            fs::write(wrapper.with_extension("blocked"), b"blocked").unwrap();
            let token = gitfleet_core::inspection::Cancellation::default();
            let cancel = token.clone();
            let started = wrapper.with_extension("started");
            let observer = std::thread::spawn(move || {
                let deadline = Instant::now();
                while !started.exists() {
                    assert!(deadline.elapsed() < Duration::from_secs(10));
                    std::thread::sleep(Duration::from_millis(5));
                }
                cancel.cancel();
            });
            let start = Instant::now();
            let error = history::page_with_options(
                root.as_ref(),
                HistoryScope::LocalBranches,
                500,
                None,
                &ReadOptions {
                    cancellation: token,
                    ..options
                },
            )
            .unwrap_err();
            observer.join().unwrap();
            assert_eq!(error.kind, gitfleet_core::ErrorKind::Cancelled);
            assert!(start.elapsed() < Duration::from_secs(3));
        }
        return;
    }
    let f = Fixture::new();
    f.commit_file();
    f.git(&["update-ref", "-d", "refs/heads/main"]);
    let mut importer = Command::new("git")
        .current_dir(&f.repo)
        .args(["fast-import", "--quiet"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut data = String::new();
    for n in 0..500 {
        let message = format!("commit-{n}\n");
        data.push_str(&format!("commit refs/heads/main\ncommitter Fixture <fixture@example.invalid> 1700000000 +0000\ndata {}\n{}\n",message.len(),message));
    }
    importer
        .stdin
        .take()
        .unwrap()
        .write_all(data.as_bytes())
        .unwrap();
    assert!(importer.wait().unwrap().success());
    let bin = f.root.join("bin");
    fs::create_dir(&bin).unwrap();
    let wrapper = bin.join("git");
    fs::write(
        &wrapper,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$0.calls\"\ncase \" $* \" in *' cat-file --batch '*) if [ -f \"$0.blocked\" ]; then touch \"$0.started\"; sleep 5; fi;; esac\nPATH=${PATH#*:} exec git \"$@\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let path = std::env::var_os("PATH").unwrap();
    let before = tree_bytes(&f.repo);
    let result = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "history_and_activity_bound_git_process_count",
            "--nocapture",
        ])
        .env(CHILD, &f.repo)
        .env(
            "PATH",
            std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&path))).unwrap(),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let calls = fs::read_to_string(wrapper.with_extension("calls")).unwrap();
    let commits = calls
        .lines()
        .filter(|line| line.contains("cat-file commit ") || line.contains("cat-file --batch"))
        .count();
    eprintln!(
        "Git object-reader processes={commits}; {}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert_eq!(tree_bytes(&f.repo), before);
    assert!(
        commits < 20,
        "per-commit process overhead remains: {commits}"
    );
}
