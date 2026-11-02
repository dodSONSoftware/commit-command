use super::*;
use gitfleet_core::remote::SshPolicy;
use std::{
    net::{TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Server {
    server: Child,
    agent: Child,
    socket: PathBuf,
    keys: PathBuf,
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.server.kill();
        let _ = self.server.wait();
        let _ = self.agent.kill();
        let _ = self.agent.wait();
    }
}
fn run(command: &mut Command) {
    let o = command.output().unwrap();
    assert!(
        o.status.success(),
        "fixture process failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}
fn server(t: &mut Test) -> Server {
    fs::write(
        t.f.root.join(".gitfleet-ssh-fixture"),
        b"disposable SSH transport test",
    )
    .unwrap();
    let keys = t.f.root.join("SSH policy's files");
    fs::create_dir(&keys).unwrap();
    fs::set_permissions(&keys, fs::Permissions::from_mode(0o700)).unwrap();
    for key in ["host", "client", "other"] {
        run(Command::new("/usr/bin/ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(keys.join(key)));
    }
    // A public-key filename with no private sibling demonstrates agent-only use.
    fs::copy(keys.join("client.pub"), keys.join("selected.pub")).unwrap();
    let socket = t.f.root.join("agent.sock");
    let agent = Command::new("/usr/bin/ssh-agent")
        .args(["-D", "-a"])
        .arg(&socket)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while !socket.exists() {
        assert!(start.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(10));
    }
    for key in ["client", "other"] {
        run(Command::new("/usr/bin/ssh-add")
            .env("SSH_AUTH_SOCK", &socket)
            .arg(keys.join(key)));
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let pubkey = fs::read_to_string(keys.join("host.pub")).unwrap();
    let mut parts = pubkey.split_whitespace();
    let kind = parts.next().unwrap();
    let encoded = parts.next().unwrap();
    fs::write(
        keys.join("known_hosts"),
        format!("[127.0.0.1]:{port} {kind} {encoded}\n"),
    )
    .unwrap();
    let script = t.f.root.join("serve.py");
    let upload = String::from_utf8(t.f.git(&["--exec-path"])).unwrap();
    let upload = PathBuf::from(upload.trim()).join("git-upload-pack");
    fs::write(&script,format!("import os,sys,time,select\nfrom pathlib import Path\nr=Path({:?})\nwith (r/'ssh-requests').open('a') as f:f.write(os.environ.get('SSH_ORIGINAL_COMMAND','')+'\\n')\nwhile (r/'hold-ssh').exists():\n if select.select([sys.stdin],[],[],.02)[0] and not os.read(0,65536):sys.exit(0)\nos.execv({:?},[{:?},{:?}])\n",t.f.root.to_str().unwrap(),upload.to_str().unwrap(),upload.to_str().unwrap(),t.remote.to_str().unwrap())).unwrap();
    let config = t.f.root.join("sshd.conf");
    let user = String::from_utf8(Command::new("id").arg("-un").output().unwrap().stdout).unwrap();
    fs::write(&config,format!("ListenAddress 127.0.0.1\nPort {port}\nHostKey \"{}\"\nPidFile {}\nAuthorizedKeysFile \"{}\"\nAllowUsers {}\nStrictModes yes\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\nAllowAgentForwarding no\nAllowTcpForwarding no\nX11Forwarding no\nPermitTTY no\nForceCommand /usr/bin/python3 {}\nLogLevel VERBOSE\n",keys.join("host").display(),t.f.root.join("sshd.pid").display(),keys.join("client.pub").display(),user.trim(),script.display())).unwrap();
    let log = fs::File::create(t.f.root.join("sshd.log")).unwrap();
    let mut child = Command::new("/usr/sbin/sshd")
        .args(["-D", "-e", "-f"])
        .arg(config)
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let start = Instant::now();
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "{}",
            fs::read_to_string(t.f.root.join("sshd.log")).unwrap()
        );
        assert!(start.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(10));
    }
    t.f.git(&[
        "remote",
        "set-url",
        "origin",
        &format!(
            "ssh://{}@127.0.0.1:{port}{}",
            user.trim(),
            t.remote.display()
        ),
    ]);
    t.options.remote.ssh = Some(SshPolicy {
        known_hosts: keys.join("known_hosts"),
        public_identity: keys.join("selected.pub"),
        agent_socket: socket.clone(),
    });
    reopen(t);
    Server {
        server: child,
        agent,
        socket,
        keys,
    }
}
fn remote_review(t: &mut Test) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                remote: "origin".into(),
                reference: "refs/heads/main".into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
#[test]
fn ssh_agent_authenticates_only_selected_identity_then_downloads_and_imports_offline() {
    let mut t = setup(false);
    let s = server(&mut t);
    let sentinel = t.f.root.join("managed-command-ran");
    let malicious = format!("!touch {}", sentinel.display());
    t.f.git(&["config", "core.sshCommand", &malicious]);
    t.f.git(&["config", "credential.helper", &malicious]);
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let hosts = fs::read(s.keys.join("known_hosts")).unwrap();
    let r = remote_review(&mut t);
    assert!(r.summary.contains("Strict known-hosts"));
    assert!(r.summary.contains("ssh_public_identity"));
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let observation: Observation =
        serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap();
    assert_eq!(
        observation.phase,
        ObservationPhase::Complete,
        "{:?}; {}",
        observation.issue,
        fs::read_to_string(t.f.root.join("sshd.log")).unwrap()
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert!(!sentinel.exists());
    let r = fetch_review(&mut t, &observation.plan.id);
    let captured = fetch_confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        captured.phase,
        gitfleet_core::remote::fetch::FetchCapturePhase::Complete,
        "{:?}",
        captured.issue
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    let r = import_review(&mut t, &captured.plan.transport.id);
    let imported = import_confirm(&mut t, &r);
    assert_eq!(
        imported.phase,
        gitfleet_core::remote::fetch::FetchImportPhase::Complete,
        "{:?}",
        imported.issue
    );
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
    assert_eq!(
        preview.relation,
        gitfleet_core::remote::push::PushRelation::UpToDate
    );
    assert_eq!((preview.outgoing_count, preview.incoming_count), (0, 0));
    assert_eq!(remote, tree_bytes(&t.remote));
    assert_eq!(hosts, fs::read(s.keys.join("known_hosts")).unwrap());
    assert!(!sentinel.exists());
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let raw: String = db
        .query_row("select group_concat(value) from state", [], |r| r.get(0))
        .unwrap();
    assert!(!raw.contains("OPENSSH PRIVATE KEY"));
    assert!(!raw.contains(
        fs::read_to_string(s.keys.join("client.pub"))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
    ));
}
#[test]
fn ssh_unknown_or_changed_host_and_wrong_selected_identity_fail_without_fallback() {
    for mode in ["unknown_host", "changed_host", "wrong_identity"] {
        let mut t = setup(false);
        let s = server(&mut t);
        match mode {
            "unknown_host" => fs::write(s.keys.join("known_hosts"), b"").unwrap(),
            "changed_host" => {
                let other = fs::read_to_string(s.keys.join("other.pub")).unwrap();
                let old = fs::read_to_string(s.keys.join("known_hosts")).unwrap();
                fs::write(
                    s.keys.join("known_hosts"),
                    format!("{} {other}", old.split_whitespace().next().unwrap()),
                )
                .unwrap();
            }
            _ => {
                fs::copy(s.keys.join("other.pub"), s.keys.join("selected.pub")).unwrap();
            }
        }
        trust(&mut t);
        let before = tree_bytes(&t.f.repo);
        let r = remote_review(&mut t);
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        let value: Observation =
            serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap())
                .unwrap();
        assert_eq!(
            value.phase,
            ObservationPhase::Blocked,
            "{mode}: {:?}",
            value.issue
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert!(
            !t.f.root.join("ssh-requests").exists(),
            "{mode}: authenticated unexpectedly"
        );
    }
}
#[test]
fn ssh_policy_drift_decline_and_precancel_never_authenticate() {
    for mode in [
        "known_hosts",
        "identity",
        "agent",
        "decline",
        "cancel",
        "no_policy",
        "private_key",
        "private_swap",
    ] {
        let mut t = setup(false);
        let mut s = server(&mut t);
        if mode == "no_policy" {
            t.options.remote.ssh = None;
            reopen(&mut t);
        }
        if mode == "private_key" {
            t.options.remote.ssh.as_mut().unwrap().public_identity = s.keys.join("client");
            reopen(&mut t);
        }
        trust(&mut t);
        let before = tree_bytes(&t.f.repo);
        if mode == "no_policy" || mode == "private_key" {
            assert!(t
                .app
                .request(
                    t.app.epoch(),
                    Request::ReviewRemoteObservation {
                        workspace: t.workspace.clone(),
                        worktree: t.worktree.clone(),
                        remote: "origin".into(),
                        reference: "refs/heads/main".into()
                    }
                )
                .is_err());
            continue;
        }
        let r = remote_review(&mut t);
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        match mode {
            "known_hosts" => {
                use std::io::Write;
                fs::OpenOptions::new()
                    .append(true)
                    .open(s.keys.join("known_hosts"))
                    .unwrap()
                    .write_all(b"\n")
                    .unwrap();
            }
            "identity" => {
                fs::copy(s.keys.join("other.pub"), s.keys.join("selected.pub")).unwrap();
            }
            "private_swap" => {
                fs::copy(s.keys.join("client"), s.keys.join("selected.pub")).unwrap();
            }
            "agent" => {
                s.agent.kill().unwrap();
                s.agent.wait().unwrap();
                let _ = fs::remove_file(&s.socket);
                s.agent = Command::new("/usr/bin/ssh-agent")
                    .args(["-D", "-a"])
                    .arg(&s.socket)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
                let start = Instant::now();
                while !s.socket.exists() {
                    assert!(start.elapsed() < Duration::from_secs(5));
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            _ => {}
        }
        let token = Cancellation::default();
        if mode == "cancel" {
            token.cancel();
        }
        let result = t.app.confirm(cap, mode != "decline", token);
        if mode == "decline" {
            assert!(result.is_err());
        } else {
            let value: Observation = serde_json::from_value(result.unwrap()).unwrap();
            assert_ne!(value.phase, ObservationPhase::Complete, "{mode}");
        }
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert!(!t.f.root.join("ssh-requests").exists(), "{mode}");
    }
}

#[test]
fn ssh_cancels_proven_authenticated_observation_and_download_without_repository_mutation() {
    for download in [false, true] {
        let mut t = setup(false);
        let _s = server(&mut t);
        trust(&mut t);
        let r = if download {
            let source = observed(&mut t);
            fs::remove_file(t.f.root.join("ssh-requests")).unwrap();
            fetch_review(&mut t, &source.plan.id)
        } else {
            remote_review(&mut t)
        };
        fs::write(t.f.root.join("hold-ssh"), b"hold only this fixture").unwrap();
        let token = Cancellation::default();
        let cancel = token.clone();
        let marker = t.f.root.join("ssh-requests");
        let waiter = std::thread::spawn(move || {
            let start = Instant::now();
            while !marker.exists() {
                if start.elapsed() > Duration::from_secs(10) {
                    cancel.cancel();
                    return false;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            cancel.cancel();
            true
        });
        let before = tree_bytes(&t.f.repo);
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        let result = t.app.confirm(cap, true, token).unwrap();
        assert!(waiter.join().unwrap());
        assert_eq!(result["phase"], "cancelled");
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
}
#[test]
fn ssh_server_failure_diagnostics_are_not_retained_in_results_or_journals() {
    let mut t = setup(false);
    let _s = server(&mut t);
    let script = t.f.root.join("serve.py");
    let code = fs::read_to_string(&script).unwrap().replace(
        "os.execv(",
        "sys.stderr.write('FIXTURE-SSH-SERVER-SECRET');sys.exit(17)\nos.execv(",
    );
    fs::write(script, code).unwrap();
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = remote_review(&mut t);
    let value = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(value.phase, ObservationPhase::Blocked);
    assert!(!serde_json::to_string(&value)
        .unwrap()
        .contains("FIXTURE-SSH-SERVER-SECRET"));
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    let raw: String = db
        .query_row("select group_concat(value) from state", [], |r| r.get(0))
        .unwrap();
    assert!(!raw.contains("FIXTURE-SSH-SERVER-SECRET"));
    assert_eq!(before, tree_bytes(&t.f.repo));
}
