mod support;
use gitfleet_core::{activity, graph, history::HistoryScope, inspection::ReadOptions, ErrorKind};
use std::{collections::BTreeMap, fs};
use support::{tree_bytes, Fixture};

#[test]
fn graph_uses_actual_merge_parents_and_preserves_lanes_across_pages_without_writes() {
    let f = Fixture::new();
    f.commit_file();
    f.git(&["checkout", "-qb", "side"]);
    fs::write(f.repo.join("side.txt"), b"side\n").unwrap();
    f.git(&["add", "side.txt"]);
    f.git(&["commit", "-qm", "side"]);
    f.git(&["checkout", "-q", "main"]);
    fs::write(f.repo.join("main.txt"), b"main\n").unwrap();
    f.git(&["add", "main.txt"]);
    f.git(&["commit", "-qm", "main"]);
    f.git(&["merge", "--no-ff", "-qm", "merge", "side"]);
    f.git(&["tag", "-a", "v1", "-m", "release"]);
    let raw = f.git(&["rev-list", "--parents", "--branches"]);
    let expected: BTreeMap<_, _> = String::from_utf8(raw)
        .unwrap()
        .lines()
        .map(|s| {
            let mut words = s.split_whitespace();
            (
                words.next().unwrap().to_owned(),
                words.map(str::to_owned).collect::<Vec<_>>(),
            )
        })
        .collect();
    let before = tree_bytes(&f.repo);
    let options = ReadOptions::default();
    let all = graph::page(&f.repo, HistoryScope::LocalBranches, 100, None, &options).unwrap();
    assert_eq!(all.rows.len(), 4);
    for row in &all.rows {
        assert_eq!(
            row.commit
                .parents
                .iter()
                .map(|p| p.hex.clone())
                .collect::<Vec<_>>(),
            expected[&row.commit.oid.hex]
        );
        assert_eq!(row.before[row.lane], row.commit.oid);
        assert!(row.commit.parents.iter().all(|p| row.after.contains(p)));
    }
    let mut cursor = None;
    let mut rows = Vec::new();
    loop {
        let page = graph::page(
            &f.repo,
            HistoryScope::LocalBranches,
            1,
            cursor.as_ref(),
            &options,
        )
        .unwrap();
        rows.extend(page.rows);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(
        serde_json::to_value(&rows).unwrap(),
        serde_json::to_value(&all.rows).unwrap()
    );
    let tag = graph::page(
        &f.repo,
        HistoryScope::Reference("refs/tags/v1".into()),
        100,
        None,
        &options,
    )
    .unwrap();
    assert_eq!(tag.rows.len(), 4);
    assert_eq!(before, tree_bytes(&f.repo));
    let first = graph::page(&f.repo, HistoryScope::LocalBranches, 1, None, &options).unwrap();
    f.git(&["branch", "another"]);
    assert_eq!(
        graph::page(
            &f.repo,
            HistoryScope::LocalBranches,
            1,
            first.next.as_ref(),
            &options
        )
        .unwrap_err()
        .kind,
        ErrorKind::StaleCursor
    );
}

#[test]
fn activity_uses_committer_time_and_deduplicates_branches_with_explicit_truncation() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(f.repo.join("file.txt"), b"next\n").unwrap();
    f.git(&["add", "file.txt"]);
    f.git(&["commit", "-qm", "second"]);
    f.git(&["branch", "same-tip"]);
    let at = String::from_utf8(f.git(&["show", "-s", "--format=%ct", "HEAD"]))
        .unwrap()
        .trim()
        .parse::<u64>()
        .unwrap();
    let before = tree_bytes(&f.repo);
    let options = ReadOptions::default();
    let page = activity::query(
        &f.repo,
        HistoryScope::LocalBranches,
        at - 100,
        at + 100,
        100,
        &options,
    )
    .unwrap();
    assert!(page.complete);
    assert_eq!(page.commits.len(), 2);
    assert_eq!(page.commits[0].subject, b"second");
    assert_eq!(page.commits[0].committer_time, at as i64);
    let limited = activity::query(
        &f.repo,
        HistoryScope::LocalBranches,
        at - 100,
        at + 100,
        1,
        &options,
    )
    .unwrap();
    assert!(!limited.complete);
    assert_eq!(limited.commits.len(), 1);
    assert!(activity::query(
        &f.repo,
        HistoryScope::LocalBranches,
        at + 100,
        at,
        100,
        &options
    )
    .is_err());
    assert_eq!(tree_bytes(&f.repo), before);
}

#[test]
fn unborn_graph_and_activity_are_empty_and_malformed_graph_lanes_are_rejected() {
    let f = Fixture::new();
    let options = ReadOptions::default();
    let page = graph::page(&f.repo, HistoryScope::LocalBranches, 100, None, &options).unwrap();
    assert!(page.rows.is_empty());
    let page =
        activity::query(&f.repo, HistoryScope::LocalBranches, 0, 100, 100, &options).unwrap();
    assert!(page.complete && page.commits.is_empty());
    f.commit_file();
    f.git(&["commit", "--allow-empty", "-qm", "next"]);
    let first = graph::page(&f.repo, HistoryScope::LocalBranches, 1, None, &options).unwrap();
    let mut cursor = first.next.unwrap();
    cursor.lanes.push(cursor.lanes[0].clone());
    assert_eq!(
        graph::page(
            &f.repo,
            HistoryScope::LocalBranches,
            1,
            Some(&cursor),
            &options
        )
        .unwrap_err()
        .kind,
        ErrorKind::InvalidRequest
    );
}

#[test]
fn sha256_graph_retains_full_object_ids() {
    let f = Fixture::new();
    let repo = f.root.join("sha256");
    f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "-q",
        "--object-format=sha256",
        "-b",
        "main",
        repo.to_str().unwrap(),
    ]);
    let path = repo.to_str().unwrap();
    for (key, value) in [
        ("user.name", "Graph fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        f.git(&["-C", path, "config", key, value]);
    }
    fs::write(repo.join("file.txt"), b"sha256\n").unwrap();
    f.git(&["-C", path, "add", "file.txt"]);
    f.git(&[
        "-C",
        path,
        "-c",
        "core.hooksPath=",
        "commit",
        "-qm",
        "sha256",
    ]);
    let before = tree_bytes(&repo);
    let page = graph::page(
        &repo,
        HistoryScope::LocalBranches,
        100,
        None,
        &ReadOptions::default(),
    )
    .unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].commit.oid.hex.len(), 64);
    assert!(page.rows[0].tip);
    assert!(page.rows[0].commit.parents.is_empty());
    assert_eq!(before, tree_bytes(&repo));
}

#[test]
fn activity_does_not_prune_recent_ancestors_behind_older_committer_dates() {
    let f = Fixture::new();
    fn commit(f: &Fixture, date: &str, message: &str) {
        let mut cmd = std::process::Command::new("git");
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(key);
            }
        }
        let output = cmd
            .current_dir(&f.repo)
            .args([
                "-c",
                "core.hooksPath=",
                "-c",
                "maintenance.auto=false",
                "-c",
                "gc.auto=0",
                "commit",
                "--allow-empty",
                "-qm",
                message,
            ])
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_COMMITTER_DATE", date)
            .env("GIT_AUTHOR_DATE", date)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    commit(&f, "2026-10-20T12:00:00+0000", "recent ancestor");
    let at = String::from_utf8(f.git(&["show", "-s", "--format=%ct", "HEAD"]))
        .unwrap()
        .trim()
        .parse::<u64>()
        .unwrap();
    commit(&f, "2026-10-01T12:00:00+0000", "older descendant");
    let before = tree_bytes(&f.repo);
    let page = activity::query(
        &f.repo,
        HistoryScope::LocalBranches,
        at - 1,
        at + 1,
        100,
        &ReadOptions::default(),
    )
    .unwrap();
    assert!(page.complete);
    assert_eq!(page.commits.len(), 1);
    assert_eq!(page.commits[0].subject, b"recent ancestor");
    assert_eq!(before, tree_bytes(&f.repo));
}
