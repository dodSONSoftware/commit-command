use super::*;
use gitfleet_core::remote::HttpsCredentialPolicy;
use std::{
    os::unix::fs::PermissionsExt,
    process::Command,
    time::{Duration, Instant},
};
const ACCOUNT: &str = "fixture's account";
fn policy(t: &mut Test) -> String {
    let secret = format!("fixture-token-{}", uuid::Uuid::new_v4());
    let selected = t.f.root.join("helper policy's files");
    fs::create_dir(&selected).unwrap();
    fs::write(
        t.f.root.join("basic-auth.json"),
        serde_json::to_vec(&serde_json::json!({"account":ACCOUNT,"password":secret})).unwrap(),
    )
    .unwrap();
    fs::write(
        selected.join("credential.json"),
        serde_json::to_vec(&serde_json::json!({"account":ACCOUNT,"password":secret})).unwrap(),
    )
    .unwrap();
    let helper = selected.join("get-only helper.py");
    let script=format!("#!/usr/bin/python3\nimport sys,json,time,os,subprocess\nfrom pathlib import Path\nr=Path({:?})\nwith (r/'helper-calls').open('a') as f:f.write(sys.argv[1]+'\\n')\nrequest=sys.stdin.read()\nif (r/'background-helper').exists():\n child=subprocess.Popen(['/bin/sleep','60'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)\n with (r/'helper-background-pids').open('a') as f:f.write(str(child.pid)+'\\n')\nif (r/'hold-helper').exists():\n (r/'helper-started').write_text(str(os.getpid()))\n time.sleep(60)\nvalue=json.loads((r/\"helper policy's files/credential.json\").read_text())\nprint('username='+value['account'])\nprint('password='+value['password'])\nprint()\n",t.f.root.to_str().unwrap());
    fs::write(&helper, script).unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    t.options.remote.https_credentials = Some(HttpsCredentialPolicy {
        helper,
        bridge: env!("CARGO_BIN_EXE_gitfleet-credential").into(),
        account: ACCOUNT.into(),
    });
    reopen(t);
    secret
}
fn assert_private(t: &Test, secret: &str) {
    fn base64(value: &[u8]) -> Vec<u8> {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = Vec::new();
        for c in value.chunks(3) {
            let a = c[0] as usize;
            let b = *c.get(1).unwrap_or(&0) as usize;
            let d = *c.get(2).unwrap_or(&0) as usize;
            out.extend([
                alphabet[a >> 2],
                alphabet[((a & 3) << 4) | (b >> 4)],
                if c.len() > 1 {
                    alphabet[((b & 15) << 2) | (d >> 6)]
                } else {
                    b'='
                },
                if c.len() > 2 { alphabet[d & 63] } else { b'=' },
            ]);
        }
        out
    }
    fn scan(path: &std::path::Path, secret: &[u8]) {
        for entry in fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            let m = fs::symlink_metadata(&p).unwrap();
            if m.is_dir() {
                scan(&p, secret)
            } else if m.is_file() {
                let bytes = fs::read(&p).unwrap();
                assert!(
                    !bytes.windows(secret.len()).any(|v| v == secret),
                    "Secret persisted in {:?}",
                    p
                );
            }
        }
    }
    scan(&t.f.root.join("state"), secret.as_bytes());
    scan(
        &t.f.root.join("state"),
        &base64(format!("{ACCOUNT}:{secret}").as_bytes()),
    );
}
#[test]
fn approved_https_helper_authenticates_downloads_and_imports_without_persisting_or_mutating_credentials(
) {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = policy(&mut t);
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let r = review(&mut t, "refs/heads/main");
    assert!(r.summary.contains("Only get"));
    assert!(!r.summary.contains(&secret));
    assert!(!r.content.contains(&secret));
    assert!(!t.f.root.join("helper-calls").exists());
    let observed = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        observed.phase,
        ObservationPhase::Complete,
        "{:?}",
        observed.issue
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    let r = fetch_review(&mut t, &observed.plan.id);
    let capture = fetch_confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        capture.phase,
        gitfleet_core::remote::fetch::FetchCapturePhase::Complete,
        "{:?}",
        capture.issue
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    let calls = fs::read_to_string(t.f.root.join("helper-calls")).unwrap();
    assert!(calls.lines().all(|v| v == "get"));
    assert!(calls.lines().count() >= 2);
    let r = import_review(&mut t, &capture.plan.transport.id);
    let result = import_confirm(&mut t, &r);
    assert_eq!(
        result.phase,
        gitfleet_core::remote::fetch::FetchImportPhase::Complete,
        "{:?}",
        result.issue
    );
    assert_eq!(
        calls,
        fs::read_to_string(t.f.root.join("helper-calls")).unwrap()
    );
    assert_eq!(remote, tree_bytes(&t.remote));
    assert_private(&t, &secret);
}
#[test]
fn declined_stale_or_cancelled_https_reviews_never_read_a_credential() {
    for mode in ["decline", "cancel", "helper_changed", "revoked"] {
        let mut t = setup(false);
        let server = tls_fixture(&mut t, "git");
        configure_https(&mut t, &server, "127.0.0.1");
        let secret = policy(&mut t);
        trust(&mut t);
        let r = review(&mut t, "refs/heads/main");
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        match mode {
            "helper_changed" => {
                use std::io::Write;
                fs::OpenOptions::new()
                    .append(true)
                    .open(&t.options.remote.https_credentials.as_ref().unwrap().helper)
                    .unwrap()
                    .write_all(b"\n# changed\n")
                    .unwrap();
            }
            "revoked" => {
                t.app
                    .request(
                        t.app.epoch(),
                        Request::Revoke {
                            workspace: t.workspace.clone(),
                            worktree: t.worktree.clone(),
                        },
                    )
                    .unwrap();
            }
            _ => {}
        }
        let before = tree_bytes(&t.f.repo);
        let token = Cancellation::default();
        if mode == "cancel" {
            token.cancel();
        }
        let result = t.app.confirm(cap, mode != "decline", token);
        if mode == "decline" || mode == "revoked" {
            assert!(result.is_err());
        } else {
            let value: Observation = serde_json::from_value(result.unwrap()).unwrap();
            assert_ne!(value.phase, ObservationPhase::Complete);
        }
        assert!(!t.f.root.join("helper-calls").exists());
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_private(&t, &secret);
    }
}
#[test]
fn wrong_missing_and_changed_account_credentials_fail_without_store_or_erase() {
    for mode in ["wrong_password", "wrong_account", "missing"] {
        let mut t = setup(false);
        let server = tls_fixture(&mut t, "git");
        configure_https(&mut t, &server, "127.0.0.1");
        let secret = policy(&mut t);
        trust(&mut t);
        let r = review(&mut t, "refs/heads/main");
        let file = t.f.root.join("helper policy's files/credential.json");
        let value = match mode {
            "wrong_account" => serde_json::json!({"account":"another-account","password":secret}),
            "wrong_password" => serde_json::json!({"account":ACCOUNT,"password":"wrong-password"}),
            _ => serde_json::json!({"account":ACCOUNT,"password":""}),
        };
        fs::write(file, serde_json::to_vec(&value).unwrap()).unwrap();
        let before = tree_bytes(&t.f.repo);
        let result = confirm(&mut t, &r, Cancellation::default());
        assert_eq!(result.phase, ObservationPhase::Blocked, "{mode}");
        assert!(!serde_json::to_string(&result).unwrap().contains(&secret));
        assert!(fs::read_to_string(t.f.root.join("helper-calls"))
            .unwrap()
            .lines()
            .all(|v| v == "get"));
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_private(&t, &secret);
    }
}
#[test]
fn cancellation_after_credential_lookup_starts_reaps_the_helper_and_preserves_state() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = policy(&mut t);
    trust(&mut t);
    fs::write(t.f.root.join("hold-helper"), b"hold fixture only").unwrap();
    let r = review(&mut t, "refs/heads/main");
    let token = Cancellation::default();
    let cancel = token.clone();
    let marker = t.f.root.join("helper-started");
    let waiter = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        let pid = fs::read_to_string(&marker).ok();
        cancel.cancel();
        pid
    });
    let before = tree_bytes(&t.f.repo);
    let result = confirm(&mut t, &r, token);
    let pid = waiter
        .join()
        .unwrap()
        .expect("Credential helper never started");
    assert_eq!(result.phase, ObservationPhase::Cancelled);
    let status = Command::new("ps")
        .args(["-p", pid.trim(), "-o", "stat="])
        .output()
        .unwrap();
    assert!(status.stdout.is_empty() || status.stdout.starts_with(b"Z"));
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_private(&t, &secret);
}

#[test]
fn completed_https_transport_reaps_helper_background_descendants() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = policy(&mut t);
    fs::write(
        t.f.root.join("background-helper"),
        b"fixture descendants only",
    )
    .unwrap();
    trust(&mut t);
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        result.phase,
        ObservationPhase::Complete,
        "{:?}",
        result.issue
    );
    let pids = fs::read_to_string(t.f.root.join("helper-background-pids")).unwrap();
    for pid in pids.lines() {
        let result = Command::new("ps")
            .args(["-p", pid, "-o", "stat="])
            .output()
            .unwrap();
        assert!(
            result.stdout.is_empty() || result.stdout.starts_with(b"Z"),
            "Helper descendant survived its network operation"
        );
    }
    assert_private(&t, &secret);
}
#[test]
fn malformed_oversized_and_failing_helpers_cannot_expose_a_secret_or_authenticate() {
    for mode in ["duplicate", "oversized", "stderr", "wrong_scope"] {
        let mut t = setup(false);
        let server = tls_fixture(&mut t, "git");
        configure_https(&mut t, &server, "127.0.0.1");
        let secret = policy(&mut t);
        let helper = &t.options.remote.https_credentials.as_ref().unwrap().helper;
        let code = fs::read_to_string(helper).unwrap();
        let suffix = match mode {
            "duplicate" => "print('username=unexpected')\n",
            "oversized" => "print('extra='+'x'*20000)\n",
            "stderr" => "sys.stderr.write(value['password']);sys.exit(17)\n",
            _ => "print('host=other.invalid')\n",
        };
        let code = code.replace(
            "print('username='+value['account'])",
            &format!("{suffix}print('username='+value['account'])"),
        );
        fs::write(helper, code).unwrap();
        reopen(&mut t);
        trust(&mut t);
        let before = tree_bytes(&t.f.repo);
        let r = review(&mut t, "refs/heads/main");
        let result = confirm(&mut t, &r, Cancellation::default());
        assert_eq!(
            result.phase,
            ObservationPhase::Blocked,
            "{mode}: {:?}",
            result.issue
        );
        assert!(!serde_json::to_string(&result).unwrap().contains(&secret));
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_private(&t, &secret);
    }
}
#[test]
fn credential_helper_timeout_is_bounded_by_the_owning_network_operation() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = policy(&mut t);
    t.options.process.timeout = Duration::from_secs(2);
    reopen(&mut t);
    trust(&mut t);
    fs::write(t.f.root.join("hold-helper"), b"fixture only").unwrap();
    let r = review(&mut t, "refs/heads/main");
    let before = tree_bytes(&t.f.repo);
    let start = Instant::now();
    let value = confirm(&mut t, &r, Cancellation::default());
    assert!(start.elapsed() < Duration::from_secs(8));
    assert_eq!(value.phase, ObservationPhase::Blocked);
    assert_eq!(value.issue.unwrap().kind, ErrorKind::Timeout);
    let pid = fs::read_to_string(t.f.root.join("helper-started")).unwrap();
    let result = Command::new("ps")
        .args(["-p", pid.trim(), "-o", "stat="])
        .output()
        .unwrap();
    assert!(result.stdout.is_empty() || result.stdout.starts_with(b"Z"));
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_private(&t, &secret);
}

#[test]
fn helper_credentials_never_bypass_tls_hostname_validation_or_follow_redirects() {
    for mode in ["hostname", "redirect"] {
        let mut t = setup(false);
        let server = tls_fixture(
            &mut t,
            if mode == "redirect" {
                "redirect"
            } else {
                "git"
            },
        );
        configure_https(
            &mut t,
            &server,
            if mode == "hostname" {
                "localhost"
            } else {
                "127.0.0.1"
            },
        );
        let secret = policy(&mut t);
        trust(&mut t);
        let before = tree_bytes(&t.f.repo);
        let r = review(&mut t, "refs/heads/main");
        let value = confirm(&mut t, &r, Cancellation::default());
        assert_eq!(value.phase, ObservationPhase::Blocked);
        let requests = fs::read_to_string(&server.marker).unwrap_or_default();
        if mode == "hostname" {
            assert!(requests.is_empty());
        }
        assert!(!requests.contains("/forbidden"));
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_private(&t, &secret);
    }
}
