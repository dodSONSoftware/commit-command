use gitfleet_core::{
    commit::{self, PlanRequest, Source},
    git,
    lab::Fixture,
    ErrorKind,
};
use std::collections::BTreeMap;
use std::fs;

fn request(paths: &[(&str, Source)]) -> PlanRequest {
    PlanRequest {
        selected: paths
            .iter()
            .map(|(p, source)| (p.to_string(), *source))
            .collect(),
        message: "M0 approved\n".into(),
        hooks: true,
        sign: false,
    }
}

fn blob(f: &Fixture, revision: &str, path: &str) -> Vec<u8> {
    let output = git::command(
        &f.repo,
        &["cat-file", "blob", &format!("{revision}:{path}")],
        None,
        None,
    )
    .unwrap();
    assert!(output.status.success());
    output.stdout
}

#[test]
fn same_oid_branch_switch_invalidates_prepared_plan() {
    let f = Fixture::new().unwrap();
    git::run(&f.repo, &["branch", "other"], None).unwrap();
    f.stage("full.txt", "selected\n").unwrap();
    let old = git::run(&f.repo, &["rev-parse", "HEAD"], None).unwrap();
    commit::prepare(
        &f.repo,
        &f.journal(),
        &request(&[("full.txt", Source::Index)]),
    )
    .unwrap();
    git::run(&f.repo, &["checkout", "-q", "other"], None).unwrap();
    let index = fs::read(git::index_path(&f.repo).unwrap()).unwrap();
    let outcome = commit::execute(&f.repo, &f.journal(), None).unwrap();
    assert_eq!(outcome.status, "blocked_stale_head");
    for reference in ["HEAD", "refs/heads/main", "refs/heads/other"] {
        assert_eq!(
            git::run(&f.repo, &["rev-parse", reference], None).unwrap(),
            old
        );
    }
    assert_eq!(fs::read(git::index_path(&f.repo).unwrap()).unwrap(), index);
    assert_eq!(fs::read(f.repo.join("full.txt")).unwrap(), b"selected\n");
}

#[test]
fn selected_pathspec_magic_is_a_literal_filename() {
    for path in [":(glob)*.txt", "*.txt", "[ab].txt"] {
        let f = Fixture::new().unwrap();
        fs::write(f.repo.join(path), b"literal selected\n").unwrap();
        fs::write(f.repo.join("full.txt"), b"excluded modification\n").unwrap();
        f.stage("other.txt", "preserved staging\n").unwrap();
        commit::prepare(&f.repo, &f.journal(), &request(&[(path, Source::Worktree)])).unwrap();
        let outcome = commit::execute(&f.repo, &f.journal(), None).unwrap();
        assert_eq!(outcome.status, "committed");
        assert_eq!(blob(&f, "HEAD", path), b"literal selected\n");
        assert_eq!(blob(&f, "HEAD", "full.txt"), b"original\n");
        assert_eq!(blob(&f, "HEAD", "other.txt"), b"other original\n");
        assert_eq!(blob(&f, "", "other.txt"), b"preserved staging\n");
        assert_eq!(
            fs::read(f.repo.join("full.txt")).unwrap(),
            b"excluded modification\n"
        );
        let changed = git::command(
            &f.repo,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                "HEAD",
            ],
            None,
            None,
        )
        .unwrap();
        assert!(changed.status.success());
        assert_eq!(changed.stdout, [path.as_bytes(), &[0]].concat());
    }
}

#[test]
fn nonzero_hook_preserves_refs_index_and_worktree() {
    let f = Fixture::new().unwrap();
    f.stage("full.txt", "selected\n").unwrap();
    f.stage("other.txt", "preserved staging\n").unwrap();
    f.hook("pre-commit", "printf 'fixture rejection\\n' >&2\nexit 42")
        .unwrap();
    let head = git::run(&f.repo, &["rev-parse", "HEAD"], None).unwrap();
    let index = fs::read(git::index_path(&f.repo).unwrap()).unwrap();
    let error = commit::prepare(
        &f.repo,
        &f.journal(),
        &request(&[("full.txt", Source::Index)]),
    )
    .unwrap_err();
    assert_eq!(error.kind, ErrorKind::HookRejected);
    assert!(error.to_string().contains("42"));
    assert_eq!(
        git::run(&f.repo, &["rev-parse", "HEAD"], None).unwrap(),
        head
    );
    assert_eq!(fs::read(git::index_path(&f.repo).unwrap()).unwrap(), index);
    assert_eq!(fs::read(f.repo.join("full.txt")).unwrap(), b"selected\n");
    assert_eq!(
        fs::read(f.repo.join("other.txt")).unwrap(),
        b"preserved staging\n"
    );
    assert!(!f.journal().exists());
}

#[test]
fn partial_staging_and_binary_content_have_independent_oracles() {
    for source in [Source::Index, Source::Worktree] {
        let f = Fixture::new().unwrap();
        f.stage("partial.txt", "first staged\nsecond old\n")
            .unwrap();
        fs::write(
            f.repo.join("partial.txt"),
            b"first staged\nsecond unstaged\n",
        )
        .unwrap();
        f.stage("other.txt", "excluded staged\n").unwrap();
        let binary = [0, 255, 10, 13, 42];
        fs::write(f.repo.join("binary.bin"), binary).unwrap();
        let req = request(&[("partial.txt", source), ("binary.bin", Source::Worktree)]);
        commit::prepare(&f.repo, &f.journal(), &req).unwrap();
        assert_eq!(
            commit::execute(&f.repo, &f.journal(), None).unwrap().status,
            "committed"
        );
        let expected: &[u8] = if source == Source::Index {
            b"first staged\nsecond old\n"
        } else {
            b"first staged\nsecond unstaged\n"
        };
        assert_eq!(blob(&f, "HEAD", "partial.txt"), expected);
        assert_eq!(blob(&f, "", "partial.txt"), expected);
        assert_eq!(blob(&f, "HEAD", "binary.bin"), binary);
        assert_eq!(fs::read(f.repo.join("binary.bin")).unwrap(), binary);
        assert_eq!(
            fs::read(f.repo.join("partial.txt")).unwrap(),
            b"first staged\nsecond unstaged\n"
        );
        assert_eq!(blob(&f, "HEAD", "other.txt"), b"other original\n");
        assert_eq!(blob(&f, "", "other.txt"), b"excluded staged\n");
        let actual = git::entries(&f.repo, None).unwrap();
        assert!(actual.contains_key("binary.bin"));
        let raw = git::command(&f.repo, &["cat-file", "commit", "HEAD"], None, None).unwrap();
        assert!(raw.status.success());
        let split = raw
            .stdout
            .windows(2)
            .position(|pair| pair == b"\n\n")
            .unwrap();
        assert_eq!(&raw.stdout[split + 2..], b"M0 approved\n");
    }
}

#[test]
fn unknown_journal_phase_is_rejected() {
    let f = Fixture::new().unwrap();
    f.stage("full.txt", "selected\n").unwrap();
    commit::prepare(
        &f.repo,
        &f.journal(),
        &request(&[("full.txt", Source::Index)]),
    )
    .unwrap();
    let mut journal: serde_json::Value =
        serde_json::from_slice(&fs::read(f.journal()).unwrap()).unwrap();
    journal["state"] = "unrecognized".into();
    fs::write(f.journal(), serde_json::to_vec(&journal).unwrap()).unwrap();
    assert_eq!(
        gitfleet_core::journal::load(&f.journal()).unwrap_err().kind,
        ErrorKind::InvalidData
    );
}

#[test]
fn journal_phase_serialization_preserves_legacy_spelling() {
    use gitfleet_core::journal::JournalState;
    let expected = BTreeMap::from([
        ("prepared", JournalState::Prepared),
        ("commit_object_created", JournalState::CommitObjectCreated),
        ("ref_update_pending", JournalState::RefUpdatePending),
        (
            "ref_updated_pending_index",
            JournalState::RefUpdatedPendingIndex,
        ),
        ("index_promoted", JournalState::IndexPromoted),
        ("ref_rejected", JournalState::RefRejected),
        ("aborted_before_ref", JournalState::AbortedBeforeRef),
        ("complete", JournalState::Complete),
    ]);
    for (name, phase) in expected {
        assert_eq!(serde_json::to_value(phase).unwrap(), name);
        assert_eq!(
            serde_json::from_value::<JournalState>(name.into()).unwrap(),
            phase
        );
    }
}
