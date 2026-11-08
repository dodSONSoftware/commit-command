use super::*;
use gitfleet_core::{
    remote::{push::PushPlanningSelection, push_publication::*},
    writes::HookPolicy,
};

fn selection(t: &mut Test) -> PushPlanningSelection {
    let observation = observed(t);
    fs::write(
        t.f.repo.join("publication.txt"),
        b"Approved authenticated source\n",
    )
    .unwrap();
    t.f.git(&["add", "publication.txt"]);
    t.f.git(&["commit", "-qm", "Authenticated publication fixture"]);
    let Data::PushPreview(preview) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.plan.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    PushPlanningSelection {
        worktree: t.worktree.clone(),
        observation: observation.plan.id,
        source_oid: preview.source_oid,
        children: vec![],
    }
}
fn propose(t: &mut Test, selection: PushPlanningSelection) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewPushPublication {
                workspace: t.workspace.clone(),
                selection,
                hooks: HookPolicy::Execute,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
fn finish(t: &mut Test, r: &ReviewDisplay) -> PushPublication {
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap()
}
#[test]
fn verified_https_publication_uses_only_reviewed_get_helper_and_preserves_managed_state() {
    for sha256 in [false, true] {
        let mut t = setup_format(false, sha256);
        t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "config",
            "http.receivepack",
            "true",
        ]);
        let server = tls_fixture(&mut t, "publication");
        configure_https(&mut t, &server, "127.0.0.1");
        let secret = super::https_credential_tests::policy(&mut t);
        trust(&mut t);
        let selected = selection(&mut t);
        let source = selected.source_oid.clone();
        let r = propose(&mut t, selected);
        let before = tree_bytes(&t.f.repo);
        let credentials = fs::read(t.f.root.join("helper policy's files/credential.json")).unwrap();
        let result = finish(&mut t, &r);
        assert_eq!(result.phase, PushPublicationPhase::Complete, "{result:?}");
        assert!(result.published);
        assert_eq!(result.observed_remote_oid, Some(source.clone()));
        assert_eq!(
            String::from_utf8(t.f.git(&[
                "-C",
                t.remote.to_str().unwrap(),
                "rev-parse",
                "refs/heads/main"
            ]))
            .unwrap()
            .trim(),
            source.hex
        );
        assert_eq!(tree_bytes(&t.f.repo), before);
        assert_eq!(
            fs::read(t.f.root.join("helper policy's files/credential.json")).unwrap(),
            credentials
        );
        assert!(fs::read_to_string(t.f.root.join("helper-calls"))
            .unwrap()
            .lines()
            .all(|line| line == "get"));
        let requests = fs::read_to_string(&server.marker).unwrap();
        assert!(requests.contains("git-receive-pack"));
        super::https_credential_tests::assert_private(&t, &secret);
    }
}
#[test]
fn https_publication_rejects_wrong_credentials_before_transport_without_secret_persistence() {
    let mut t = setup(false);
    t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "config",
        "http.receivepack",
        "true",
    ]);
    let server = tls_fixture(&mut t, "publication");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = super::https_credential_tests::policy(&mut t);
    trust(&mut t);
    let selected = selection(&mut t);
    let r = propose(&mut t, selected);
    let path = t.f.root.join("helper policy's files/credential.json");
    let mut credential: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    credential["password"] = serde_json::json!("fixture-wrong-password");
    fs::write(&path, serde_json::to_vec(&credential).unwrap()).unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let result = finish(&mut t, &r);
    assert_eq!(result.phase, PushPublicationPhase::Blocked);
    assert!(!result.may_have_published);
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
    assert!(fs::read_to_string(t.f.root.join("helper-calls"))
        .unwrap()
        .lines()
        .all(|line| line == "get"));
    super::https_credential_tests::assert_private(&t, &secret);
    super::https_credential_tests::assert_private(&t, "fixture-wrong-password");
}
#[test]
fn ssh_publication_uses_only_reviewed_agent_identity_and_exact_fixed_server_command() {
    let mut t = setup(false);
    let _server = super::ssh_transport_tests::server(&mut t);
    use std::os::unix::fs::PermissionsExt;
    // The fixture server must not inherit the operator's Git configuration either.
    let forbidden = t.f.root.join("unreviewed-server-hook");
    let hooks = t.f.root.join("unreviewed-server-hooks");
    fs::create_dir(&hooks).unwrap();
    let hook = hooks.join("pre-receive");
    fs::write(
        &hook,
        b"#!/bin/sh\nprintf unreviewed > ../unreviewed-server-hook\nexit 1\n",
    )
    .unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    let global = t.f.root.join("server-parent-global.config");
    t.f.git(&[
        "config",
        "--file",
        global.to_str().unwrap(),
        "core.hooksPath",
        hooks.to_str().unwrap(),
    ]);
    let exec = PathBuf::from(String::from_utf8(t.f.git(&["--exec-path"])).unwrap().trim());
    let script = format!(
        r#"import os,shlex,sys
from pathlib import Path
r=Path({:?})
args=shlex.split(os.environ.get('SSH_ORIGINAL_COMMAND',''))
with (r/'ssh-requests').open('a') as f:f.write(str(args)+'\n')
if len(args)!=2 or args[0] not in ['git-upload-pack','git-receive-pack'] or args[1]!={:?}:sys.exit(2)
# Deliberately taint the parent environment; the actual service gets only this allowlist.
os.environ['GIT_CONFIG_GLOBAL']=str(r/'server-parent-global.config')
program=str(Path({:?})/args[0])
environment=dict(PATH='/usr/bin:/bin:/usr/sbin:/sbin',HOME=str(r/'home'),
 GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL='/dev/null',GIT_EXEC_PATH={:?},
 GIT_TERMINAL_PROMPT='0',LANG='C',GIT_CONFIG_COUNT='4',
 GIT_CONFIG_KEY_0='core.hooksPath',GIT_CONFIG_VALUE_0='',
 GIT_CONFIG_KEY_1='core.fsmonitor',GIT_CONFIG_VALUE_1='false',
 GIT_CONFIG_KEY_2='gc.auto',GIT_CONFIG_VALUE_2='0',
 GIT_CONFIG_KEY_3='maintenance.auto',GIT_CONFIG_VALUE_3='false')
os.execve(program,[program,args[1]],environment)
"#,
        t.f.root.to_str().unwrap(),
        t.remote.to_str().unwrap(),
        exec.to_str().unwrap(),
        exec.to_str().unwrap()
    );
    fs::write(t.f.root.join("serve.py"), script).unwrap();
    trust(&mut t);
    let selected = selection(&mut t);
    let source = selected.source_oid.clone();
    let r = propose(&mut t, selected);
    let before = tree_bytes(&t.f.repo);
    let result = finish(&mut t, &r);
    assert_eq!(result.phase, PushPublicationPhase::Complete, "{result:?}");
    assert!(result.published);
    assert_eq!(result.observed_remote_oid, Some(source.clone()));
    assert_eq!(
        String::from_utf8(t.f.git(&[
            "-C",
            t.remote.to_str().unwrap(),
            "rev-parse",
            "refs/heads/main"
        ]))
        .unwrap()
        .trim(),
        source.hex
    );
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert!(
        !forbidden.exists(),
        "Fixture service inherited unreviewed global hooks"
    );
    assert!(fs::read_to_string(t.f.root.join("ssh-requests"))
        .unwrap()
        .contains("git-receive-pack"));
}

#[test]
fn existing_read_only_https_fixture_mode_never_accepts_receive_pack() {
    let mut t = setup(false);
    t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "config",
        "http.receivepack",
        "true",
    ]);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let selected = selection(&mut t);
    let r = propose(&mut t, selected);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let result = finish(&mut t, &r);
    assert_eq!(result.phase, PushPublicationPhase::Unconfirmed);
    assert!(!result.published);
    assert_eq!(tree_bytes(&t.f.repo), before);
    assert_eq!(tree_bytes(&t.remote), remote);
}
