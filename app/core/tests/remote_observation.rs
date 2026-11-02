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
    let f = Fixture::new();
    f.commit_file();
    let remote = f.root.join("remote.git");
    f.git(&[
        "-c",
        "init.templateDir=",
        "init",
        "--bare",
        "-q",
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
