#![cfg(target_os = "macos")]
mod support;
use gitfleet_core::{
    domain::{NativePath, WorkspaceId, WorktreeId},
    workspace::WorkspaceService,
    writes::*,
    ErrorKind,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use support::{tree_bytes, Fixture};

struct M2 {
    f: Fixture,
    state: PathBuf,
    workspace: WorkspaceId,
    worktree: WorktreeId,
    options: Options,
    home: PathBuf,
}
impl M2 {
    fn new() -> Self {
        Self::with_initial(true)
    }
    fn with_initial(initial: bool) -> Self {
        let f = Fixture::new();
        if initial {
            f.commit_file();
        }
        fs::write(
            f.repo.join(".gitfleet-m2-fixture"),
            b"disposable M2 fixture\n",
        )
        .unwrap();
        let state = f.root.join("state");
        let global = f.root.join("global.config");
        fs::write(&global, b"").unwrap();
        let home = PathBuf::from(format!("/private/tmp/gf-m2-key-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        let options = Options {
            global_config: Some(global),
            signer_home: Some(home.clone()),
            ..Options::default()
        };
        let mut ws = WorkspaceService::open(&state).unwrap();
        let workspace = ws.create("M2", &f.repo).unwrap().id;
        let worktree = ws.scan(&workspace).unwrap().entries.remove(0).id;
        drop(ws);
        Self {
            f,
            state,
            workspace,
            worktree,
            options,
            home,
        }
    }
    fn service(&self) -> ControlledWrites {
        ControlledWrites::open(&self.state, self.options.clone()).unwrap()
    }
    fn trust(&self, s: &mut ControlledWrites) {
        let review = s.inspect_trust(&self.workspace, &self.worktree).unwrap();
        s.grant_trust(&review).unwrap();
    }
    fn hook(&self, name: &str, script: &str) {
        let path = self.f.repo.join(".git/hooks").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, format!("#!/bin/sh\nset -eu\n{script}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fn request(&self, paths: &[(&str, ContentSource)], hooks: HookPolicy) -> WriteRequest {
        WriteRequest::Commit {
            selected: paths
                .iter()
                .map(|(path, source)| SelectedPath {
                    path: NativePath::from_path(Path::new(path)),
                    source: *source,
                })
                .collect(),
            message: "Approved M2 message\n\nBody with trailing spaces.  \n".into(),
            hooks,
            signing: SigningPreference::Configured,
        }
    }
    fn execute(&self, s: &mut ControlledWrites, request: WriteRequest) -> Outcome {
        let p = s.propose(&self.workspace, &self.worktree, request).unwrap();
        let a = s.approve(&p).unwrap();
        s.execute(&a).unwrap()
    }
    fn blob(&self, revision: &str, path: &str) -> Vec<u8> {
        self.f
            .git(&["cat-file", "blob", &format!("{revision}:{path}")])
    }
}
impl Drop for M2 {
    fn drop(&mut self) {
        if let Some(gpg) = std::env::var_os("GITFLEET_TEST_GPG") {
            let conf = PathBuf::from(gpg).parent().unwrap().join("gpgconf");
            if conf.is_file() {
                let _ = Command::new(conf)
                    .args(["--homedir"])
                    .arg(&self.home)
                    .args(["--kill", "gpg-agent"])
                    .output();
            }
        }
        let _ = fs::remove_dir_all(&self.home);
    }
}
#[test]
fn untrusted_proposals_and_forged_trust_reviews_are_rejected_without_repo_writes() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
    let before = tree_bytes(&t.f.repo);
    let mut s = t.service();
    assert_eq!(
        s.propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute
            )
        )
        .unwrap_err()
        .kind,
        ErrorKind::UntrustedRepository
    );
    let mut review = s.inspect_trust(&t.workspace, &t.worktree).unwrap();
    review.binding = "forged".into();
    assert_eq!(
        s.grant_trust(&review).unwrap_err().kind,
        ErrorKind::StaleReview
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[test]
fn exact_worktree_commit_is_reviewed_before_any_repo_object_write_and_idempotent() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved exact\n").unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let before = tree_bytes(&t.f.repo);
    let proposal = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    assert_eq!(before, tree_bytes(&t.f.repo));
    let approval = s.approve(&proposal).unwrap();
    assert_eq!(before, tree_bytes(&t.f.repo));
    let outcome = s.execute(&approval).unwrap();
    assert_eq!(outcome.phase, Phase::Complete, "{outcome:?}");
    assert!(outcome.published);
    assert!(outcome.index_reconciled);
    assert_eq!(t.blob("HEAD", "file.txt"), b"approved exact\n");
    let raw = t.f.git(&["cat-file", "commit", "HEAD"]);
    let position = raw.windows(2).position(|p| p == b"\n\n").unwrap();
    assert_eq!(&raw[position + 2..], proposal.message);
    assert_eq!(s.execute(&approval).unwrap().new_head, outcome.new_head);
    assert!(!t.f.repo.join(".git/index.lock").exists());
}
#[test]
fn stage_unstage_and_commit_preserve_partial_and_excluded_staging() {
    let t = M2::new();
    fs::write(t.f.repo.join("other"), b"excluded staged\n").unwrap();
    t.f.git(&["add", "other"]);
    fs::write(t.f.repo.join("file.txt"), b"index version\n").unwrap();
    t.f.git(&["add", "file.txt"]);
    fs::write(t.f.repo.join("file.txt"), b"working version\n").unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let outcome = t.execute(
        &mut s,
        t.request(&[("file.txt", ContentSource::Index)], HookPolicy::Execute),
    );
    assert_eq!(outcome.phase, Phase::Complete, "{outcome:?}");
    assert_eq!(t.blob("HEAD", "file.txt"), b"index version\n");
    assert_eq!(
        fs::read(t.f.repo.join("file.txt")).unwrap(),
        b"working version\n"
    );
    assert_eq!(t.blob("", "other"), b"excluded staged\n");
    let p = NativePath::from_path(Path::new("file.txt"));
    let outcome = t.execute(
        &mut s,
        WriteRequest::Stage {
            paths: vec![p.clone()],
            hooks: HookPolicy::Execute,
        },
    );
    assert_eq!(outcome.phase, Phase::Complete, "{outcome:?}");
    assert!(!outcome.published);
    assert_eq!(t.blob("", "file.txt"), b"working version\n");
    let outcome = t.execute(
        &mut s,
        WriteRequest::Unstage {
            paths: vec![p],
            hooks: HookPolicy::Execute,
        },
    );
    assert_eq!(outcome.phase, Phase::Complete, "{outcome:?}");
    assert_eq!(t.blob("", "file.txt"), b"index version\n");
    assert_eq!(
        fs::read(t.f.repo.join("file.txt")).unwrap(),
        b"working version\n"
    );
    assert_eq!(t.blob("", "other"), b"excluded staged\n");
}

#[test]
fn approvals_reject_tampering_expiry_restart_and_same_oid_branch_switch() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    let mut forged = p.clone();
    forged.message.push(b'!');
    assert_eq!(s.approve(&forged).unwrap_err().kind, ErrorKind::StaleReview);
    let a = s.approve(&p).unwrap();
    t.f.git(&["branch", "other"]);
    t.f.git(&["switch", "other"]);
    let before = tree_bytes(&t.f.repo);
    assert_eq!(s.execute(&a).unwrap().phase, Phase::NeedsReauthorization);
    assert_eq!(before, tree_bytes(&t.f.repo));
    drop(s);
    let mut s = t.service();
    assert_eq!(s.execute(&a).unwrap_err().kind, ErrorKind::StaleReview);
    drop(s);
    let mut options = t.options.clone();
    options.approval_lifetime = std::time::Duration::ZERO;
    let mut s = ControlledWrites::open(&t.state, options).unwrap();
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    let a = s.approve(&p).unwrap();
    assert_eq!(s.execute(&a).unwrap_err().kind, ErrorKind::StaleReview);
}
#[test]
fn trust_revocation_and_changed_hook_or_configuration_block_before_publication() {
    for change in ["revoke", "hook", "config"] {
        let t = M2::new();
        fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
        let mut s = t.service();
        t.trust(&mut s);
        let p = s
            .propose(
                &t.workspace,
                &t.worktree,
                t.request(
                    &[("file.txt", ContentSource::WorkingTree)],
                    HookPolicy::Execute,
                ),
            )
            .unwrap();
        let a = s.approve(&p).unwrap();
        match change {
            "revoke" => s.revoke_trust(&t.worktree).unwrap(),
            "hook" => t.hook("pre-commit", "touch sentinel"),
            _ => {
                t.f.git(&["config", "user.email", "other@example.invalid"]);
            }
        }
        let before = tree_bytes(&t.f.repo);
        let outcome = s.execute(&a).unwrap();
        assert_eq!(
            outcome.phase,
            Phase::NeedsReauthorization,
            "{change}: {outcome:?}"
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
}
#[test]
fn hooks_reject_tree_message_and_selected_content_changes_without_publishing() {
    for script in [
        "exit 9",
        "printf rewrite > \"$1\"",
        "printf changed > file.txt",
        "printf injected > extra; git add -- extra",
    ] {
        let t = M2::new();
        fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
        t.hook(
            if script.contains("$1") {
                "commit-msg"
            } else {
                "pre-commit"
            },
            script,
        );
        let old = t.f.git(&["rev-parse", "HEAD"]);
        let index = fs::read(t.f.repo.join(".git/index")).unwrap();
        let mut s = t.service();
        t.trust(&mut s);
        let out = t.execute(
            &mut s,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        );
        assert!(!out.published, "{script}: {out:?}");
        assert!(
            matches!(out.phase, Phase::Blocked | Phase::NeedsReauthorization),
            "{out:?}"
        );
        assert_eq!(old, t.f.git(&["rev-parse", "HEAD"]));
        assert_eq!(index, fs::read(t.f.repo.join(".git/index")).unwrap());
        assert!(!t.f.repo.join(".git/index.lock").exists());
    }
}
#[test]
fn hook_bypass_is_complete_and_managers_and_reference_hooks_fail_closed() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    for name in [
        "pre-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "post-index-change",
        "reference-transaction",
    ] {
        t.hook(name, "touch sentinel; exit 3");
    }
    let mut s = t.service();
    t.trust(&mut s);
    assert_eq!(
        s.propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute
            )
        )
        .unwrap_err()
        .kind,
        ErrorKind::UnsupportedRepositoryState
    );
    s.remember_hook_policy(&t.worktree, HookPolicy::BypassAll)
        .unwrap();
    assert_eq!(s.hook_policy(&t.worktree).unwrap(), HookPolicy::BypassAll);
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::BypassAll,
        ),
    );
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert!(!t.f.repo.join("sentinel").exists());
    let t = M2::new();
    t.hook("pre-commit", "# lefthook\nexit 0");
    let mut s = t.service();
    t.trust(&mut s);
    assert!(s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute
            )
        )
        .is_err());
}
#[test]
fn post_hook_failure_preserves_published_commit_and_excluded_side_effects_are_reported() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    t.hook("pre-commit", "printf side-effect > excluded");
    t.hook("post-commit", "printf post >> counter; exit 4");
    let mut s = t.service();
    t.trust(&mut s);
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert_eq!(
        out.phase,
        Phase::PublishedWithPostProcessingFailure,
        "{out:?}"
    );
    assert!(out.published);
    assert!(out.index_reconciled);
    assert_eq!(t.blob("HEAD", "file.txt"), b"approved\n");
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", "HEAD:excluded"])
        .status
        .success());
    assert_eq!(fs::read(t.f.repo.join("excluded")).unwrap(), b"side-effect");
    assert!(!out.post_processing.is_empty());
}
#[test]
fn literal_binary_symlink_executable_deletion_and_initial_commit_have_exact_trees() {
    let t = M2::with_initial(false);
    let names = [":(glob)*", "--literal", "line\nname", "binary"];
    for name in names {
        fs::write(t.f.repo.join(name), b"binary\0\xff\n").unwrap();
    }
    fs::write(t.f.repo.join("run"), b"#!/bin/sh\n").unwrap();
    fs::set_permissions(t.f.repo.join("run"), fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink("binary", t.f.repo.join("link")).unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let out = t.execute(
        &mut s,
        t.request(
            &names
                .iter()
                .chain(["run", "link"].iter())
                .map(|p| (*p, ContentSource::WorkingTree))
                .collect::<Vec<_>>(),
            HookPolicy::Execute,
        ),
    );
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    for name in names {
        assert_eq!(t.blob("HEAD", name), b"binary\0\xff\n");
    }
    assert_eq!(t.blob("HEAD", "link"), b"binary");
    assert!(String::from_utf8(t.f.git(&["ls-tree", "HEAD", "run"]))
        .unwrap()
        .starts_with("100755"));
    fs::remove_file(t.f.repo.join("binary")).unwrap();
    let out = t.execute(
        &mut s,
        t.request(
            &[("binary", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", "HEAD:binary"])
        .status
        .success());
}
#[test]
fn unsupported_indexes_operation_states_filters_and_foreign_locks_do_not_write() {
    for setup in [
        "assume",
        "intent",
        "v4",
        "split",
        "merge",
        "headlock",
        "indexlock",
        "filter",
        "symlink",
    ] {
        let t = M2::new();
        match setup {
            "assume" => {
                t.f.git(&["update-index", "--assume-unchanged", "file.txt"]);
            }
            "intent" => {
                fs::write(t.f.repo.join("new"), b"new").unwrap();
                t.f.git(&["add", "-N", "new"]);
            }
            "v4" => {
                t.f.git(&["update-index", "--index-version=4"]);
            }
            "split" => {
                t.f.git(&["update-index", "--split-index"]);
            }
            "merge" => {
                fs::write(
                    t.f.repo.join(".git/MERGE_HEAD"),
                    t.f.git(&["rev-parse", "HEAD"]),
                )
                .unwrap();
            }
            "headlock" => {
                fs::write(t.f.repo.join(".git/HEAD.lock"), b"foreign").unwrap();
            }
            "indexlock" => {
                fs::write(t.f.repo.join(".git/index.lock"), b"foreign").unwrap();
            }
            "filter" => {
                t.f.git(&["config", "filter.bad.clean", "touch sentinel"]);
            }
            _ => {
                fs::rename(t.f.repo.join(".git/objects"), t.f.root.join("objects")).unwrap();
                std::os::unix::fs::symlink(t.f.root.join("objects"), t.f.repo.join(".git/objects"))
                    .unwrap();
            }
        }
        let before = tree_bytes(&t.f.repo);
        let mut s = t.service();
        assert!(
            s.inspect_trust(&t.workspace, &t.worktree).is_err(),
            "{setup}"
        );
        assert_eq!(before, tree_bytes(&t.f.repo), "{setup}");
    }
}
fn generate_ssh(t: &M2) -> PathBuf {
    let key = t.home.join("ssh");
    let out = Command::new("/usr/bin/ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-f"])
        .arg(&key)
        .output()
        .unwrap();
    assert!(out.status.success());
    key
}
#[test]
fn ssh_signing_is_verified_against_approved_key_without_unsigned_fallback() {
    let t = M2::new();
    let key = generate_ssh(&t);
    t.f.git(&["config", "commit.gpgsign", "yes"]);
    t.f.git(&["config", "gpg.format", "ssh"]);
    t.f.git(&["config", "user.signingkey", key.to_str().unwrap()]);
    fs::write(t.f.repo.join("file.txt"), b"ssh approved\n").unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    assert_eq!(p.signing.as_ref().unwrap().format, "ssh");
    let a = s.approve(&p).unwrap();
    let out = s.execute(&a).unwrap();
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    let allowed = t.home.join("allowed");
    fs::write(
        &allowed,
        [
            b"fixture@example.invalid ".as_slice(),
            &fs::read(key.with_extension("pub")).unwrap(),
        ]
        .concat(),
    )
    .unwrap();
    let verify = t.f.git_output(&[
        "-c",
        &format!("gpg.ssh.allowedSignersFile={}", allowed.display()),
        "verify-commit",
        "HEAD",
    ]);
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    fs::write(t.f.repo.join("file.txt"), b"next\n").unwrap();
    fs::remove_file(key).unwrap();
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert!(!out.published);
    assert_eq!(out.phase, Phase::Blocked);
}
#[test]
fn openpgp_signing_positive_gate_requires_isolated_tool_and_keyring() {
    let gpg = std::env::var_os("GITFLEET_TEST_GPG")
        .expect("M2 positive OpenPGP gate requires GITFLEET_TEST_GPG (no silent skip)");
    let t = M2::new();
    let output = Command::new(&gpg)
        .arg("--homedir")
        .arg(&t.home)
        .args([
            "--batch",
            "--pinentry-mode",
            "loopback",
            "--passphrase",
            "",
            "--quick-generate-key",
            "M2 Fixture <fixture@example.invalid>",
            "ed25519",
            "sign",
            "0",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&gpg)
        .arg("--homedir")
        .arg(&t.home)
        .args(["--with-colons", "--list-secret-keys"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let listing = String::from_utf8(output.stdout).unwrap();
    let fpr = listing
        .lines()
        .find(|l| l.starts_with("fpr:"))
        .unwrap()
        .split(':')
        .nth(9)
        .unwrap();
    t.f.git(&["config", "commit.gpgsign", "true"]);
    t.f.git(&["config", "user.signingkey", fpr]);
    t.f.git(&["config", "gpg.program", Path::new(&gpg).to_str().unwrap()]);
    fs::write(t.f.repo.join("file.txt"), b"gpg approved\n").unwrap();
    let mut s = t.service();
    t.trust(&mut s);
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert_eq!(t.blob("HEAD", "file.txt"), b"gpg approved\n");
    let verify = Command::new("git")
        .current_dir(&t.f.repo)
        .env("GNUPGHOME", &t.home)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["verify-commit", "--raw", "HEAD"])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stderr).contains(fpr));
}

#[cfg(feature = "write-test-support")]
fn crash_fixture(phase: Phase) -> (M2, String) {
    use std::os::unix::process::ExitStatusExt;
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"crash approved\n").unwrap();
    t.hook("post-commit", "printf ran >> counter");
    let mut s = t.service();
    t.trust(&mut s);
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    drop(s);
    let input = t.f.root.join("child.json");
    fs::write(
        &input,
        serde_json::to_vec(&(
            t.state.clone(),
            t.options.global_config.clone(),
            t.home.clone(),
            p.clone(),
            phase,
        ))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "crash_worker", "--nocapture"])
        .env("GITFLEET_M2_CHILD", &input)
        .output()
        .unwrap();
    assert_eq!(
        output.status.signal(),
        Some(9),
        "{phase:?}: status={:?} {} {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (t, p.operation_id)
}
#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "child-process crash worker; invoked and asserted by crash matrix, never a skipped acceptance scenario"]
fn crash_worker() {
    let path = std::env::var_os("GITFLEET_M2_CHILD")
        .expect("worker requires parent-owned disposable fixture");
    let (state, global, home, p, phase): (PathBuf, Option<PathBuf>, PathBuf, Proposal, Phase) =
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut s = ControlledWrites::open(
        &state,
        Options {
            global_config: global,
            signer_home: Some(home),
            ..Options::default()
        },
    )
    .unwrap();
    s.set_test_fault(TestFault {
        at: phase,
        kind: TestFaultKind::Sigkill,
    });
    let a = s.approve(&p).unwrap();
    let out = s.execute(&a).unwrap();
    panic!("did not crash: {out:?}");
}
#[cfg(feature = "write-test-support")]
#[test]
fn sigkill_matrix_recovers_from_actual_ref_index_and_owned_locks_without_replay() {
    for phase in [
        Phase::Preparing,
        Phase::HooksRunning,
        Phase::Revalidated,
        Phase::ObjectReady,
        Phase::IndexLockWritten,
        Phase::PublicationPending,
        Phase::ReferenceUpdated,
        Phase::Published,
        Phase::HeadLogWritten,
        Phase::IndexRenamed,
        Phase::IndexReconciled,
        Phase::PostProcessing,
        Phase::Complete,
    ] {
        let (t, id) = crash_fixture(phase);
        let mut s = t.service();
        let review = s.inspect_recovery(&id).unwrap();
        assert_ne!(review.action, "manual", "{phase:?}: {review:?}");
        let out = s.recover(&review).unwrap();
        let published = matches!(
            phase,
            Phase::ReferenceUpdated
                | Phase::Published
                | Phase::IndexRenamed
                | Phase::HeadLogWritten
                | Phase::IndexReconciled
                | Phase::PostProcessing
                | Phase::Complete
        );
        assert_eq!(out.published, published, "{phase:?}: {out:?}");
        assert_eq!(
            t.f.git(&["rev-list", "--count", "HEAD"]),
            if published { b"2\n" } else { b"1\n" }
        );
        assert_eq!(
            t.blob("HEAD", "file.txt"),
            if published {
                b"crash approved\n".as_slice()
            } else {
                b"original\n".as_slice()
            }
        );
        assert!(!t.f.repo.join(".git/index.lock").exists(), "{phase:?}");
        assert!(!t.f.repo.join(".git/HEAD.lock").exists(), "{phase:?}");
        assert_eq!(
            t.f.repo.join("counter").exists(),
            phase == Phase::Complete,
            "{phase:?}"
        );
        if published {
            assert!(t
                .f
                .git(&["reflog", "show", "--format=%gs", "HEAD"])
                .windows(id.len())
                .any(|v| v == id.as_bytes()));
        }
    }
}
#[cfg(feature = "write-test-support")]
#[test]
fn recovery_refuses_foreign_index_lock_ref_and_partial_reflog_without_writes() {
    for change in [
        "index", "lock", "headlock", "ref", "log", "artifact", "revoked",
    ] {
        let (t, id) = crash_fixture(Phase::Published);
        match change {
            "index" => {
                fs::write(t.f.repo.join(".git/index"), b"foreign index").unwrap();
            }
            "lock" => {
                fs::remove_file(t.f.repo.join(".git/index.lock")).unwrap();
                fs::write(t.f.repo.join(".git/index.lock"), b"foreign lock").unwrap();
            }
            "headlock" => {
                fs::remove_file(t.f.repo.join(".git/HEAD.lock")).unwrap();
                fs::write(t.f.repo.join(".git/HEAD.lock"), b"foreign headlock").unwrap();
            }
            "ref" => {
                let old = t.f.git(&["rev-parse", "HEAD^"]);
                fs::write(t.f.repo.join(".git/refs/heads/main"), old).unwrap();
            }
            "log" => {
                let mut data = fs::read(t.f.repo.join(".git/logs/HEAD")).unwrap();
                data.extend(b"partial");
                fs::write(t.f.repo.join(".git/logs/HEAD"), data).unwrap();
            }
            "artifact" => {
                fs::write(
                    t.state
                        .join("operations")
                        .join(&id)
                        .join("reconciled.index"),
                    b"corrupt",
                )
                .unwrap();
            }
            _ => {}
        }
        let mut s = t.service();
        if change == "revoked" {
            s.revoke_trust(&t.worktree).unwrap();
        }
        let before = tree_bytes(&t.f.repo);
        if let Ok(review) = s.inspect_recovery(&id) {
            if let Ok(out) = s.recover(&review) {
                assert_eq!(
                    out.phase,
                    Phase::NeedsManualReconciliation,
                    "{change}: {out:?}"
                );
            }
        }
        assert_eq!(before, tree_bytes(&t.f.repo), "{change}");
    }
}
#[cfg(feature = "write-test-support")]
#[test]
fn storage_failure_and_corrupt_candidate_cannot_publish_unapproved_content() {
    for (phase, kind) in [
        (Phase::ObjectReady, TestFaultKind::CorruptCandidate),
        (Phase::IndexLockWritten, TestFaultKind::StorageFailure),
        (Phase::Published, TestFaultKind::StorageFailure),
        (Phase::HeadLogWritten, TestFaultKind::StorageFailure),
    ] {
        let t = M2::new();
        fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
        let mut s = t.service();
        t.trust(&mut s);
        s.set_test_fault(TestFault { at: phase, kind });
        let out = t.execute(
            &mut s,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        );
        assert_ne!(out.phase, Phase::Complete);
        assert_eq!(
            out.published,
            matches!(phase, Phase::Published | Phase::HeadLogWritten)
        );
        if out.published {
            let r = s.inspect_recovery(&out.operation_id).unwrap();
            let recovered = s.recover(&r).unwrap();
            assert_eq!(recovered.phase, Phase::Complete, "{recovered:?}");
            assert_eq!(t.blob("HEAD", "file.txt"), b"approved\n");
        }
    }
}

#[test]
fn linked_worktrees_require_independent_trust_and_preserve_other_checkout() {
    let t = M2::new();
    let linked = t.f.root.join("linked");
    t.f.git(&["worktree", "add", "-b", "linked", linked.to_str().unwrap()]);
    let before = tree_bytes(&linked);
    let mut ws = WorkspaceService::open(&t.state).unwrap();
    let w = ws.create("linked", &linked).unwrap().id;
    let id = ws.scan(&w).unwrap().entries.remove(0).id;
    drop(ws);
    let mut s = t.service();
    t.trust(&mut s);
    fs::write(linked.join("file.txt"), b"linked\n").unwrap();
    let request = t.request(
        &[("file.txt", ContentSource::WorkingTree)],
        HookPolicy::Execute,
    );
    assert_eq!(
        s.propose(&w, &id, request.clone()).unwrap_err().kind,
        ErrorKind::UntrustedRepository
    );
    let r = s.inspect_trust(&w, &id).unwrap();
    s.grant_trust(&r).unwrap();
    let p = s.propose(&w, &id, request).unwrap();
    let a = s.approve(&p).unwrap();
    let out = s.execute(&a).unwrap();
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert_eq!(t.blob("HEAD", "file.txt"), b"original\n");
    assert_eq!(fs::read(linked.join("file.txt")).unwrap(), b"linked\n");
    assert_eq!(
        fs::read(linked.join(".git")).unwrap(),
        before.get(Path::new(".git")).unwrap().1
    );
    drop(s);
    let conflicting = t.f.root.join("conflicting");
    t.f.git(&[
        "worktree",
        "add",
        "--force",
        conflicting.to_str().unwrap(),
        "main",
    ]);
    let before = tree_bytes(&t.f.repo);
    let mut s = t.service();
    assert_eq!(
        s.inspect_trust(&t.workspace, &t.worktree).unwrap_err().kind,
        ErrorKind::UnsupportedRepositoryState
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[test]
fn supported_sha256_unborn_stage_unstage_and_commit() {
    let t = M2::with_initial(false);
    // Recreate this disposable fixture in SHA-256 format before discovering its new identity.
    fs::remove_dir_all(t.f.repo.join(".git")).unwrap();
    t.f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "--object-format=sha256",
        "-q",
        "-b",
        "main",
    ]);
    for (k, v) in [
        ("user.name", "M2 Fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        t.f.git(&["config", k, v]);
    }
    let mut ws = WorkspaceService::open(&t.state).unwrap();
    let id = ws
        .scan(&t.workspace)
        .unwrap()
        .entries
        .into_iter()
        .find(|e| {
            matches!(
                e.availability,
                gitfleet_core::workspace::Availability::Available
            )
        })
        .unwrap()
        .id;
    drop(ws);
    let mut s = t.service();
    let r = s.inspect_trust(&t.workspace, &id).unwrap();
    s.grant_trust(&r).unwrap();
    fs::write(t.f.repo.join("new"), b"sha256\n").unwrap();
    for request in [
        WriteRequest::Stage {
            paths: vec![NativePath::from_path(Path::new("new"))],
            hooks: HookPolicy::Execute,
        },
        WriteRequest::Unstage {
            paths: vec![NativePath::from_path(Path::new("new"))],
            hooks: HookPolicy::Execute,
        },
        t.request(&[("new", ContentSource::WorkingTree)], HookPolicy::Execute),
    ] {
        let p = s.propose(&t.workspace, &id, request).unwrap();
        let a = s.approve(&p).unwrap();
        let out = s.execute(&a).unwrap();
        assert_eq!(out.phase, Phase::Complete, "{out:?}");
        if out.published {
            assert_eq!(out.new_head.unwrap().len(), 64);
        }
    }
    assert_eq!(t.blob("HEAD", "new"), b"sha256\n");
}
#[test]
fn hook_timeout_is_bounded_and_shared_repository_lock_prevents_execution() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    t.hook("pre-commit", "sleep 30");
    let mut options = t.options.clone();
    options.process.timeout = std::time::Duration::from_millis(1000);
    let mut s = ControlledWrites::open(&t.state, options).unwrap();
    t.trust(&mut s);
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    let a = s.approve(&p).unwrap();
    let common = fs::File::open(t.f.repo.join(".git")).unwrap();
    common.try_lock().unwrap();
    assert_eq!(s.execute(&a).unwrap_err().kind, ErrorKind::Busy);
    drop(common);
    let start = std::time::Instant::now();
    let out = s.execute(&a).unwrap();
    assert_eq!(out.phase, Phase::Blocked, "{out:?}");
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
    assert!(!out.published);
}

#[test]
fn mixed_sources_rename_gitlinks_and_worktree_config_preserve_exact_selection() {
    let t = M2::new();
    fs::write(t.f.repo.join("index-only"), b"staged\n").unwrap();
    t.f.git(&["add", "index-only"]);
    fs::write(t.f.repo.join("index-only"), b"unstaged\n").unwrap();
    fs::rename(t.f.repo.join("file.txt"), t.f.repo.join("renamed file")).unwrap();
    let oid = String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    t.f.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{oid},child"),
    ]);
    t.f.git(&["config", "extensions.worktreeConfig", "true"]);
    t.f.git(&["config", "--worktree", "user.name", "Worktree Identity"]);
    let mut s = t.service();
    t.trust(&mut s);
    let p = s
        .propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[
                    ("index-only", ContentSource::Index),
                    ("file.txt", ContentSource::WorkingTree),
                    ("renamed file", ContentSource::WorkingTree),
                    ("child", ContentSource::Index),
                ],
                HookPolicy::Execute,
            ),
        )
        .unwrap();
    assert_eq!(p.author_name, "Worktree Identity");
    let a = s.approve(&p).unwrap();
    let out = s.execute(&a).unwrap();
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert_eq!(t.blob("HEAD", "index-only"), b"staged\n");
    assert_eq!(
        fs::read(t.f.repo.join("index-only")).unwrap(),
        b"unstaged\n"
    );
    assert_eq!(t.blob("HEAD", "renamed file"), b"original\n");
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", "HEAD:file.txt"])
        .status
        .success());
    assert_eq!(
        String::from_utf8(t.f.git(&["ls-tree", "HEAD", "child"])).unwrap(),
        format!("160000 commit {oid}\tchild\n")
    );
}
#[test]
fn effective_global_hook_path_and_prepare_message_rewrites_are_reviewed() {
    let t = M2::new();
    let directory = t.f.root.join("global-hooks");
    fs::create_dir(&directory).unwrap();
    let hook = directory.join("prepare-commit-msg");
    fs::write(&hook, b"#!/bin/sh\nprintf rewrite > \"$1\"\n").unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        t.options.global_config.as_ref().unwrap(),
        format!("[core]\n hooksPath = {}\n", directory.display()),
    )
    .unwrap();
    fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
    let mut s = t.service();
    let r = s.inspect_trust(&t.workspace, &t.worktree).unwrap();
    assert_eq!(r.hooks.len(), 1);
    assert_eq!(
        r.hooks[0].resolved.to_path().unwrap(),
        fs::canonicalize(hook).unwrap()
    );
    assert!(r.config_origins.len() >= 2);
    s.grant_trust(&r).unwrap();
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert_eq!(out.phase, Phase::NeedsReauthorization);
    assert!(!out.published);
}
#[test]
fn m2_schema_upgrade_backs_up_m1_and_default_services_can_reopen_schema_two() {
    let t = M2::new();
    let connection = rusqlite::Connection::open(t.state.join("state.db")).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
    drop(connection);
    let mut s = t.service();
    let r = s.inspect_trust(&t.workspace, &t.worktree).unwrap();
    s.grant_trust(&r).unwrap();
    drop(s);
    let backup = fs::read_dir(&t.state)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("state.pre-m2-")
        })
        .unwrap();
    let connection = rusqlite::Connection::open(backup).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
    drop(connection);
    let ws = WorkspaceService::open(&t.state).unwrap();
    assert_eq!(ws.list().len(), 1);
    drop(ws);
    let mut s = t.service();
    assert!(s.inspect_trust(&t.workspace, &t.worktree).is_ok());
}

#[test]
fn changed_selection_index_and_ref_invalidate_approval_and_noop_is_rejected() {
    for change in ["worktree", "index", "ref"] {
        let t = M2::new();
        fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
        let mut s = t.service();
        t.trust(&mut s);
        let p = s
            .propose(
                &t.workspace,
                &t.worktree,
                t.request(
                    &[("file.txt", ContentSource::WorkingTree)],
                    HookPolicy::Execute,
                ),
            )
            .unwrap();
        let a = s.approve(&p).unwrap();
        match change {
            "worktree" => fs::write(t.f.repo.join("file.txt"), b"external\n").unwrap(),
            "index" => {
                fs::write(t.f.repo.join("excluded"), b"external\n").unwrap();
                t.f.git(&["add", "excluded"]);
            }
            _ => {
                t.f.git(&[
                    "-c",
                    "core.hooksPath=",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "external",
                ]);
            }
        }
        let before = tree_bytes(&t.f.repo);
        let out = s.execute(&a).unwrap();
        assert_eq!(out.phase, Phase::NeedsReauthorization, "{change}: {out:?}");
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
    let t = M2::new();
    let mut s = t.service();
    t.trust(&mut s);
    let before = tree_bytes(&t.f.repo);
    assert_eq!(
        s.propose(
            &t.workspace,
            &t.worktree,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute
            )
        )
        .unwrap_err()
        .kind,
        ErrorKind::InvalidRequest
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[cfg(feature = "write-test-support")]
#[test]
fn recovery_review_is_session_owned_stale_and_post_attempt_uncertainty_survives_retry() {
    let (t, id) = crash_fixture(Phase::PostProcessing);
    let mut s = t.service();
    let mut r = s.inspect_recovery(&id).unwrap();
    r.action = "abort_unpublished".into();
    assert_eq!(s.recover(&r).unwrap_err().kind, ErrorKind::StaleReview);
    let r = s.inspect_recovery(&id).unwrap();
    let out = s.recover(&r).unwrap();
    assert_eq!(out.phase, Phase::UncertainOutcome);
    let r = s.inspect_recovery(&id).unwrap();
    assert_eq!(s.recover(&r).unwrap().phase, Phase::UncertainOutcome);
    assert!(!t.f.repo.join("counter").exists());
    let (t, id) = crash_fixture(Phase::Published);
    let mut s = t.service();
    let r = s.inspect_recovery(&id).unwrap();
    fs::write(t.f.repo.join(".git/HEAD.lock"), b"foreign changed").unwrap();
    let before = tree_bytes(&t.f.repo);
    assert_eq!(s.recover(&r).unwrap_err().kind, ErrorKind::StaleReview);
    assert_eq!(before, tree_bytes(&t.f.repo));
}

#[test]
fn wrong_key_and_timed_out_signers_never_publish_or_fall_back_to_unsigned() {
    for timeout in [false, true] {
        let t = M2::new();
        let key = generate_ssh(&t);
        let wrong = t.home.join("wrong");
        let out = Command::new("/usr/bin/ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-f"])
            .arg(&wrong)
            .output()
            .unwrap();
        assert!(out.status.success());
        let program = t.home.join("signer");
        let body = if timeout {
            "sleep 30\nexit 1\n".to_owned()
        } else {
            format!("if [ \"$2\" = sign ]; then\n for argument do input=\"$argument\"; done\n exec /usr/bin/ssh-keygen -Y sign -n git -f '{}' \"$input\"\nfi\nexec /usr/bin/ssh-keygen \"$@\"\n",wrong.display())
        };
        fs::write(&program, format!("#!/bin/sh\n{body}")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        for (k, v) in [
            ("commit.gpgsign", "true"),
            ("gpg.format", "ssh"),
            ("user.signingkey", key.to_str().unwrap()),
            ("gpg.ssh.program", program.to_str().unwrap()),
        ] {
            t.f.git(&["config", k, v]);
        }
        fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
        let mut options = t.options.clone();
        if timeout {
            options.process.timeout = std::time::Duration::from_secs(1);
        }
        let mut s = ControlledWrites::open(&t.state, options).unwrap();
        t.trust(&mut s);
        let before = tree_bytes(&t.f.repo);
        let out = t.execute(
            &mut s,
            t.request(
                &[("file.txt", ContentSource::WorkingTree)],
                HookPolicy::Execute,
            ),
        );
        assert_eq!(out.phase, Phase::Blocked, "{out:?}");
        assert!(!out.published);
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
}
#[test]
fn unsupported_sparse_conflicts_detached_and_conversions_never_mutate_git() {
    for setup in [
        "sparse",
        "conflict",
        "detached",
        "attributes",
        "global-attributes",
    ] {
        let t = M2::new();
        match setup {
            "global-attributes" => {
                let attributes = t.f.root.join("attributes");
                fs::write(&attributes, b"file.txt text eol=crlf\n").unwrap();
                fs::write(
                    t.options.global_config.as_ref().unwrap(),
                    format!("[core]\n attributesFile = {}\n", attributes.display()),
                )
                .unwrap();
            }
            "sparse" => {
                t.f.git(&["sparse-checkout", "init", "--cone", "--sparse-index"]);
            }
            "conflict" => {
                let oid = String::from_utf8(t.f.git(&["rev-parse", "HEAD:file.txt"])).unwrap();
                let record = format!("100644 {} 1\tfile.txt\n", oid.trim());
                let data = t.f.root.join("entries");
                fs::write(&data, record).unwrap();
                let out = Command::new("/bin/sh")
                    .current_dir(&t.f.repo)
                    .arg("-c")
                    .arg("git -c core.hooksPath= update-index --index-info < ../entries")
                    .output()
                    .unwrap();
                assert!(out.status.success());
            }
            "detached" => {
                t.f.git(&["switch", "--detach"]);
            }
            _ => {
                fs::write(t.f.repo.join(".gitattributes"), b"file.txt text eol=crlf\n").unwrap();
            }
        }
        fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
        let before = tree_bytes(&t.f.repo);
        let mut s = t.service();
        if setup == "attributes" {
            t.trust(&mut s);
            assert!(s
                .propose(
                    &t.workspace,
                    &t.worktree,
                    t.request(
                        &[("file.txt", ContentSource::WorkingTree)],
                        HookPolicy::Execute
                    )
                )
                .is_err());
        } else {
            assert!(
                s.inspect_trust(&t.workspace, &t.worktree).is_err(),
                "{setup}"
            );
        }
        assert_eq!(before, tree_bytes(&t.f.repo), "{setup}");
    }
}

#[cfg(feature = "write-test-support")]
#[test]
fn revoked_crash_can_be_explicitly_retrusted_with_only_proven_native_locks() {
    let (t, id) = crash_fixture(Phase::Published);
    let mut s = t.service();
    s.revoke_trust(&t.worktree).unwrap();
    let r = s.inspect_recovery(&id).unwrap();
    assert_eq!(
        s.recover(&r).unwrap_err().kind,
        ErrorKind::UntrustedRepository
    );
    t.trust(&mut s);
    let r = s.inspect_recovery(&id).unwrap();
    let out = s.recover(&r).unwrap();
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert_eq!(t.blob("HEAD", "file.txt"), b"crash approved\n");
    assert_eq!(t.f.git(&["rev-list", "--count", "HEAD"]), b"2\n");
}

#[test]
fn moved_replaced_and_permission_changed_metadata_never_inherit_write_authority() {
    for change in ["move", "replace", "permissions"] {
        let t = M2::new();
        fs::write(t.f.repo.join("file.txt"), b"selected\n").unwrap();
        let mut s = t.service();
        t.trust(&mut s);
        drop(s);
        let mut root = t.f.repo.clone();
        match change {
            "move" => {
                root = t.f.root.join("moved");
                fs::rename(&t.f.repo, &root).unwrap();
            }
            "replace" => {
                fs::rename(t.f.repo.join(".git"), t.f.root.join("old.git")).unwrap();
                t.f.git(&["-c", "init.templateDir=", "init", "-q", "-b", "main"]);
                for (k, v) in [
                    ("user.name", "M2 Fixture"),
                    ("user.email", "fixture@example.invalid"),
                    ("commit.gpgsign", "false"),
                ] {
                    t.f.git(&["config", k, v]);
                }
            }
            _ => {
                fs::set_permissions(t.f.repo.join(".git"), fs::Permissions::from_mode(0o700))
                    .unwrap();
            }
        }
        let mut ws = WorkspaceService::open(&t.state).unwrap();
        if change == "move" {
            ws.update(&t.workspace, None, Some(&root), None).unwrap();
        }
        let id = ws
            .scan(&t.workspace)
            .unwrap()
            .entries
            .into_iter()
            .find(|e| {
                matches!(
                    e.availability,
                    gitfleet_core::workspace::Availability::Available
                )
            })
            .unwrap()
            .id;
        drop(ws);
        let before = tree_bytes(&root);
        let mut s = t.service();
        assert_eq!(
            s.propose(
                &t.workspace,
                &id,
                t.request(
                    &[("file.txt", ContentSource::WorkingTree)],
                    HookPolicy::Execute
                )
            )
            .unwrap_err()
            .kind,
            ErrorKind::UntrustedRepository,
            "{change}"
        );
        assert_eq!(before, tree_bytes(&root), "{change}");
    }
}

#[test]
fn preparatory_hooks_can_read_selected_private_blobs_and_rejected_adds_stay_private() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved private\n").unwrap();
    t.hook("pre-commit", "git show :file.txt > observed");
    let mut s = t.service();
    t.trust(&mut s);
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert_eq!(out.phase, Phase::Complete, "{out:?}");
    assert_eq!(
        fs::read(t.f.repo.join("observed")).unwrap(),
        b"approved private\n"
    );
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", "HEAD:observed"])
        .status
        .success());
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved private\n").unwrap();
    t.hook("pre-commit", "printf injected > extra; git add -- extra");
    let before = tree_bytes(&t.f.repo.join(".git/objects"));
    let mut s = t.service();
    t.trust(&mut s);
    let out = t.execute(
        &mut s,
        t.request(
            &[("file.txt", ContentSource::WorkingTree)],
            HookPolicy::Execute,
        ),
    );
    assert!(!out.published);
    assert_eq!(before, tree_bytes(&t.f.repo.join(".git/objects")));
}

#[test]
fn unresolved_fetch_import_blocks_new_and_already_approved_local_writes_in_every_feature_build() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    let mut service = t.service();
    t.trust(&mut service);
    let request = t.request(
        &[("file.txt", ContentSource::WorkingTree)],
        HookPolicy::Execute,
    );
    let proposal = service
        .propose(&t.workspace, &t.worktree, request.clone())
        .unwrap();
    let approval = service.approve(&proposal).unwrap();
    let metadata = service
        .inspect_trust(&t.workspace, &t.worktree)
        .unwrap()
        .metadata;
    // Synthetic interrupted journal; no import is executed by this fixture.
    let db = rusqlite::Connection::open(t.state.join("state.db")).unwrap();
    let record = serde_json::json!({"value":{"plan":{"metadata":{"common_binding":metadata.common_binding}},"phase":"uncertain"}});
    db.execute(
        "INSERT INTO state(key,value) VALUES('fetch-import:test-interruption',?1)",
        [record.to_string()],
    )
    .unwrap();
    let before = tree_bytes(&t.f.repo);
    assert_eq!(
        service
            .propose(&t.workspace, &t.worktree, request)
            .unwrap_err()
            .kind,
        ErrorKind::Busy
    );
    assert_eq!(
        service.execute(&approval).unwrap_err().kind,
        ErrorKind::Busy
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    db.execute(
        "DELETE FROM state WHERE key='fetch-import:test-interruption'",
        [],
    )
    .unwrap();
    assert_eq!(service.execute(&approval).unwrap().phase, Phase::Complete);
}

#[test]
fn upstream_recovery_barrier_blocks_controlled_only_new_and_approved_local_writes() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    let mut service = t.service();
    t.trust(&mut service);
    let request = t.request(
        &[("file.txt", ContentSource::WorkingTree)],
        HookPolicy::Execute,
    );
    let proposal = service
        .propose(&t.workspace, &t.worktree, request.clone())
        .unwrap();
    let approval = service.approve(&proposal).unwrap();
    let metadata = service
        .inspect_trust(&t.workspace, &t.worktree)
        .unwrap()
        .metadata;
    let db = rusqlite::Connection::open(t.state.join("state.db")).unwrap();
    let record = serde_json::json!({"value":{"plan":{"metadata":{"common_binding":metadata.common_binding}},"phase":"lock_pending"}});
    db.execute(
        "INSERT INTO state(key,value) VALUES('upstream:test-interruption',?1)",
        [record.to_string()],
    )
    .unwrap();
    let before = tree_bytes(&t.f.repo);
    assert_eq!(
        service
            .propose(&t.workspace, &t.worktree, request)
            .unwrap_err()
            .kind,
        ErrorKind::Busy
    );
    assert_eq!(
        service.execute(&approval).unwrap_err().kind,
        ErrorKind::Busy
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    db.execute(
        "DELETE FROM state WHERE key='upstream:test-interruption'",
        [],
    )
    .unwrap();
    assert_eq!(service.execute(&approval).unwrap().phase, Phase::Complete);
}

#[test]
fn version_action_recovery_barrier_blocks_new_and_approved_writes_without_application_features() {
    let t = M2::new();
    fs::write(t.f.repo.join("file.txt"), b"approved\n").unwrap();
    let mut service = t.service();
    t.trust(&mut service);
    let request = t.request(
        &[("file.txt", ContentSource::WorkingTree)],
        HookPolicy::Execute,
    );
    let proposal = service
        .propose(&t.workspace, &t.worktree, request.clone())
        .unwrap();
    let approval = service.approve(&proposal).unwrap();
    let metadata = service
        .inspect_trust(&t.workspace, &t.worktree)
        .unwrap()
        .metadata;
    let db = rusqlite::Connection::open(t.state.join("state.db")).unwrap();
    let before = tree_bytes(&t.f.repo);
    // Synthetic recovery metadata only; this fixture executes no Action.
    for phase in ["preparing", "replacing", "unconfirmed"] {
        let raw=serde_json::json!({"value":{"plan":{"metadata":{"common_binding":metadata.common_binding}},"phase":phase}}).to_string();
        db.execute(
            "INSERT OR REPLACE INTO state(key,value) VALUES('version-action:test-interruption',?1)",
            [raw],
        )
        .unwrap();
        assert_eq!(
            service
                .propose(&t.workspace, &t.worktree, request.clone())
                .unwrap_err()
                .kind,
            ErrorKind::Busy
        );
        assert_eq!(
            service.execute(&approval).unwrap_err().kind,
            ErrorKind::Busy
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
    db.execute(
        "DELETE FROM state WHERE key='version-action:test-interruption'",
        [],
    )
    .unwrap();
    assert_eq!(service.execute(&approval).unwrap().phase, Phase::Complete);
}
