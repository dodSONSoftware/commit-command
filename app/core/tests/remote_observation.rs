mod support;
use gitfleet_core::{
    application::*,
    inspection::Cancellation,
    remote::{observation::*, TransportOptions},
    writes::Options,
    ErrorKind,
};
use sha2::Digest;
use std::{fs, path::PathBuf};
use support::{tree_bytes, Fixture};
struct Test {
    f: Fixture,
    app: Application,
    workspace: String,
    worktree: String,
    remote: PathBuf,
    options: Options,
}
fn setup(local: bool) -> Test {
    setup_format(local, false)
}
fn setup_format(local: bool, sha256: bool) -> Test {
    let f = Fixture::new();
    if sha256 {
        fs::remove_dir_all(f.repo.join(".git")).unwrap();
        f.git(&[
            "-c",
            "init.templateDir=",
            "init",
            "-q",
            "-b",
            "main",
            "--object-format=sha256",
        ]);
        f.git(&["config", "user.name", "SHA256 fixture"]);
        f.git(&["config", "user.email", "fixture@example.invalid"]);
        f.git(&["config", "commit.gpgsign", "false"]);
    }
    f.commit_file();
    let remote = f.root.join("remote.git");
    f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "--bare",
        "-q",
        if sha256 {
            "--object-format=sha256"
        } else {
            "--object-format=sha1"
        },
        remote.to_str().unwrap(),
    ]);
    f.git(&[
        "-C",
        remote.to_str().unwrap(),
        "-c",
        "protocol.file.allow=always",
        "fetch",
        "-q",
        f.repo.to_str().unwrap(),
        "refs/heads/main:refs/heads/main",
    ]);
    f.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    let global = f.root.join("global.config");
    fs::write(&global, b"").unwrap();
    let home = f.root.join("home");
    fs::create_dir(&home).unwrap();
    let options = Options {
        global_config: Some(global),
        signer_home: Some(home),
        remote: TransportOptions {
            local_fixture_root: local.then(|| f.root.clone()),
            ca_file: None,
            ssh: None,
        },
        ..Options::default()
    };
    let mut app = Application::open(&f.root.join("state"), options.clone()).unwrap();
    let picker = app.register_directory(&f.repo).unwrap();
    app.request(
        app.epoch(),
        Request::Create {
            name: "Observation".into(),
            picker,
        },
    )
    .unwrap();
    let Data::Workspaces(workspaces) = app
        .request(app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    let workspace = workspaces.active.unwrap();
    let Data::Inventory(inventory) = app
        .request(
            app.epoch(),
            Request::Inventory {
                workspace: workspace.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let worktree = inventory.entries[0].id.0.clone();
    Test {
        f,
        app,
        workspace,
        worktree,
        remote,
        options,
    }
}
fn trust(t: &mut Test) {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewTrust {
                workspace: t.workspace.clone(),
                worktrees: vec![t.worktree.clone()],
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    t.app.confirm(cap, true, Cancellation::default()).unwrap();
}
fn review(t: &mut Test, reference: &str) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewRemoteObservation {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                remote: "origin".into(),
                reference: reference.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
fn observations(t: &mut Test) -> ObservationPage {
    let Data::RemoteObservations(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::RemoteObservations {
                workspace: t.workspace.clone(),
                worktree: Some(t.worktree.clone()),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    page
}
fn confirm(t: &mut Test, r: &ReviewDisplay, token: Cancellation) -> Observation {
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, token).unwrap()).unwrap()
}
#[test]
fn observation_needs_independent_native_authority_and_changes_neither_repository() {
    let mut t = setup(true);
    trust(&mut t);
    let expected = String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let client = tree_bytes(&t.f.repo);
    let server = tree_bytes(&t.remote);
    let r = review(&mut t, "refs/heads/main");
    assert!(r.summary.contains("No credentials"));
    assert!(!t.f.root.join("state/network").exists());
    assert_eq!(
        observations(&mut t).items[0].phase,
        ObservationPhase::Proposed
    );
    assert_eq!(client, tree_bytes(&t.f.repo));
    assert_eq!(server, tree_bytes(&t.remote));
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        result.phase,
        ObservationPhase::Complete,
        "{:?}",
        result.issue
    );
    assert_eq!(result.oid.unwrap().hex, expected);
    assert!(result.observed_at_ms.is_some());
    assert_eq!(client, tree_bytes(&t.f.repo));
    assert_eq!(server, tree_bytes(&t.remote));
    assert!(t.app.begin_confirmation(&r.id).is_err());
}
#[test]
fn missing_remote_branch_is_observed_absent_without_creating_a_destination() {
    let mut t = setup(true);
    trust(&mut t);
    let before = tree_bytes(&t.remote);
    let r = review(&mut t, "refs/heads/does-not-exist");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        result.phase,
        ObservationPhase::Complete,
        "{:?}",
        result.issue
    );
    assert!(result.oid.is_none());
    assert_eq!(before, tree_bytes(&t.remote));
}
#[test]
fn untrusted_or_ordinary_local_destinations_never_gain_network_authority() {
    let mut t = setup(false);
    let request = || Request::ReviewRemoteObservation {
        workspace: t.workspace.clone(),
        worktree: t.worktree.clone(),
        remote: "origin".into(),
        reference: "refs/heads/main".into(),
    };
    assert_eq!(
        t.app.request(t.app.epoch(), request()).unwrap_err().kind,
        ErrorKind::UntrustedRepository
    );
    trust(&mut t);
    let result = t.app.request(
        t.app.epoch(),
        Request::ReviewRemoteObservation {
            workspace: t.workspace.clone(),
            worktree: t.worktree.clone(),
            remote: "origin".into(),
            reference: "refs/heads/main".into(),
        },
    );
    assert_eq!(
        result.unwrap_err().kind,
        ErrorKind::UnsupportedRepositoryState
    );
    assert!(!t.f.root.join("state/network").exists());
}
#[test]
fn decline_cancellation_and_configuration_drift_never_contact_a_remote() {
    let mut t = setup(true);
    trust(&mut t);
    let r = review(&mut t, "refs/heads/main");
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert_eq!(
        t.app
            .confirm(cap, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(
        observations(&mut t).items[0].phase,
        ObservationPhase::Cancelled
    );
    let r = review(&mut t, "refs/heads/main");
    let token = Cancellation::default();
    token.cancel();
    let result = confirm(&mut t, &r, token);
    assert_eq!(result.phase, ObservationPhase::Cancelled);
    assert!(!t.f.root.join("state/network").exists());
    let r = review(&mut t, "refs/heads/main");
    t.f.git(&[
        "config",
        "remote.origin.url",
        "https://changed.example.invalid/repo",
    ]);
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    assert_eq!(result.issue.unwrap().kind, ErrorKind::UntrustedRepository);
    assert!(!t.f.root.join("state/network").exists());
}
#[test]
fn restart_exposes_an_orphaned_review_but_never_restores_authority() {
    let mut t = setup(true);
    trust(&mut t);
    let r = review(&mut t, "refs/heads/main");
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let before = tree_bytes(&t.f.repo);
    let old = std::mem::replace(
        &mut t.app,
        Application::open(&t.f.root.join("other-state"), t.options.clone()).unwrap(),
    );
    drop(old);
    t.app = Application::open(&t.f.root.join("state"), t.options.clone()).unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(
        observations(&mut t).items[0].phase,
        ObservationPhase::Interrupted
    );
    assert!(!t.f.root.join("state/network").exists());
    assert_eq!(before, tree_bytes(&t.f.repo));
}

struct TlsServer {
    child: std::process::Child,
    port: u16,
    marker: PathBuf,
}
impl Drop for TlsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn tls_fixture(t: &mut Test, mode: &str) -> TlsServer {
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };
    let cert = t.f.root.join("certificate.pem");
    let key = t.f.root.join("key.pem");
    let config = t.f.root.join("certificate.cnf");
    fs::write(&config,b"[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=GitFleet TLS Fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyCertSign\n").unwrap();
    let certificate = Command::new("openssl")
        .args([
            "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1", "-config",
        ])
        .arg(&config)
        .arg("-keyout")
        .arg(&key)
        .arg("-out")
        .arg(&cert)
        .output()
        .unwrap();
    assert!(
        certificate.status.success(),
        "{}",
        String::from_utf8_lossy(&certificate.stderr)
    );
    let marker = t.f.root.join("network-requests");
    let log = fs::File::create(t.f.root.join("https-server.txt")).unwrap();
    let oid = String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let mut child = Command::new("python3")
        .args([
            "-I",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/https_observation.py"
            ),
        ])
        .arg(&cert)
        .arg(&key)
        .arg(&marker)
        .arg(mode)
        .arg(oid)
        .arg(&t.remote)
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout).read_line(&mut line);
        let _ = sender.send((result, line));
    });
    let ready = receiver.recv_timeout(std::time::Duration::from_secs(10));
    if ready.is_err() {
        let _ = child.kill();
    }
    reader.join().unwrap();
    let (result, line) = ready.unwrap();
    result.unwrap();
    let port = line.trim().parse().unwrap_or_else(|_| {
        panic!(
            "TLS server did not start: {}",
            fs::read_to_string(t.f.root.join("https-server.txt")).unwrap()
        )
    });
    t.options.remote.ca_file = Some(cert);
    TlsServer {
        child,
        port,
        marker,
    }
}
fn reopen(t: &mut Test) {
    let old = std::mem::replace(
        &mut t.app,
        Application::open(
            &t.f.root
                .join(format!("replacement-{}", uuid::Uuid::new_v4())),
            t.options.clone(),
        )
        .unwrap(),
    );
    drop(old);
    t.app = Application::open(&t.f.root.join("state"), t.options.clone()).unwrap();
}
fn configure_https(t: &mut Test, server: &TlsServer, hostname: &str) {
    t.f.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("https://{hostname}:{}/repo.git", server.port),
    ]);
    reopen(t);
    trust(t);
}
#[test]
fn verified_https_observation_ignores_managed_helpers_proxies_and_unsafe_tls_settings() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "normal");
    let helper = t.f.root.join("helper-ran");
    t.f.git(&[
        "config",
        "credential.helper",
        &format!("!touch {}", helper.display()),
    ]);
    t.f.git(&["config", "http.proxy", "http://127.0.0.1:1"]);
    t.f.git(&["config", "http.sslVerify", "false"]);
    configure_https(&mut t, &server, "127.0.0.1");
    let before = tree_bytes(&t.f.repo);
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        result.phase,
        ObservationPhase::Complete,
        "{:?}",
        result.issue
    );
    assert!(result.oid.is_some());
    assert!(!helper.exists());
    assert!(!fs::read_to_string(&server.marker)
        .unwrap()
        .contains("auth=True"));
    assert_eq!(before, tree_bytes(&t.f.repo));
}
#[test]
fn https_requires_a_valid_certificate_hostname_and_refuses_redirects() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "normal");
    let ca = t.options.remote.ca_file.take();
    configure_https(&mut t, &server, "127.0.0.1");
    let r = review(&mut t, "refs/heads/main");
    assert_eq!(
        confirm(&mut t, &r, Cancellation::default()).phase,
        ObservationPhase::Blocked
    );
    assert!(!server.marker.exists());
    t.options.remote.ca_file = ca;
    configure_https(&mut t, &server, "localhost");
    let r = review(&mut t, "refs/heads/main");
    assert_eq!(
        confirm(&mut t, &r, Cancellation::default()).phase,
        ObservationPhase::Blocked
    );
    assert!(!server.marker.exists());
    drop(server);
    let server = tls_fixture(&mut t, "redirect");
    configure_https(&mut t, &server, "127.0.0.1");
    let r = review(&mut t, "refs/heads/main");
    assert_eq!(
        confirm(&mut t, &r, Cancellation::default()).phase,
        ObservationPhase::Blocked
    );
    let requests = fs::read_to_string(&server.marker).unwrap();
    assert_eq!(requests.lines().count(), 1);
    assert!(!requests.contains("/forbidden"));
}
#[test]
fn running_https_observation_is_cancelled_after_the_actual_request_starts() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "slow");
    configure_https(&mut t, &server, "127.0.0.1");
    let r = review(&mut t, "refs/heads/main");
    let token = Cancellation::default();
    let cancel = token.clone();
    let marker = server.marker.clone();
    let waiter = std::thread::spawn(move || {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !marker.exists() && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let started = marker.exists();
        cancel.cancel();
        started
    });
    let before = tree_bytes(&t.f.repo);
    let result = confirm(&mut t, &r, token);
    assert!(waiter.join().unwrap());
    assert_eq!(result.phase, ObservationPhase::Cancelled);
    assert_eq!(before, tree_bytes(&t.f.repo));
}

#[test]
fn certificate_source_drift_requires_new_review_before_any_request() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "normal");
    configure_https(&mut t, &server, "127.0.0.1");
    let r = review(&mut t, "refs/heads/main");
    let certificate = t.options.remote.ca_file.as_ref().unwrap();
    use std::io::Write;
    fs::OpenOptions::new()
        .append(true)
        .open(certificate)
        .unwrap()
        .write_all(b"\n")
        .unwrap();
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    assert_eq!(result.issue.unwrap().kind, ErrorKind::StaleReview);
    assert!(!server.marker.exists());
    assert!(!t.f.root.join("state/network").exists());
}
#[test]
fn authentication_failure_never_uses_a_configured_helper_or_retains_raw_server_text() {
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "authentication");
    let helper = t.f.root.join("helper-ran");
    t.f.git(&[
        "config",
        "credential.helper",
        &format!("!touch {}", helper.display()),
    ]);
    configure_https(&mut t, &server, "127.0.0.1");
    let r = review(&mut t, "refs/heads/main");
    let result = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(result.phase, ObservationPhase::Blocked);
    assert!(!helper.exists());
    assert!(!serde_json::to_string(&result)
        .unwrap()
        .contains("SECRET-SERVER-DIAGNOSTIC"));
    assert!(!fs::read_to_string(&server.marker)
        .unwrap()
        .contains("auth=True"));
}

fn advance_server(t: &Test) -> String {
    let old = String::from_utf8(t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main",
    ]))
    .unwrap();
    let tree = String::from_utf8(t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "rev-parse",
        "refs/heads/main^{tree}",
    ]))
    .unwrap();
    let next = String::from_utf8(t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "-c",
        "user.name=Remote fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit-tree",
        tree.trim(),
        "-p",
        old.trim(),
        "-m",
        "Remote-only commit",
    ]))
    .unwrap();
    t.f.git(&[
        "-C",
        t.remote.to_str().unwrap(),
        "update-ref",
        "refs/heads/main",
        next.trim(),
    ]);
    next.trim().into()
}
fn fetch_review(t: &mut Test, observation: &str) -> ReviewDisplay {
    let Data::Review(review) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchCapture {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    review
}
fn observed(t: &mut Test) -> Observation {
    let review = review(t, "refs/heads/main");
    confirm(t, &review, Cancellation::default())
}
fn fetch_confirm(
    t: &mut Test,
    review: &ReviewDisplay,
    token: Cancellation,
) -> gitfleet_core::remote::fetch::FetchCapture {
    let (cap, _) = t.app.begin_confirmation(&review.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, token).unwrap()).unwrap()
}
fn captures(t: &mut Test) -> gitfleet_core::remote::fetch::FetchCapturePage {
    let Data::FetchCaptures(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::FetchCaptures {
                workspace: t.workspace.clone(),
                worktree: Some(t.worktree.clone()),
                offset: 0,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    page
}
#[test]
fn fetch_capture_requires_fresh_native_authority_and_produces_exact_verified_bundle_without_managed_mutation(
) {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    let mut t = setup(true);
    trust(&mut t);
    let remote_only = advance_server(&t);
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", &remote_only])
        .status
        .success());
    let source = observed(&mut t);
    let client = tree_bytes(&t.f.repo);
    let server = tree_bytes(&t.remote);
    let review = fetch_review(&mut t, &source.plan.id);
    assert!(review.summary.contains("isolated app storage"));
    let pending = captures(&mut t).items.remove(0);
    let id = pending.plan.transport.id;
    assert_eq!(pending.phase, FetchCapturePhase::Proposed);
    assert!(!t.f.root.join("state/network").join(&id).exists());
    let result = fetch_confirm(&mut t, &review, Cancellation::default());
    assert_eq!(
        result.phase,
        FetchCapturePhase::Complete,
        "{:?}",
        result.issue
    );
    let artifact = result.artifact.unwrap();
    assert!(artifact.object_count >= 3);
    assert!(artifact.bundle_bytes > 0);
    let bundle =
        t.f.root
            .join("state/network")
            .join(id)
            .join("capture.bundle");
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(fs::read(&bundle).unwrap())),
        artifact.bundle_digest
    );
    let heads =
        String::from_utf8(t.f.git(&["bundle", "list-heads", bundle.to_str().unwrap()])).unwrap();
    assert_eq!(
        heads,
        format!("{} refs/heads/gitfleet-fetch\n", source.oid.unwrap().hex)
    );
    let verify = t.f.root.join("verify.git");
    t.f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "--bare",
        "-q",
        verify.to_str().unwrap(),
    ]);
    t.f.git(&[
        "-C",
        verify.to_str().unwrap(),
        "fetch",
        "--no-write-fetch-head",
        "--no-tags",
        bundle.to_str().unwrap(),
        "refs/heads/gitfleet-fetch:refs/heads/verify",
    ]);
    assert_eq!(
        t.f.git(&[
            "-C",
            verify.to_str().unwrap(),
            "show",
            "refs/heads/verify:file.txt"
        ]),
        t.f.git(&["show", "HEAD:file.txt"])
    );
    assert!(!t
        .f
        .git_output(&["cat-file", "-e", &remote_only])
        .status
        .success());
    assert_eq!(client, tree_bytes(&t.f.repo));
    assert_eq!(server, tree_bytes(&t.remote));
    assert!(t.app.begin_confirmation(&review.id).is_err());
    let Data::OperationAudit(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: None,
                    operation_kind: Some(OperationAuditKind::FetchCapture),
                    result: Some(OperationAuditResult::Complete),
                    since_ms: None,
                    until_ms: None,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items.len(), 1);
    assert!(matches!(
        &page.items[0].record,
        OperationAuditRecord::FetchCapture(_)
    ));
}
#[test]
fn fetch_capture_decline_precancel_and_configuration_drift_cannot_import_or_contact_a_server() {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    let mut t = setup(true);
    trust(&mut t);
    let source = observed(&mut t);
    let before = tree_bytes(&t.f.repo);
    let review = fetch_review(&mut t, &source.plan.id);
    let (cap, _) = t.app.begin_confirmation(&review.id).unwrap();
    assert_eq!(
        t.app
            .confirm(cap, false, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(
        captures(&mut t).items[0].phase,
        FetchCapturePhase::Cancelled
    );
    let review = fetch_review(&mut t, &source.plan.id);
    let token = Cancellation::default();
    token.cancel();
    let result = fetch_confirm(&mut t, &review, token);
    assert_eq!(result.phase, FetchCapturePhase::Cancelled);
    assert!(!t
        .f
        .root
        .join("state/network")
        .join(result.plan.transport.id)
        .exists());
    assert_eq!(before, tree_bytes(&t.f.repo));
    let review = fetch_review(&mut t, &source.plan.id);
    t.f.git(&[
        "config",
        "remote.origin.url",
        "https://example.invalid/drift.git",
    ]);
    let changed = tree_bytes(&t.f.repo);
    let result = fetch_confirm(&mut t, &review, Cancellation::default());
    assert_eq!(result.phase, FetchCapturePhase::Blocked);
    assert!(!t
        .f
        .root
        .join("state/network")
        .join(result.plan.transport.id)
        .exists());
    assert_eq!(changed, tree_bytes(&t.f.repo));
}
#[test]
fn fetch_capture_rejects_missing_or_absent_sources_and_loses_authority_on_restart() {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    let mut t = setup(true);
    trust(&mut t);
    for id in ["forged", "00000000-0000-4000-8000-000000000000"] {
        assert!(t
            .app
            .request(
                t.app.epoch(),
                Request::ReviewFetchCapture {
                    workspace: t.workspace.clone(),
                    worktree: t.worktree.clone(),
                    observation: id.into()
                }
            )
            .is_err());
    }
    let absent = review(&mut t, "refs/heads/absent");
    let absent = confirm(&mut t, &absent, Cancellation::default());
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchCapture {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: absent.plan.id
            }
        )
        .is_err());
    let source = observed(&mut t);
    let review = fetch_review(&mut t, &source.plan.id);
    let id = captures(&mut t).items[0].plan.transport.id.clone();
    let (cap, _) = t.app.begin_confirmation(&review.id).unwrap();
    reopen(&mut t);
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(
        captures(&mut t).items[0].phase,
        FetchCapturePhase::Interrupted
    );
    assert!(!t.f.root.join("state/network").join(id).exists());
}
#[test]
fn fetch_capture_detects_remote_branch_drift_without_importing_downloaded_objects() {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    let mut t = setup(true);
    trust(&mut t);
    let source = observed(&mut t);
    let review = fetch_review(&mut t, &source.plan.id);
    // The server can independently move a branch after the observation/review.
    advance_server(&t);
    let before = tree_bytes(&t.f.repo);
    let result = fetch_confirm(&mut t, &review, Cancellation::default());
    assert_eq!(result.phase, FetchCapturePhase::Blocked);
    assert_eq!(
        result.issue.as_ref().unwrap().kind,
        ErrorKind::ConcurrentChange
    );
    assert!(result.artifact.is_none());
    assert_eq!(before, tree_bytes(&t.f.repo));
}

#[test]
fn fetch_capture_downloads_real_objects_over_verified_https_with_managed_helpers_isolated() {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    let mut t = setup(false);
    let server = tls_fixture(&mut t, "git");
    configure_https(&mut t, &server, "127.0.0.1");
    let source = observed(&mut t);
    assert!(source.oid.is_some());
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let review = fetch_review(&mut t, &source.plan.id);
    let result = fetch_confirm(&mut t, &review, Cancellation::default());
    assert_eq!(
        result.phase,
        FetchCapturePhase::Complete,
        "{:?}",
        result.issue
    );
    assert!(result.artifact.unwrap().object_count >= 3);
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_eq!(remote, tree_bytes(&t.remote));
    assert!(fs::read_to_string(&server.marker)
        .unwrap()
        .contains("info/refs?service=git-upload-pack"));
}
#[test]
fn fetch_capture_refuses_authentication_redirects_and_cancels_a_proven_running_request() {
    use gitfleet_core::remote::fetch::FetchCapturePhase;
    for mode in ["authentication", "redirect", "slow"] {
        let mut t = setup(false);
        let server = tls_fixture(&mut t, "git");
        configure_https(&mut t, &server, "127.0.0.1");
        let source = observed(&mut t);
        let review = fetch_review(&mut t, &source.plan.id);
        let before = tree_bytes(&t.f.repo);
        fs::write(t.f.root.join("network-mode"), mode).unwrap();
        fs::write(&server.marker, "").unwrap();
        let token = Cancellation::default();
        let cancel = token.clone();
        let marker = server.marker.clone();
        let worker = std::thread::spawn(move || {
            let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !marker.exists() || fs::read_to_string(&marker).unwrap_or_default().is_empty() {
                assert!(
                    std::time::Instant::now() < until,
                    "No fetch request started"
                );
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            if mode == "slow" {
                cancel.cancel();
            }
        });
        let result = fetch_confirm(&mut t, &review, token);
        worker.join().unwrap();
        assert_eq!(
            result.phase,
            if mode == "slow" {
                FetchCapturePhase::Cancelled
            } else {
                FetchCapturePhase::Blocked
            },
            "{:?}",
            result.issue
        );
        assert!(result.artifact.is_none());
        assert_eq!(before, tree_bytes(&t.f.repo));
        let rendered = serde_json::to_string(&result).unwrap();
        assert!(!rendered.contains("SECRET-SERVER-DIAGNOSTIC"));
        let requests = fs::read_to_string(&server.marker).unwrap();
        assert!(!requests.contains("/forbidden"));
        assert!(!requests.contains("auth=true"));
    }
}

fn downloaded(t: &mut Test) -> gitfleet_core::remote::fetch::FetchCapture {
    let source = observed(t);
    let r = fetch_review(t, &source.plan.id);
    fetch_confirm(t, &r, Cancellation::default())
}
fn import_review(t: &mut Test, capture: &str) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchImport {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                capture: capture.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
fn import_confirm(t: &mut Test, r: &ReviewDisplay) -> gitfleet_core::remote::fetch::FetchImport {
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap()
}
#[test]
fn fetch_import_adds_exact_objects_and_one_tracking_ref_with_separate_authority() {
    use gitfleet_core::remote::fetch::*;
    let mut t = setup(true);
    trust(&mut t);
    let oid = advance_server(&t);
    let capture = downloaded(&mut t);
    assert_eq!(capture.phase, FetchCapturePhase::Complete);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let r = import_review(&mut t, &capture.plan.transport.id);
    assert!(r.summary.contains(&oid));
    assert!(r
        .summary
        .contains(&capture.artifact.as_ref().unwrap().bundle_digest));
    assert_eq!(before, tree_bytes(&t.f.repo));
    let imported = import_confirm(&mut t, &r);
    assert_eq!(
        imported.phase,
        FetchImportPhase::Complete,
        "{:?}",
        imported.issue
    );
    assert!(imported.objects_imported);
    assert_eq!(imported.reference_status, FetchReferenceStatus::Updated);
    assert_eq!(
        String::from_utf8(t.f.git(&["rev-parse", "refs/remotes/origin/main"]))
            .unwrap()
            .trim(),
        oid
    );
    t.f.git(&["cat-file", "-e", &oid]);
    let mut after = tree_bytes(&t.f.repo);
    after.retain(|p, _| {
        !p.starts_with(".git/objects")
            && !p.starts_with(".git/refs/remotes")
            && !p.starts_with(".git/logs/refs/remotes")
    });
    let mut before = before;
    before.retain(|p, _| !p.starts_with(".git/objects"));
    assert_eq!(before, after);
    assert_eq!(remote, tree_bytes(&t.remote));
    assert_eq!(
        fs::read_to_string(t.f.repo.join(".git/logs/refs/remotes/origin/main"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(t.app.begin_confirmation(&r.id).is_err());
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchImport {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                capture: capture.plan.transport.id
            }
        )
        .is_err());
    let Data::OperationAudit(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: None,
                    operation_kind: Some(OperationAuditKind::FetchImport),
                    result: Some(OperationAuditResult::Complete),
                    since_ms: None,
                    until_ms: None,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, imported.plan.transport.id);
}

#[test]
fn fetch_import_decline_cancel_tamper_config_and_ref_drift_fail_closed() {
    use gitfleet_core::remote::fetch::*;
    for mode in ["decline", "cancel", "tamper", "config", "ref", "symbolic"] {
        let mut t = setup(true);
        trust(&mut t);
        advance_server(&t);
        let capture = downloaded(&mut t);
        let r = import_review(&mut t, &capture.plan.transport.id);
        match mode {
            "tamper" => {
                fs::write(
                    t.f.root
                        .join("state/network")
                        .join(&capture.plan.transport.id)
                        .join("capture.bundle"),
                    b"changed",
                )
                .unwrap();
            }
            "config" => {
                t.f.git(&["config", "remote.origin.url", "/missing"]);
            }
            "ref" => {
                t.f.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
            }
            "symbolic" => {
                t.f.git(&[
                    "symbolic-ref",
                    "refs/remotes/origin/main",
                    "refs/heads/main",
                ]);
            }
            _ => {}
        }
        let before = tree_bytes(&t.f.repo);
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        let token = Cancellation::default();
        if mode == "cancel" {
            token.cancel();
        }
        let result = t.app.confirm(cap, mode != "decline", token);
        if mode == "decline" {
            assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        } else {
            let value: FetchImport = serde_json::from_value(result.unwrap()).unwrap();
            assert!(
                matches!(
                    value.phase,
                    FetchImportPhase::Blocked | FetchImportPhase::Cancelled
                ),
                "{mode}: {value:?}"
            );
            assert!(!value.objects_imported);
        }
        assert_eq!(before, tree_bytes(&t.f.repo), "{mode}");
        assert!(t.app.begin_confirmation(&r.id).is_err());
    }
}
#[test]
fn fetch_import_rejects_aliases_and_shared_lock_and_orphaned_authority() {
    use std::os::unix::fs::symlink;
    for mode in ["symbolic", "symlink", "case", "shared", "restart"] {
        let mut t = setup(true);
        trust(&mut t);
        advance_server(&t);
        let capture = downloaded(&mut t);
        match mode {
            "symbolic" => {
                t.f.git(&[
                    "symbolic-ref",
                    "refs/remotes/origin/main",
                    "refs/heads/main",
                ]);
            }
            "symlink" => {
                symlink(
                    t.f.repo.join(".git/refs/heads"),
                    t.f.repo.join(".git/refs/remotes"),
                )
                .unwrap();
            }
            "case" => {
                fs::create_dir_all(t.f.repo.join(".git/refs/Remotes")).unwrap();
            }
            _ => {}
        }
        let before = tree_bytes(&t.f.repo);
        let result = t.app.request(
            t.app.epoch(),
            Request::ReviewFetchImport {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                capture: capture.plan.transport.id,
            },
        );
        if ["symbolic", "symlink", "case"].contains(&mode) {
            assert!(result.is_err(), "{mode}");
        } else {
            let Data::Review(r) = result.unwrap().data else {
                panic!()
            };
            let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
            if mode == "shared" {
                let file = fs::File::open(t.f.repo.join(".git")).unwrap();
                file.try_lock().unwrap();
                assert_eq!(
                    t.app
                        .confirm(cap, true, Cancellation::default())
                        .unwrap_err()
                        .kind,
                    ErrorKind::Busy
                );
                file.unlock().unwrap();
            } else {
                reopen(&mut t);
                assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
                let Data::FetchImports(page) = t
                    .app
                    .request(
                        t.app.epoch(),
                        Request::FetchImports {
                            workspace: t.workspace.clone(),
                            offset: 0,
                        },
                    )
                    .unwrap()
                    .data
                else {
                    panic!()
                };
                assert_eq!(
                    page.items[0].phase,
                    gitfleet_core::remote::fetch::FetchImportPhase::Interrupted
                );
            }
        }
        assert_eq!(before, tree_bytes(&t.f.repo), "{mode}");
    }
}
#[cfg(feature = "write-test-support")]
#[test]
fn fetch_import_storage_failure_requires_fresh_observation_and_preserves_foreign_lock() {
    use gitfleet_core::{remote::fetch::*, writes::TestFaultKind};
    let mut t = setup(true);
    fs::write(t.f.repo.join(".gitfleet-m2-fixture"), b"fixture").unwrap();
    trust(&mut t);
    advance_server(&t);
    let capture = downloaded(&mut t);
    let r = import_review(&mut t, &capture.plan.transport.id);
    t.app.set_import_test_fault(
        FetchImportPhase::ReferenceUpdated,
        TestFaultKind::StorageFailure,
    );
    let result = import_confirm(&mut t, &r);
    assert_eq!(result.phase, FetchImportPhase::Uncertain);
    let oid = t.f.git(&["rev-parse", "refs/remotes/origin/main"]);
    let before = tree_bytes(&t.f.repo);
    let Data::Workspaces(list) = t
        .app
        .request(t.app.epoch(), Request::Workspaces {})
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(list.fetch_import_recovery.len(), 1);
    assert_eq!(
        list.fetch_import_recovery[0].operation_id,
        result.plan.transport.id
    );
    assert_eq!(
        t.app
            .request(
                t.app.epoch(),
                Request::ReviewFetchImport {
                    workspace: t.workspace.clone(),
                    worktree: t.worktree.clone(),
                    capture: capture.plan.transport.id
                }
            )
            .unwrap_err()
            .kind,
        ErrorKind::Busy
    );

    assert_eq!(
        t.app
            .request(
                t.app.epoch(),
                Request::Remove {
                    workspace: t.workspace.clone()
                }
            )
            .unwrap_err()
            .kind,
        ErrorKind::Busy
    );
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchImportRecovery {
                workspace: t.workspace.clone(),
                operation: result.plan.transport.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(r.summary.contains("observe_target"));
    let lock = t.f.repo.join(".git/refs/remotes/origin/main.lock");
    fs::write(&lock, b"foreign lock").unwrap();
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(fs::read(&lock).unwrap(), b"foreign lock");
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchImportRecovery {
                workspace: t.workspace.clone(),
                operation: result.plan.transport.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(r.summary.contains("manual"));
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    fs::remove_file(lock).unwrap();
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchImportRecovery {
                workspace: t.workspace.clone(),
                operation: result.plan.transport.id,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let recovered = import_confirm(&mut t, &r);
    assert_eq!(recovered.phase, FetchImportPhase::ObservedTarget);
    assert_eq!(
        recovered.reference_status,
        FetchReferenceStatus::MatchesTarget
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_eq!(oid, t.f.git(&["rev-parse", "refs/remotes/origin/main"]));
}
#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "subprocess-only disposable SIGKILL worker"]
fn fetch_import_crash_worker() {
    use gitfleet_core::{remote::fetch::FetchImportPhase, writes::TestFaultKind};
    let Ok(marker) = std::env::var("GITFLEET_IMPORT_CRASH_MARKER") else {
        return;
    };
    let phase: FetchImportPhase =
        serde_json::from_str(&std::env::var("GITFLEET_IMPORT_CRASH_PHASE").unwrap()).unwrap();
    let mut t = setup(true);
    fs::write(t.f.repo.join(".gitfleet-m2-fixture"), b"fixture").unwrap();
    trust(&mut t);
    advance_server(&t);
    let capture = downloaded(&mut t);
    let r = import_review(&mut t, &capture.plan.transport.id);
    fs::write(
        marker,
        serde_json::to_vec(&(t.f.root.clone(), t.workspace.clone(), t.worktree.clone())).unwrap(),
    )
    .unwrap();
    t.app.set_import_test_fault(phase, TestFaultKind::Sigkill);
    let result = import_confirm(&mut t, &r);
    panic!("fault at {phase:?} did not kill worker: {result:?}");
}
#[cfg(feature = "write-test-support")]
#[test]
fn fetch_import_sigkill_before_and_after_publication_never_replays() {
    use gitfleet_core::remote::fetch::*;
    use std::os::unix::process::ExitStatusExt;
    for phase in [
        FetchImportPhase::ImportingObjects,
        FetchImportPhase::ReferencePrepared,
        FetchImportPhase::ReferenceUpdated,
    ] {
        let owner = Fixture::new();
        let marker = owner.root.join("worker.json");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "fetch_import_crash_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("GITFLEET_IMPORT_CRASH_MARKER", &marker)
            .env(
                "GITFLEET_IMPORT_CRASH_PHASE",
                serde_json::to_string(&phase).unwrap(),
            )
            .output()
            .unwrap();
        assert_eq!(
            output.status.signal(),
            Some(libc::SIGKILL),
            "phase {phase:?}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let (root, workspace, worktree): (PathBuf, String, String) =
            serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
        let f = Fixture {
            repo: root.join("repo"),
            root: root.clone(),
        };
        let options = Options {
            global_config: Some(root.join("global.config")),
            signer_home: Some(root.join("home")),
            remote: TransportOptions {
                local_fixture_root: Some(root.clone()),
                ca_file: None,
                ssh: None,
            },
            ..Options::default()
        };
        // EOF aborts a prepared child; wait for Git's owned lock cleanup before inspection.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while f.repo.join(".git/refs/remotes/origin/main.lock").exists() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let before = tree_bytes(&f.repo);
        let app = Application::open(&root.join("state"), options.clone()).unwrap();
        let mut t = Test {
            f,
            app,
            workspace,
            worktree,
            remote: root.join("remote.git"),
            options,
        };
        let Data::FetchImports(page) = t
            .app
            .request(
                t.app.epoch(),
                Request::FetchImports {
                    workspace: t.workspace.clone(),
                    offset: 0,
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        assert_eq!(page.items[0].phase, phase);
        assert_eq!(before, tree_bytes(&t.f.repo));
        let Data::Review(r) = t
            .app
            .request(
                t.app.epoch(),
                Request::ReviewFetchImportRecovery {
                    workspace: t.workspace.clone(),
                    operation: page.items[0].plan.transport.id.clone(),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let recovered = import_confirm(&mut t, &r);
        assert_eq!(
            recovered.phase,
            if phase == FetchImportPhase::ReferenceUpdated {
                FetchImportPhase::ObservedTarget
            } else {
                FetchImportPhase::Aborted
            }
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
        reopen(&mut t);
        assert_eq!(before, tree_bytes(&t.f.repo));
    }
}

#[test]
fn fetch_import_updates_packed_expected_old_for_sha1_and_sha256() {
    use gitfleet_core::remote::fetch::*;
    for sha256 in [false, true] {
        let mut t = setup_format(true, sha256);
        trust(&mut t);
        let old = String::from_utf8(t.f.git(&["rev-parse", "HEAD"])).unwrap();
        t.f.git(&["update-ref", "refs/remotes/origin/main", old.trim()]);
        t.f.git(&["pack-refs", "--all"]);
        assert!(!t.f.repo.join(".git/refs/remotes/origin/main").exists());
        let next = advance_server(&t);
        let capture = downloaded(&mut t);
        let r = import_review(&mut t, &capture.plan.transport.id);
        let result = import_confirm(&mut t, &r);
        assert_eq!(
            result.phase,
            FetchImportPhase::Complete,
            "{:?}",
            result.issue
        );
        assert_eq!(result.plan.expected_old.unwrap().hex, old.trim());
        assert_eq!(next.len(), if sha256 { 64 } else { 40 });
        assert_eq!(
            String::from_utf8(t.f.git(&["rev-parse", "refs/remotes/origin/main"]))
                .unwrap()
                .trim(),
            next
        );
        assert_eq!(
            String::from_utf8(t.f.git(&["rev-parse", "HEAD"])).unwrap(),
            old
        );
    }
}

fn push_preview(t: &mut Test, observation: &str) -> gitfleet_core::remote::push::PushPreview {
    let Data::PushPreview(preview) = t
        .app
        .request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    preview
}
fn advance_local(t: &Test) -> String {
    fs::write(t.f.repo.join("file.txt"), b"Local outgoing commit\n").unwrap();
    t.f.git(&["add", "file.txt"]);
    t.f.git(&["commit", "-qm", "Local outgoing <script> commit"]);
    String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .into()
}
#[test]
fn local_push_preview_reports_up_to_date_and_fast_forward_without_authority_or_mutation() {
    use gitfleet_core::remote::push::PushRelation;
    let mut t = setup(true);
    t.f.git(&["config", "branch.main.remote", "origin"]);
    t.f.git(&["config", "branch.main.merge", "refs/heads/main"]);
    trust(&mut t);
    let observed = observed(&mut t);
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let result = push_preview(&mut t, &observed.plan.id);
    assert_eq!(result.relation, PushRelation::UpToDate);
    assert_eq!(result.outgoing_count, 0);
    assert_eq!(result.incoming_count, 0);
    assert!(result.upstream_matches);
    assert!(result.outgoing.is_empty());
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_eq!(remote, tree_bytes(&t.remote));
    let oid = advance_local(&t);
    let before = tree_bytes(&t.f.repo);
    let result = push_preview(&mut t, &observed.plan.id);
    assert_eq!(result.relation, PushRelation::FastForward);
    assert_eq!(result.outgoing_count, 1);
    assert_eq!(result.incoming_count, 0);
    assert_eq!(result.outgoing[0].oid.hex, oid);
    assert_eq!(
        result.outgoing[0].subject,
        b"Local outgoing <script> commit"
    );
    assert!(!result.outgoing_truncated);
    assert_eq!(result.observed_oid, observed.oid.unwrap());
    assert_eq!(result.source_ref, "refs/heads/main");
    assert_eq!(result.destination_ref, "refs/heads/main");
    assert_eq!(before, tree_bytes(&t.f.repo));
    assert_eq!(remote, tree_bytes(&t.remote));
    let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
    assert_eq!(
        db.query_row(
            "select count(*) from state where key like 'remote-observation:%'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("select count(*) from write_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(t.app.begin_confirmation(&result.observation_id).is_err());
}
#[test]
fn local_push_preview_requires_explicit_objects_then_reports_behind_and_diverged() {
    use gitfleet_core::remote::push::PushRelation;
    for diverged in [false, true] {
        let mut t = setup(true);
        trust(&mut t);
        let target = advance_server(&t);
        let observation = observed(&mut t);
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.remote);
        assert_eq!(
            t.app
                .request(
                    t.app.epoch(),
                    Request::PushPreview {
                        workspace: t.workspace.clone(),
                        worktree: t.worktree.clone(),
                        observation: observation.plan.id.clone()
                    }
                )
                .unwrap_err()
                .kind,
            ErrorKind::MissingObject
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_eq!(remote, tree_bytes(&t.remote));
        let r = fetch_review(&mut t, &observation.plan.id);
        let capture = fetch_confirm(&mut t, &r, Cancellation::default());
        let r = import_review(&mut t, &capture.plan.transport.id);
        assert_eq!(
            import_confirm(&mut t, &r).phase,
            gitfleet_core::remote::fetch::FetchImportPhase::Complete
        );
        let local = if diverged {
            Some(advance_local(&t))
        } else {
            None
        };
        let before = tree_bytes(&t.f.repo);
        let result = push_preview(&mut t, &observation.plan.id);
        assert_eq!(
            result.relation,
            if diverged {
                PushRelation::Diverged
            } else {
                PushRelation::Behind
            }
        );
        assert_eq!(result.incoming_count, 1);
        assert_eq!(result.outgoing_count, u32::from(diverged));
        assert_eq!(result.observed_oid.hex, target);
        assert!(!result.upstream_matches);
        if let Some(oid) = local {
            assert_eq!(result.outgoing[0].oid.hex, oid);
        }
        assert_eq!(before, tree_bytes(&t.f.repo));
        assert_eq!(remote, tree_bytes(&t.remote));
    }
}
#[test]
fn local_push_preview_refuses_ambiguous_changed_or_unsupported_context_without_effects() {
    for mode in [
        "different_pushurl",
        "multiple_pushurl",
        "push_refspec",
        "config_drift",
        "revoked",
        "shallow",
        "graft",
        "cancelled_observation",
        "wrong_scope",
        "detached",
        "unborn",
    ] {
        let mut t = setup(true);
        trust(&mut t);
        if mode == "different_pushurl" {
            t.f.git(&[
                "config",
                "remote.origin.pushurl",
                "https://example.invalid/different.git",
            ]);
            trust(&mut t);
        }
        if mode == "push_refspec" {
            t.f.git(&[
                "config",
                "remote.origin.push",
                "refs/heads/main:refs/heads/other",
            ]);
            trust(&mut t);
        }
        let observation = if mode == "cancelled_observation" {
            let r = review(&mut t, "refs/heads/main");
            let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
            assert!(t.app.confirm(cap, false, Cancellation::default()).is_err());
            observations(&mut t).items[0].clone()
        } else {
            observed(&mut t)
        };
        if mode == "multiple_pushurl" {
            t.f.git(&[
                "config",
                "--add",
                "remote.origin.pushurl",
                t.remote.to_str().unwrap(),
            ]);
            t.f.git(&[
                "config",
                "--add",
                "remote.origin.pushurl",
                "https://example.invalid/other.git",
            ]);
            trust(&mut t);
        }
        match mode {
            "detached" => {
                t.f.git(&["checkout", "--detach", "-q"]);
            }
            "unborn" => {
                t.f.git(&["symbolic-ref", "HEAD", "refs/heads/unborn"]);
            }
            "config_drift" => {
                t.f.git(&[
                    "config",
                    "remote.origin.url",
                    "https://example.invalid/changed.git",
                ]);
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
            "shallow" => fs::write(
                t.f.repo.join(".git/shallow"),
                format!("{}\n", observation.oid.as_ref().unwrap().hex),
            )
            .unwrap(),
            "graft" => {
                fs::create_dir_all(t.f.repo.join(".git/info")).unwrap();
                fs::write(t.f.repo.join(".git/info/grafts"), b"").unwrap();
            }
            _ => {}
        }
        let before = tree_bytes(&t.f.repo);
        let remote = tree_bytes(&t.remote);
        let result = t.app.request(
            t.app.epoch(),
            Request::PushPreview {
                workspace: if mode == "wrong_scope" {
                    "forged".into()
                } else {
                    t.workspace.clone()
                },
                worktree: t.worktree.clone(),
                observation: observation.plan.id,
            },
        );
        assert!(result.is_err(), "{mode}");
        assert_eq!(before, tree_bytes(&t.f.repo), "{mode}");
        assert_eq!(remote, tree_bytes(&t.remote), "{mode}");
    }
}
#[test]
fn local_push_preview_bounds_outgoing_details_and_loses_no_exact_count() {
    let mut t = setup(true);
    trust(&mut t);
    let observation = observed(&mut t);
    let tree = String::from_utf8(t.f.git(&["rev-parse", "HEAD^{tree}"])).unwrap();
    let mut parent = String::from_utf8(t.f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_string();
    for i in 0..101 {
        parent = String::from_utf8(t.f.git(&[
            "commit-tree",
            tree.trim(),
            "-p",
            &parent,
            "-m",
            &format!("Outgoing {i}"),
        ]))
        .unwrap()
        .trim()
        .into();
    }
    t.f.git(&["update-ref", "refs/heads/main", &parent]);
    let before = tree_bytes(&t.f.repo);
    let result = push_preview(&mut t, &observation.plan.id);
    assert_eq!(result.outgoing_count, 101);
    assert_eq!(result.outgoing.len(), 100);
    assert!(result.outgoing_truncated);
    assert_eq!(result.outgoing[0].oid.hex, parent);
    assert_eq!(before, tree_bytes(&t.f.repo));
    let token = Cancellation::default();
    token.cancel();
    assert_eq!(
        t.app
            .request_with_cancellation(
                t.app.epoch(),
                Request::PushPreview {
                    workspace: t.workspace.clone(),
                    worktree: t.worktree.clone(),
                    observation: observation.plan.id
                },
                token
            )
            .unwrap_err()
            .kind,
        ErrorKind::Cancelled
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
}

fn upstream_review(t: &mut Test, observation: &str) -> ReviewDisplay {
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewUpstreamSetup {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.into(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    r
}
fn upstream_confirm(
    t: &mut Test,
    r: &ReviewDisplay,
) -> gitfleet_core::remote::upstream::UpstreamOperation {
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    serde_json::from_value(t.app.confirm(cap, true, Cancellation::default()).unwrap()).unwrap()
}
#[test]
fn upstream_setup_adds_only_missing_keys_preserves_bytes_and_requires_new_trust() {
    use gitfleet_core::remote::upstream::*;
    let mut t = setup(true);
    trust(&mut t);
    let observation = observed(&mut t);
    fs::write(t.f.repo.join("file.txt"), b"Staged upstream fixture\n").unwrap();
    t.f.git(&["add", "file.txt"]);
    fs::write(t.f.repo.join("file.txt"), b"Working content\n").unwrap();
    let before = tree_bytes(&t.f.repo);
    let remote = tree_bytes(&t.remote);
    let old = fs::read(t.f.repo.join(".git/config")).unwrap();
    let r = upstream_review(&mut t, &observation.plan.id);
    assert!(r.summary.contains("refs/heads/main"));
    assert!(r.summary.contains("Existing trust becomes stale"));
    assert_eq!(before, tree_bytes(&t.f.repo));
    let value = upstream_confirm(&mut t, &r);
    assert_eq!(value.phase, UpstreamPhase::Complete, "{:?}", value.issue);
    assert_eq!(value.configuration, UpstreamStatus::Updated);
    let config = fs::read(t.f.repo.join(".git/config")).unwrap();
    assert!(config.starts_with(&old));
    assert_eq!(
        &config[old.len()..],
        b"\n[branch \"main\"]\n\tremote = \"origin\"\n\tmerge = \"refs/heads/main\"\n"
    );
    let mut after = tree_bytes(&t.f.repo);
    after.insert(
        std::path::PathBuf::from(".git/config"),
        before[std::path::Path::new(".git/config")].clone(),
    );
    assert_eq!(before, after);
    assert_eq!(remote, tree_bytes(&t.remote));
    assert_eq!(
        t.f.git(&["config", "--get", "branch.main.remote"]),
        b"origin\n"
    );
    assert_eq!(
        t.f.git(&["config", "--get", "branch.main.merge"]),
        b"refs/heads/main\n"
    );
    assert!(t.app.begin_confirmation(&r.id).is_err());
    assert!(t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewFetchCapture {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.plan.id
            }
        )
        .is_err());
    let Data::Trust(status) = t
        .app
        .request(
            t.app.epoch(),
            Request::TrustStatus {
                workspace: t.workspace.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(status.iter().any(|v| v.validity != "current"));
    let Data::OperationAudit(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: None,
                    operation_kind: Some(OperationAuditKind::UpstreamSetup),
                    result: Some(OperationAuditResult::Complete),
                    since_ms: None,
                    until_ms: None,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, value.plan.transport.id);
}
#[test]
fn upstream_decline_stale_config_head_trust_and_foreign_locks_preserve_repository() {
    for mode in [
        "decline",
        "cancel",
        "config",
        "head",
        "revoke",
        "foreign_lock",
        "same_oid_config_replace",
    ] {
        let mut t = setup(true);
        trust(&mut t);
        let observation = observed(&mut t);
        let r = upstream_review(&mut t, &observation.plan.id);
        let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
        match mode {
            "config" => {
                t.f.git(&["config", "custom.changed", "true"]);
            }
            "head" => {
                advance_local(&t);
            }
            "revoke" => {
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
            "foreign_lock" => fs::write(t.f.repo.join(".git/config.lock"), b"foreign").unwrap(),
            "same_oid_config_replace" => {
                let path = t.f.repo.join(".git/config");
                let bytes = fs::read(&path).unwrap();
                fs::remove_file(&path).unwrap();
                fs::write(path, bytes).unwrap();
            }
            _ => {}
        }
        let before = tree_bytes(&t.f.repo);
        let token = Cancellation::default();
        if mode == "cancel" {
            token.cancel();
        }
        let result = t.app.confirm(cap, mode != "decline", token);
        if mode == "decline" {
            assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        } else if mode == "revoke" {
            assert!(result.is_err());
        } else {
            let value: gitfleet_core::remote::upstream::UpstreamOperation =
                serde_json::from_value(result.unwrap()).unwrap();
            assert!(
                matches!(
                    value.phase,
                    gitfleet_core::remote::upstream::UpstreamPhase::Blocked
                        | gitfleet_core::remote::upstream::UpstreamPhase::Cancelled
                ),
                "{mode}: {value:?}"
            );
        }
        assert_eq!(before, tree_bytes(&t.f.repo), "{mode}");
    }
}

#[test]
fn upstream_setup_refuses_unsupported_policy_and_preserves_attributes_without_persisting_secrets() {
    for mode in [
        "existing",
        "mapping",
        "worktree_config",
        "include",
        "hardlink",
        "special_mode",
        "xattr",
        "acl",
        "quotes_and_secret",
    ] {
        let mut t = setup(true);
        match mode {
            "existing" => {
                t.f.git(&[
                    "config",
                    "branch.main.description",
                    "Existing user branch policy",
                ]);
            }
            "mapping" => {
                t.f.git(&[
                    "config",
                    "remote.origin.fetch",
                    "+refs/heads/*:refs/other/*",
                ]);
            }
            "worktree_config" => {
                t.f.git(&["config", "extensions.worktreeConfig", "true"]);
            }
            "include" => {
                let p = t.f.root.join("included.config");
                fs::write(&p, b"[custom]\n value = true\n").unwrap();
                t.f.git(&["config", "include.path", p.to_str().unwrap()]);
            }
            "hardlink" => fs::hard_link(
                t.f.repo.join(".git/config"),
                t.f.root.join("config-hardlink"),
            )
            .unwrap(),
            "special_mode" => {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    t.f.repo.join(".git/config"),
                    fs::Permissions::from_mode(0o1600),
                )
                .unwrap();
            }
            "xattr" => {
                assert!(std::process::Command::new("/usr/bin/xattr")
                    .args(["-w", "com.gitfleet.fixture", "sentinel"])
                    .arg(t.f.repo.join(".git/config"))
                    .status()
                    .unwrap()
                    .success());
            }
            "acl" => {
                let rule = "everyone allow read";
                assert!(std::process::Command::new("/bin/chmod")
                    .args(["+a", rule])
                    .arg(t.f.repo.join(".git/config"))
                    .status()
                    .unwrap()
                    .success());
            }
            "quotes_and_secret" => {
                t.f.git(&["branch", "-m", "feature/quoted\"branch"]);
                t.f.git(&[
                    "remote",
                    "add",
                    "other",
                    "https://fixture:SECRET_SENTINEL@example.invalid/other.git",
                ]);
            }
            _ => {}
        }
        trust(&mut t);
        let observation = observed(&mut t);
        let before = tree_bytes(&t.f.repo);
        let result = t.app.request(
            t.app.epoch(),
            Request::ReviewUpstreamSetup {
                workspace: t.workspace.clone(),
                worktree: t.worktree.clone(),
                observation: observation.plan.id,
            },
        );
        if mode == "quotes_and_secret" || mode == "xattr" {
            let Data::Review(r) = result.unwrap().data else {
                panic!()
            };
            assert!(!r.content.contains("SECRET_SENTINEL"));
            assert!(!r.summary.contains("SECRET_SENTINEL"));
            let value = upstream_confirm(&mut t, &r);
            assert_eq!(
                value.phase,
                gitfleet_core::remote::upstream::UpstreamPhase::Complete,
                "{:?}",
                value.issue
            );
            if mode == "quotes_and_secret" {
                assert_eq!(
                    t.f.git(&["config", "--get", "branch.feature/quoted\"branch.remote"]),
                    b"origin\n"
                );
            }
            if mode == "xattr" {
                assert_eq!(
                    std::process::Command::new("/usr/bin/xattr")
                        .args(["-p", "com.gitfleet.fixture"])
                        .arg(t.f.repo.join(".git/config"))
                        .output()
                        .unwrap()
                        .stdout,
                    b"sentinel\n"
                );
            }
            let db = rusqlite::Connection::open(t.f.root.join("state/state.db")).unwrap();
            let mut q = db
                .prepare("select value from state where key like 'upstream:%'")
                .unwrap();
            for raw in q.query_map([], |r| r.get::<_, String>(0)).unwrap() {
                assert!(!raw.unwrap().contains("SECRET_SENTINEL"));
            }
        } else {
            assert!(result.is_err(), "{mode}");
            assert_eq!(before, tree_bytes(&t.f.repo), "{mode}");
        }
    }
}
#[cfg(feature = "write-test-support")]
#[test]
#[ignore = "subprocess-only disposable upstream SIGKILL worker"]
fn upstream_crash_worker() {
    use gitfleet_core::{remote::upstream::UpstreamPhase, writes::TestFaultKind};
    let Ok(marker) = std::env::var("GITFLEET_UPSTREAM_CRASH_MARKER") else {
        return;
    };
    let phase: UpstreamPhase =
        serde_json::from_str(&std::env::var("GITFLEET_UPSTREAM_CRASH_PHASE").unwrap()).unwrap();
    let mut t = setup(true);
    fs::write(t.f.repo.join(".gitfleet-m2-fixture"), b"fixture").unwrap();
    trust(&mut t);
    let observation = observed(&mut t);
    let r = upstream_review(&mut t, &observation.plan.id);
    fs::write(
        marker,
        serde_json::to_vec(&(t.f.root.clone(), t.workspace.clone(), t.worktree.clone())).unwrap(),
    )
    .unwrap();
    t.app.set_upstream_test_fault(phase, TestFaultKind::Sigkill);
    let value = upstream_confirm(&mut t, &r);
    panic!("Fault {phase:?} not delivered: {value:?}");
}
#[cfg(feature = "write-test-support")]
#[test]
fn upstream_sigkill_recovery_never_replays_and_removes_only_owned_config_lock() {
    use gitfleet_core::remote::upstream::*;
    use std::os::unix::process::ExitStatusExt;
    for phase in [
        UpstreamPhase::LockPending,
        UpstreamPhase::LockCreated,
        UpstreamPhase::LockReady,
        UpstreamPhase::PublicationPending,
        UpstreamPhase::ConfigPublished,
    ] {
        let owner = Fixture::new();
        let marker = owner.root.join("worker.json");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "upstream_crash_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("GITFLEET_UPSTREAM_CRASH_MARKER", &marker)
            .env(
                "GITFLEET_UPSTREAM_CRASH_PHASE",
                serde_json::to_string(&phase).unwrap(),
            )
            .output()
            .unwrap();
        assert_eq!(
            output.status.signal(),
            Some(libc::SIGKILL),
            "{phase:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let (root, workspace, worktree): (PathBuf, String, String) =
            serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
        let f = Fixture {
            repo: root.join("repo"),
            root: root.clone(),
        };
        let options = Options {
            global_config: Some(root.join("global.config")),
            signer_home: Some(root.join("home")),
            remote: TransportOptions {
                local_fixture_root: Some(root.clone()),
                ca_file: None,
                ssh: None,
            },
            ..Options::default()
        };
        let before = tree_bytes(&f.repo);
        let app = Application::open(&root.join("state"), options.clone()).unwrap();
        let mut t = Test {
            f,
            app,
            workspace,
            worktree,
            remote: root.join("remote.git"),
            options,
        };
        let Data::Workspaces(list) = t
            .app
            .request(t.app.epoch(), Request::Workspaces {})
            .unwrap()
            .data
        else {
            panic!()
        };
        assert_eq!(list.upstream_recovery.len(), 1);
        assert_eq!(before, tree_bytes(&t.f.repo));
        let id = list.upstream_recovery[0].operation_id.clone();
        assert_eq!(
            t.app
                .request(
                    t.app.epoch(),
                    Request::Remove {
                        workspace: t.workspace.clone()
                    }
                )
                .unwrap_err()
                .kind,
            ErrorKind::Busy
        );
        let Data::Review(r) = t
            .app
            .request(
                t.app.epoch(),
                Request::ReviewUpstreamRecovery {
                    workspace: t.workspace.clone(),
                    operation: id,
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let published = phase == UpstreamPhase::ConfigPublished;
        assert!(r.summary.contains(if published {
            "observe_configuration"
        } else {
            "abort_unpublished"
        }));
        let recovered = upstream_confirm(&mut t, &r);
        assert_eq!(
            recovered.phase,
            if published {
                UpstreamPhase::ObservedConfiguration
            } else {
                UpstreamPhase::Aborted
            }
        );
        let after = tree_bytes(&t.f.repo);
        let mut expected = before;
        expected.remove(std::path::Path::new(".git/config.lock"));
        assert_eq!(expected, after);
        reopen(&mut t);
        assert_eq!(after, tree_bytes(&t.f.repo));
    }
}
#[cfg(feature = "write-test-support")]
#[test]
fn upstream_storage_failure_after_rename_and_foreign_lock_recovery_stay_truthful() {
    use gitfleet_core::{remote::upstream::*, writes::TestFaultKind};
    let mut t = setup(true);
    fs::write(t.f.repo.join(".gitfleet-m2-fixture"), b"fixture").unwrap();
    trust(&mut t);
    let observation = observed(&mut t);
    let r = upstream_review(&mut t, &observation.plan.id);
    t.app.set_upstream_test_fault(
        UpstreamPhase::ConfigPublished,
        TestFaultKind::StorageFailure,
    );
    let value = upstream_confirm(&mut t, &r);
    assert_eq!(value.phase, UpstreamPhase::Uncertain);
    assert_eq!(value.configuration, UpstreamStatus::Updated);
    let before = tree_bytes(&t.f.repo);
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewUpstreamRecovery {
                workspace: t.workspace.clone(),
                operation: value.plan.transport.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    let lock = t.f.repo.join(".git/config.lock");
    fs::write(&lock, b"foreign").unwrap();
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(fs::read(&lock).unwrap(), b"foreign");
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewUpstreamRecovery {
                workspace: t.workspace.clone(),
                operation: value.plan.transport.id.clone(),
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert!(r.summary.contains("manual"));
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(fs::read(&lock).unwrap(), b"foreign");
    fs::remove_file(lock).unwrap();
    let Data::Review(r) = t
        .app
        .request(
            t.app.epoch(),
            Request::ReviewUpstreamRecovery {
                workspace: t.workspace.clone(),
                operation: value.plan.transport.id,
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(
        upstream_confirm(&mut t, &r).phase,
        UpstreamPhase::ObservedConfiguration
    );
    assert_eq!(before, tree_bytes(&t.f.repo));
}

#[test]
fn upstream_shared_config_is_visible_to_linked_worktrees_without_touching_either_checkout() {
    let mut t = setup(true);
    let linked = t.f.root.join("linked");
    t.f.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "other",
        linked.to_str().unwrap(),
    ]);
    trust(&mut t);
    let observation = observed(&mut t);
    let before = tree_bytes(&t.f.repo);
    let other = tree_bytes(&linked);
    let r = upstream_review(&mut t, &observation.plan.id);
    let value = upstream_confirm(&mut t, &r);
    assert_eq!(
        value.phase,
        gitfleet_core::remote::upstream::UpstreamPhase::Complete,
        "{:?}",
        value.issue
    );
    assert_eq!(
        t.f.git(&[
            "-C",
            linked.to_str().unwrap(),
            "config",
            "--get",
            "branch.main.remote"
        ]),
        b"origin\n"
    );
    let mut after = tree_bytes(&t.f.repo);
    after.insert(
        std::path::PathBuf::from(".git/config"),
        before[std::path::Path::new(".git/config")].clone(),
    );
    assert_eq!(before, after);
    assert_eq!(other, tree_bytes(&linked));
}
#[test]
fn upstream_restart_revokes_setup_capability_without_creating_a_config_lock() {
    let mut t = setup(true);
    trust(&mut t);
    let observation = observed(&mut t);
    let r = upstream_review(&mut t, &observation.plan.id);
    let (cap, _) = t.app.begin_confirmation(&r.id).unwrap();
    let before = tree_bytes(&t.f.repo);
    reopen(&mut t);
    assert!(t.app.confirm(cap, true, Cancellation::default()).is_err());
    assert_eq!(before, tree_bytes(&t.f.repo));
    let Data::OperationAudit(page) = t
        .app
        .request(
            t.app.epoch(),
            Request::OperationAudit {
                workspace: t.workspace.clone(),
                query: OperationAuditQuery {
                    offset: 0,
                    worktree: None,
                    operation_kind: Some(OperationAuditKind::UpstreamSetup),
                    result: Some(OperationAuditResult::Interrupted),
                    since_ms: None,
                    until_ms: None,
                },
            },
        )
        .unwrap()
        .data
    else {
        panic!()
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].result, OperationAuditResult::Interrupted);
    assert!(!t.f.repo.join(".git/config.lock").exists());
}

#[path = "support/ssh_transport.rs"]
mod ssh_transport_tests;
