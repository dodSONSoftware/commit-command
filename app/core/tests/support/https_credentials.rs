use super::*;
use gitfleet_core::remote::HttpsCredentialPolicy;
use std::{
    os::unix::fs::PermissionsExt,
    process::Command,
    time::{Duration, Instant},
};
const ACCOUNT: &str = "fixture's account";
pub(super) fn policy(t: &mut Test) -> String {
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
        home: t.f.root.join("home"),
        helper,
        bridge: env!("CARGO_BIN_EXE_gitfleet-credential").into(),
        account: ACCOUNT.into(),
    });
    reopen(t);
    secret
}
pub(super) fn assert_private(t: &Test, secret: &str) {
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
    for mode in [
        "decline",
        "cancel",
        "helper_changed",
        "helper_home_changed",
        "revoked",
    ] {
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
            "helper_home_changed" => {
                let home = &t.options.remote.https_credentials.as_ref().unwrap().home;
                fs::rename(home, t.f.root.join("previous-helper-home")).unwrap();
                fs::create_dir(home).unwrap();
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
#[test]
fn osxkeychain_password_only_response_and_private_state_extensions_authenticate_without_persistence(
) {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let secret = policy(&mut t);
    let helper = &t.options.remote.https_credentials.as_ref().unwrap().helper;
    let script=fs::read_to_string(helper).unwrap().replace("print('username='+value['account'])\n","").replace("print()\n","print('capability[]=state')\nprint('state[]=osxkeychain:seen=__password='+value['password'])\n");
    fs::write(helper, script).unwrap();
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        result.phase,
        ObservationPhase::Complete,
        "{:?}",
        result.issue
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_private(&t, &secret);
}

#[test]
#[ignore = "opt-in loopback Gitea + exact generated macOS Keychain fixture; never uses user credentials"]
fn actual_gitea_and_osxkeychain_read_download_import_and_missing_account() {
    let file = PathBuf::from(
        std::env::var_os("GITFLEET_GITEA_FIXTURE").expect("explicit fixture metadata"),
    );
    let meta: serde_json::Value = serde_json::from_slice(&fs::read(file).unwrap()).unwrap();
    let root = PathBuf::from(meta["root"].as_str().unwrap());
    assert!(root.join(".gitfleet-gitea-fixture").is_file());
    assert!(root.join("keychain-owned.json").is_file());
    let url = meta["url"].as_str().unwrap();
    assert!(url.starts_with("https://127.0.0.1:"));
    let account = meta["account"].as_str().unwrap();
    assert!(account.starts_with("gitfleet-fixture-"));
    let credential: serde_json::Value =
        serde_json::from_slice(&fs::read(meta["credential_file"].as_str().unwrap()).unwrap())
            .unwrap();
    let secret = credential["password"].as_str().unwrap();
    let mut t = setup(false);
    t.f.git(&["remote", "set-url", "origin", url]);
    t.options.remote.ca_file = Some(meta["ca_file"].as_str().unwrap().into());
    t.options.remote.https_credentials = Some(HttpsCredentialPolicy {
        home: std::env::var_os("HOME").unwrap().into(),
        helper: meta["helper"].as_str().unwrap().into(),
        bridge: env!("CARGO_BIN_EXE_gitfleet-credential").into(),
        account: account.into(),
    });
    reopen(&mut t);
    trust(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    let observed = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        observed.phase,
        ObservationPhase::Complete,
        "{:?}",
        observed.issue
    );
    assert_eq!(
        observed.oid.as_ref().unwrap().hex,
        meta["target_oid"].as_str().unwrap()
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
    let r = import_review(&mut t, &capture.plan.transport.id);
    let imported = import_confirm(&mut t, &r);
    assert_eq!(
        imported.phase,
        gitfleet_core::remote::fetch::FetchImportPhase::Complete,
        "{:?}",
        imported.issue
    );
    assert_private(&t, secret);
    t.options.remote.https_credentials.as_mut().unwrap().account =
        "gitfleet-fixture-no-such-account".into();
    reopen(&mut t);
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_private(&t, secret);
}

#[cfg(target_os = "macos")]
#[test]
fn isolated_os_keychain_locked_missing_and_cancelled_reads_preserve_search_list_and_credentials() {
    use std::io::Write;
    struct FixtureKeychain {
        helper: PathBuf,
        before: Vec<u8>,
    }
    impl FixtureKeychain {
        fn command(&self, op: &str, input: &[u8]) -> std::process::Output {
            use std::process::Stdio;
            let mut c = Command::new(&self.helper)
                .arg(op)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            c.stdin.take().unwrap().write_all(input).unwrap();
            c.wait_with_output().unwrap()
        }
    }
    impl Drop for FixtureKeychain {
        fn drop(&mut self) {
            let _ = Command::new(&self.helper)
                .arg("delete")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let password = policy(&mut t);
    let root = t.f.root.join("private-keychain");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    for (name, data) in [
        ("credential-password", password.clone()),
        ("keychain-password", uuid::Uuid::new_v4().to_string()),
    ] {
        let file = root.join(name);
        fs::write(&file, data).unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let source = root.join("helper.c");
    fs::write(&source, include_str!("keychain_fixture.c")).unwrap();
    let helper = root.join("keychain-helper");
    let compiled = Command::new("cc")
        .args([
            "-Wno-deprecated-declarations",
            "-framework",
            "Security",
            "-framework",
            "CoreFoundation",
        ])
        .arg(format!("-DFIXTURE_ROOT=\"{}\"", root.display()))
        .arg(&source)
        .arg("-o")
        .arg(&helper)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "Fixture helper compile failed");
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    let before = Command::new(&helper).arg("searchlist").output().unwrap();
    assert!(before.status.success());
    let kc = FixtureKeychain {
        helper: helper.clone(),
        before: before.stdout,
    };
    let frame = format!(
        "protocol=https\nhost=127.0.0.1:{}\npath=repo.git\nusername={}\n\n",
        server.port, ACCOUNT
    );
    assert!(
        kc.command("seed", frame.as_bytes()).status.success(),
        "Private keychain creation failed"
    );
    assert!(
        kc.before == kc.command("searchlist", b"").stdout,
        "Private keychain changed global search list"
    );
    t.options.remote.https_credentials.as_mut().unwrap().helper = helper;
    reopen(&mut t);
    trust(&mut t);
    let baseline = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    assert_eq!(
        confirm(&mut t, &r, Cancellation::default()).phase,
        ObservationPhase::Complete
    );
    assert!(kc.command("lock", b"").status.success());
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    let status = fs::read_to_string(root.join("last-status")).unwrap();
    assert!(
        matches!(status.as_str(), "-25308" | "-25293"),
        "Unexpected locked keychain result: {status}"
    );
    assert!(kc.command("unlock", b"").status.success());
    t.options.remote.https_credentials.as_mut().unwrap().account =
        "not-the-selected-account".into();
    reopen(&mut t);
    let r = review(&mut t, "refs/heads/main");
    assert_eq!(
        confirm(&mut t, &r, Cancellation::default()).phase,
        ObservationPhase::Blocked
    );
    t.options.remote.https_credentials.as_mut().unwrap().account = ACCOUNT.into();
    reopen(&mut t);
    fs::write(root.join("hold"), b"hold after actual OS lookup").unwrap();
    let r = review(&mut t, "refs/heads/main");
    let token = Cancellation::default();
    let cancel = token.clone();
    let marker = root.join("started");
    let waiter = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(marker.exists());
        cancel.cancel();
    });
    assert_eq!(
        confirm(&mut t, &r, token).phase,
        ObservationPhase::Cancelled
    );
    waiter.join().unwrap();
    fs::remove_file(root.join("hold")).unwrap();
    let output = kc.command("get", frame.as_bytes());
    assert!(output.status.success());
    assert!(
        output
            .stdout
            .windows(password.len())
            .any(|s| s == password.as_bytes()),
        "Fixture credential was changed or erased"
    );
    assert_eq!(baseline, tree_bytes(&t.f.repo));
    assert_private(&t, &password);
    assert!(kc.command("delete", b"").status.success());
    assert!(
        kc.before == kc.command("searchlist", b"").stdout,
        "Search list changed after private fixture cleanup"
    );
}
