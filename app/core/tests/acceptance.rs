//! One subprocess per scenario keeps test configurations and signing keys isolated.
use std::process::Command;

fn scenario(name: &str) {
    let result = Command::new(env!("CARGO_BIN_EXE_gitfleet-lab"))
        .arg("--case").arg(name).output().expect("run disposable lab");
    assert!(result.status.success(), "{name}: {}", String::from_utf8_lossy(&result.stderr));
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).expect("valid JSON results");
    assert!(response["status"].is_string(), "{name}: missing status");
}

macro_rules! test_case {
    ($($name:ident),+ $(,)?) => { $(#[test] fn $name() { scenario(stringify!($name)); })+ }
}

test_case!(
    selected_staged_preserves_other_staging,
    selected_worktree_preserves_other_staging,
    partial_staging_commits_index_only,
    partial_staging_commits_worktree,
    mixed_selected_sources,
    untracked_selected,
    selected_unstaged_deletion,
    selected_rename_pair,
    space_in_path,
    hook_modifies_only_worktree,
    hook_stages_unapproved,
    hook_rewrites_message,
    post_commit_hook,
    hook_changes_selected_worktree,
    prepare_hook_rewrites_message,
    selected_worktree_changed_after_review,
    external_index_changed,
    stale_head_before_cas,
    real_sigkill_before_ref,
    real_sigkill_after_ref,
    real_sigkill_after_reconcile,
    crash_then_external_index_edit,
    signing_missing_key,
    successful_gpg_signing,
);
