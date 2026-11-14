use super::*;
fn connection(provider: Provider, endpoint: &str) -> ConnectionDraft {
    ConnectionDraft {
        id: None,
        name: "fixture".into(),
        provider,
        endpoint: endpoint.into(),
        timeout_ms: 1000,
        authentication: Default::default(),
    }
}
#[test]
fn source_request_fingerprint_binds_profile_identity_and_revision() {
    use crate::ai::{config::Profile, context::Bundle, template::Template};
    let connection = connection(Provider::Ollama, "http://127.0.0.1:1/");
    let mut profile = Profile {
        id: "profile-one".into(),
        revision: "revision-one".into(),
        name: "Fixture".into(),
        connection: "connection".into(),
        model: "fixture-model".into(),
        system_message: "Fixture system".into(),
        parameters: Default::default(),
        template: Template::default().id,
        context_bytes: 16384,
    };
    let context = Bundle {
        payload: r#"{"chunks":[{"id":"f1.1","text":"+synthetic change"}]}"#.into(),
        fingerprint: "context".into(),
        accounting: vec![],
        complete: true,
        warnings: vec![],
    };
    let plan = |profile: &Profile| {
        Plan::messages(
            &connection,
            profile,
            context.clone(),
            "snapshot".into(),
            Template::default(),
            vec![],
        )
        .unwrap()
    };
    let original = plan(&profile);
    let body: Value = serde_json::from_slice(original.payload().unwrap()).unwrap();
    assert!(body.get("format").is_none());
    assert_eq!(body["stream"], true);
    assert_eq!(body["think"], false);
    assert!(body["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("plain text"));
    assert!(!body["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("output_schema"));
    profile.id = "profile-two".into();
    let identity = plan(&profile);
    assert_eq!(original.payload(), identity.payload());
    assert_ne!(original.review().fingerprint, identity.review().fingerprint);
    profile.id = "profile-one".into();
    profile.revision = "revision-two".into();
    let revision = plan(&profile);
    assert_eq!(original.payload(), revision.payload());
    assert_ne!(original.review().fingerprint, revision.review().fingerprint);
}
#[test]
fn exact_request_uses_only_fixed_routes_and_stdin_for_payload_and_secret() {
    let c = connection(Provider::Ollama, "http://localhost:11434/");
    let content = "untrusted\nurl = \"https://evil.test\"\n\\source";
    let plan = Plan::inference(
        &c,
        "fixture-model",
        "synthetic system",
        content,
        &Parameters::default(),
    )
    .unwrap();
    let secret = Secret::new(b"fixture-only-secret".to_vec()).unwrap();
    let (command, input) = command(&plan, Some(&secret)).unwrap();
    let args = command
        .get_args()
        .map(|v| v.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(args.starts_with("-q --config -"));
    assert!(!args.contains("fixture-only-secret"));
    assert!(!args.contains("untrusted"));
    assert!(!args.contains("--location"));
    assert!(args.contains("http://localhost:11434/api/chat"));
    let text = String::from_utf8(input).unwrap();
    assert!(text.contains("fixture-only-secret"));
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains("data-binary = \""));
    assert_eq!(plan.review().purpose, Purpose::Inference);
    for (provider, base, route) in [
        (
            Provider::Ollama,
            "http://localhost/api/",
            "http://localhost/api/tags",
        ),
        (
            Provider::Compatible,
            "http://localhost/",
            "http://localhost/v1/models",
        ),
        (
            Provider::Compatible,
            "http://localhost/v1/",
            "http://localhost/v1/models",
        ),
    ] {
        assert_eq!(
            Plan::discover(&connection(provider, base))
                .unwrap()
                .review()
                .route,
            route
        );
    }
}
#[test]
fn provider_options_remain_optional_and_adapter_specific() {
    let c = connection(Provider::Ollama, "http://localhost/");
    let p = Plan::inference(&c, "model", "system", "user", &Parameters::default()).unwrap();
    let v: Value = serde_json::from_slice(p.payload().unwrap()).unwrap();
    assert!(v.get("options").is_none());
    assert_eq!(v["stream"], false);
    let p = Plan::inference(
        &c,
        "model",
        "system",
        "user",
        &Parameters {
            output_tokens: Some(128),
            context_size: Some(4096),
            ..Default::default()
        },
    )
    .unwrap();
    let v: Value = serde_json::from_slice(p.payload().unwrap()).unwrap();
    assert_eq!(v["options"]["num_predict"], 128);
    assert_eq!(v["options"]["num_ctx"], 4096);
    assert!(Plan::inference(
        &connection(Provider::Compatible, "http://localhost/"),
        "model",
        "system",
        "user",
        &Parameters {
            top_k: Some(40),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn responses_refuse_partial_tools_fallback_and_invalid_models() {
    let good = json!({"model":"model","done":true,"done_reason":"stop","message":{"role":"assistant","content":"{}"},"eval_count":3});
    assert_eq!(
        completion(Provider::Ollama, &good, "model", Duration::ZERO)
            .unwrap()
            .output_tokens,
        Some(3)
    );
    for field in [
        json!({"done":false}),
        json!({"done_reason":"length"}),
        json!({"model":"fallback"}),
        json!({"message":{"role":"assistant","content":"{}","tool_calls":[{}]}}),
    ] {
        let mut bad = good.clone();
        for (k, v) in field.as_object().unwrap() {
            bad[k] = v.clone();
        }
        assert!(completion(Provider::Ollama, &bad, "model", Duration::ZERO).is_err());
    }
    assert!(models(Provider::Ollama, &json!({"models":[{"name":"bad\nmodel"}]})).is_err());
    let models = models(
        Provider::Compatible,
        &json!({"data":[{"id":"z"},{"id":"a"},{"id":"a"}]}),
    )
    .unwrap();
    assert_eq!(
        models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "z"]
    );
}

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
type CapturedRequest = Arc<Mutex<Option<(String, Vec<u8>)>>>;
struct Server {
    address: String,
    request: CapturedRequest,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(status: u16, extra: String, body: Vec<u8>, hold: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = format!("http://{}/", listener.local_addr().unwrap());
        let request = Arc::new(Mutex::new(None));
        let captured = request.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let cancelled = stop.clone();
        let thread = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline && !cancelled.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // macOS can inherit the listener's nonblocking mode.
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut bytes = Vec::new();
                        let mut buf = [0; 4096];
                        let head = loop {
                            let n = stream.read(&mut buf).unwrap_or(0);
                            if n == 0 {
                                return;
                            }
                            bytes.extend_from_slice(&buf[..n]);
                            if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                                break at + 4;
                            }
                            if bytes.len() > 16384 {
                                return;
                            }
                        };
                        let headers = String::from_utf8(bytes[..head].to_vec()).unwrap();
                        let length = headers
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|s| s.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        assert!(length <= MAX_REQUEST);
                        while bytes.len() < head + length {
                            let n = stream.read(&mut buf).unwrap_or(0);
                            if n == 0 {
                                return;
                            }
                            bytes.extend_from_slice(&buf[..n]);
                        }
                        *captured.lock().unwrap() =
                            Some((headers, bytes[head..head + length].to_vec()));
                        while hold
                            && !cancelled.load(Ordering::Relaxed)
                            && Instant::now() < deadline
                        {
                            thread::sleep(Duration::from_millis(5));
                        }
                        let header=format!("HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n{extra}\r\n",body.len());
                        let _ = stream.write_all(header.as_bytes());
                        if extra.contains("X-Fixture-Stream") {
                            for line in body.split_inclusive(|b| *b == b'\n') {
                                let _ = stream.write_all(line);
                                thread::sleep(Duration::from_millis(100));
                            }
                        } else {
                            let _ = stream.write_all(&body);
                        }
                        return;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => return,
                }
            }
        });
        Self {
            address,
            request,
            stop,
            thread: Some(thread),
        }
    }
    fn captured(&self) -> (String, Vec<u8>) {
        self.request.lock().unwrap().clone().unwrap()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
fn good(provider: Provider) -> Vec<u8> {
    serde_json::to_vec(&if provider==Provider::Ollama{json!({"model":"fixture-model","done":true,"done_reason":"stop","message":{"role":"assistant","content":"{\"subject\":\"fix: synthetic change\",\"body\":\"\"}"},"eval_count":8})}else{json!({"model":"fixture-model","choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"{\"subject\":\"fix: synthetic change\",\"body\":\"\"}"}}],"usage":{"prompt_tokens":10,"completion_tokens":8}})}).unwrap()
}
#[test]
fn actual_both_provider_routes_preserve_exact_json_and_auth_without_redirects() {
    for provider in [Provider::Ollama, Provider::Compatible] {
        let server = Server::new(200, String::new(), good(provider), false);
        let mut c = connection(provider, &server.address);
        c.id = Some(uuid::Uuid::new_v4().to_string());
        c.authentication = super::super::credentials::Authentication::StoredBearer;
        let source = "synthetic source\nheader = \"injection\"\\\t";
        let plan = Plan::inference(
            &c,
            "fixture-model",
            "synthetic system",
            source,
            &Parameters::default(),
        )
        .unwrap();
        let expected = plan.payload().unwrap().to_vec();
        let result = plan
            .approve()
            .execute(
                Some(Secret::new(b"synthetic-bearer-only".to_vec()).unwrap()),
                Cancellation::default(),
            )
            .unwrap();
        assert!(matches!(result, ProviderResult::Completion(_)));
        let (headers, body) = server.captured();
        assert!(headers
            .to_ascii_lowercase()
            .contains("authorization: bearer synthetic-bearer-only"));
        assert_eq!(body, expected);
        assert_eq!(
            serde_json::from_slice::<Value>(&body).unwrap()["messages"][1]["content"],
            source
        );
        assert!(headers.starts_with(if provider == Provider::Ollama {
            "POST /api/chat "
        } else {
            "POST /v1/chat/completions "
        }));
    }
}
#[test]
fn actual_discovery_sorts_models_and_rejects_redirect_auth_malformed_and_size() {
    let server = Server::new(
        200,
        String::new(),
        br#"{"models":[{"name":"z","size":12},{"name":"a"}]}"#.to_vec(),
        false,
    );
    let result = Plan::discover(&connection(Provider::Ollama, &server.address))
        .unwrap()
        .approve()
        .execute(None, Cancellation::default())
        .unwrap();
    let ProviderResult::Models(models) = result else {
        panic!()
    };
    assert_eq!(models[0].id, "a");
    assert!(server.captured().0.starts_with("GET /api/tags "));
    let target = TcpListener::bind("127.0.0.1:0").unwrap();
    target.set_nonblocking(true).unwrap();
    for (status, extra, body) in [
        (
            302,
            format!(
                "Location: http://{}/stolen\r\n",
                target.local_addr().unwrap()
            ),
            b"redirect body".to_vec(),
        ),
        (
            401,
            String::new(),
            b"synthetic-secret-error-sentinel".to_vec(),
        ),
        (200, String::new(), b"{malformed".to_vec()),
        (200, String::new(), vec![b'x'; MAX_RESPONSE + 1]),
    ] {
        let server = Server::new(status, extra, body, false);
        let e = Plan::discover(&connection(Provider::Ollama, &server.address))
            .unwrap()
            .approve()
            .execute(None, Cancellation::default())
            .unwrap_err();
        assert!(!e.to_string().contains("sentinel"));
        assert!(!e.to_string().contains("malformed\""));
    }
    assert_eq!(
        target.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
#[test]
fn actual_running_inference_cancels_and_times_out_with_bounded_cleanup() {
    for cancel_it in [true, false] {
        let server = Server::new(200, String::new(), good(Provider::Ollama), true);
        let mut c = connection(Provider::Ollama, &server.address);
        c.timeout_ms = if cancel_it { 2000 } else { 150 };
        let approved = Plan::inference(
            &c,
            "fixture-model",
            "system",
            "synthetic",
            &Parameters::default(),
        )
        .unwrap()
        .approve();
        let cancel = Cancellation::default();
        let child_cancel = cancel.clone();
        let worker = thread::spawn(move || approved.execute(None, child_cancel));
        let start = Instant::now();
        while server.request.lock().unwrap().is_none() {
            assert!(start.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(5));
        }
        if cancel_it {
            cancel.cancel();
        }
        let e = worker.join().unwrap().unwrap_err();
        assert_eq!(
            e.kind,
            if cancel_it {
                ErrorKind::Cancelled
            } else {
                ErrorKind::Timeout
            }
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        server.stop.store(true, Ordering::Relaxed);
    }
}
#[test]
fn declined_or_expired_authority_cannot_contact_provider() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let c = connection(
        Provider::Ollama,
        &format!("http://{}/", listener.local_addr().unwrap()),
    );
    let plan = Plan::discover(&c).unwrap();
    let review = plan.review();
    assert_eq!(review.request_bytes, 0);
    assert!(plan.payload().is_none());
    let mut approved = plan.approve();
    approved.at = Instant::now() - Duration::from_secs(121);
    assert_eq!(
        approved
            .execute(None, Cancellation::default())
            .unwrap_err()
            .kind,
        ErrorKind::StaleReview
    );
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn independent_jobs_enforce_global_and_connection_limits_and_shutdown_discards_results() {
    use crate::ai::jobs::{Jobs, Limits, Phase};
    let servers = (0..3)
        .map(|_| Server::new(200, String::new(), good(Provider::Ollama), true))
        .collect::<Vec<_>>();
    let mut jobs = Jobs::default();
    let mut ids = Vec::new();
    for (i, server) in servers.iter().enumerate() {
        // Held calls must outlive the two-second overlap proof, including time
        // waiting for the global process permit during the complete parallel suite.
        let mut destination = connection(Provider::Ollama, &server.address);
        destination.timeout_ms = 5000;
        let approved = Plan::inference(
            &destination,
            "fixture-model",
            "system",
            "synthetic",
            &Parameters::default(),
        )
        .unwrap()
        .approve();
        ids.push(
            jobs.start(
                approved,
                None,
                if i < 2 {
                    "same-connection"
                } else {
                    "other-connection"
                }
                .into(),
                Limits {
                    global: 2,
                    connection: 1,
                },
                Cancellation::default(),
            )
            .unwrap(),
        );
    }
    let start = Instant::now();
    loop {
        let list = jobs.list().unwrap();
        if list
            .iter()
            .filter(|j| matches!(j.phase, Phase::Running))
            .count()
            == 2
        {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        thread::sleep(Duration::from_millis(5));
    }
    let start = Instant::now();
    while servers
        .iter()
        .filter(|s| s.request.lock().unwrap().is_some())
        .count()
        < 2
    {
        assert!(start.elapsed() < Duration::from_secs(2));
        thread::sleep(Duration::from_millis(5));
    }
    assert!(servers[2].request.lock().unwrap().is_some());
    assert_eq!(
        servers[..2]
            .iter()
            .filter(|s| s.request.lock().unwrap().is_some())
            .count(),
        1
    );
    jobs.set_limits(Limits {
        global: 1,
        connection: 1,
    })
    .unwrap();
    for id in &ids {
        jobs.cancel(id).unwrap();
    }
    jobs.reset();
    let start = Instant::now();
    jobs.wait_shutdown().unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(jobs.list().unwrap().is_empty());
}

#[test]
fn empty_tool_list_is_not_a_tool_invocation_and_bidi_model_ids_are_refused() {
    let value = json!({"model":"model","done":true,"done_reason":"stop","message":{"role":"assistant","content":"{}","tool_calls":[]}});
    completion(Provider::Ollama, &value, "model", Duration::ZERO).unwrap();
    assert!(models(
        Provider::Ollama,
        &json!({"models":[{"name":"model\u{202e}spoof"}]})
    )
    .is_err());
}

struct FixtureStore {
    account: String,
    fail: bool,
    calls: std::sync::atomic::AtomicUsize,
}
impl crate::ai::credentials::SecretStore for FixtureStore {
    fn get(&self, binding: &crate::ai::credentials::Binding, _: Cancellation) -> Result<Secret> {
        assert_eq!(binding.account().unwrap(), self.account);
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            Err(Error::with_kind(
                ErrorKind::Unavailable,
                "Fixture secure store locked; no fallback",
            ))
        } else {
            Secret::new(b"fixture-store-token".to_vec())
        }
    }
    fn set(&self, _: &crate::ai::credentials::Binding, _: &Secret) -> Result<()> {
        Err(invalid("Fixture does not accept writes"))
    }
    fn delete(&self, _: &crate::ai::credentials::Binding) -> Result<()> {
        Err(invalid("Fixture does not accept deletion"))
    }
}
#[test]
fn stored_authentication_is_exact_and_failure_never_contacts_an_unauthenticated_provider() {
    use crate::ai::{
        credentials::{Authentication, Binding},
        jobs::{Jobs, Limits, Phase},
    };
    for fail in [false, true] {
        let server = Server::new(200, String::new(), good(Provider::Ollama), false);
        let mut c = connection(Provider::Ollama, &server.address);
        let id = uuid::Uuid::new_v4().to_string();
        c.id = Some(id.clone());
        c.authentication = Authentication::StoredBearer;
        let store = Arc::new(FixtureStore {
            account: Binding::new(&id, &server.address)
                .unwrap()
                .account()
                .unwrap(),
            fail,
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let approved = Plan::inference(
            &c,
            "fixture-model",
            "system",
            "synthetic",
            &Parameters::default(),
        )
        .unwrap()
        .approve();
        assert_eq!(store.calls.load(Ordering::SeqCst), 0);
        let mut jobs = Jobs::default();
        jobs.start(
            approved,
            Some(store.clone()),
            "connection".into(),
            Limits::default(),
            Cancellation::default(),
        )
        .unwrap();
        let start = Instant::now();
        let job = loop {
            let j = jobs.list().unwrap().remove(0);
            if matches!(j.phase, Phase::Complete | Phase::Failed) {
                break j;
            }
            assert!(start.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(store.calls.load(Ordering::SeqCst), 1);
        assert!(!serde_json::to_string(&job)
            .unwrap()
            .contains("fixture-store-token"));
        if fail {
            assert!(matches!(job.phase, Phase::Failed));
            assert!(server.request.lock().unwrap().is_none());
        } else {
            assert!(matches!(job.phase, Phase::Complete));
            assert!(server
                .captured()
                .0
                .to_ascii_lowercase()
                .contains("authorization: bearer fixture-store-token"));
        }
        jobs.reset();
        jobs.wait_shutdown().unwrap();
    }
}
#[test]
fn absent_or_unapproved_authentication_cannot_reach_the_provider() {
    use crate::ai::credentials::Authentication;
    let server = Server::new(200, String::new(), good(Provider::Ollama), false);
    let mut c = connection(Provider::Ollama, &server.address);
    c.id = Some(uuid::Uuid::new_v4().to_string());
    c.authentication = Authentication::StoredBearer;
    let e = Plan::discover(&c)
        .unwrap()
        .approve()
        .execute(None, Cancellation::default())
        .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unavailable);
    assert!(server.request.lock().unwrap().is_none());
    c.authentication = Authentication::None;
    let e = Plan::discover(&c)
        .unwrap()
        .approve()
        .execute(
            Some(Secret::new(b"unapproved-auth".to_vec()).unwrap()),
            Cancellation::default(),
        )
        .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unavailable);
    assert!(server.request.lock().unwrap().is_none());
}

#[test]
fn reflected_credential_is_refused_before_result_reaches_renderer() {
    for content in [
        "fixture-store-token",
        "{\"subject\":\"fixture-store-token\"}",
    ] {
        let body=json!({"model":"fixture-model","done":true,"message":{"role":"assistant","content":content}}).to_string();
        let server = Server::new(200, String::new(), body.into_bytes(), false);
        let mut c = connection(Provider::Ollama, &server.address);
        c.id = Some(uuid::Uuid::new_v4().to_string());
        c.authentication = crate::ai::credentials::Authentication::StoredBearer;
        let error = Plan::inference(
            &c,
            "fixture-model",
            "system",
            "synthetic",
            &Parameters::default(),
        )
        .unwrap()
        .approve()
        .execute(
            Some(Secret::new(b"fixture-store-token".to_vec()).unwrap()),
            Cancellation::default(),
        )
        .unwrap_err();
        assert!(!error.to_string().contains("fixture-store-token"));
        assert!(
            error.to_string().contains("reflected authentication"),
            "{error}"
        );
    }
}

#[test]
fn truncated_answers_are_actionable_refusals_even_when_the_partial_message_looks_valid() {
    for (provider, value) in [
        (
            Provider::Ollama,
            json!({"model":"model","done":true,"done_reason":"length","message":{"role":"assistant","content":"{\"subject\":\"fix: never-display-partial\",\"body\":\"\"}"}}),
        ),
        (
            Provider::Compatible,
            json!({"model":"model","choices":[{"finish_reason":"length","message":{"role":"assistant","content":"{\"subject\":\"fix: never-display-partial\",\"body\":\"\"}"}}]}),
        ),
    ] {
        let error = completion(provider, &value, "model", Duration::ZERO).unwrap_err();
        assert_eq!(error.kind, ErrorKind::OutputLimit);
        assert!(error.to_string().contains("Maximum output tokens"));
        assert!(!error.to_string().contains("never-display-partial"));
    }
}
#[test]
fn unfinished_or_unrecognized_finish_metadata_never_exposes_the_provider_value() {
    for (done, reason) in [(false, "stop"), (true, "private-provider-value")] {
        let value = json!({"model":"model","done":done,"done_reason":reason,"message":{"role":"assistant","content":"never-display-content"}});
        let error = completion(Provider::Ollama, &value, "model", Duration::ZERO).unwrap_err();
        assert!(!error.to_string().contains("private-provider-value"));
        assert!(!error.to_string().contains("never-display-content"));
        assert!(error.to_string().contains("discarded"));
    }
}

#[test]
fn blank_parameters_omit_ollama_options_but_explicit_zero_is_preserved() {
    let c = connection(Provider::Ollama, "http://localhost:11434/");
    let defaults = Plan::inference(
        &c,
        "fixture-model",
        "system",
        "user",
        &Parameters::default(),
    )
    .unwrap();
    let body: Value = serde_json::from_slice(defaults.payload().unwrap()).unwrap();
    assert!(body.get("options").is_none());
    let explicit = Plan::inference(
        &c,
        "fixture-model",
        "system",
        "user",
        &Parameters {
            temperature: Some(0.0),
            ..Default::default()
        },
    )
    .unwrap();
    let body: Value = serde_json::from_slice(explicit.payload().unwrap()).unwrap();
    assert_eq!(body["options"], json!({"temperature": 0.0}));
}

#[test]
fn empty_ollama_request_loads_only_the_selected_model_without_source() {
    let server = Server::new(
        200,
        String::new(),
        json!({"model":"fixture-model","done":true,"done_reason":"load"})
            .to_string()
            .into_bytes(),
        false,
    );
    let c = connection(Provider::Ollama, &server.address);
    let parameters = Parameters {
        context_size: Some(8192),
        ..Default::default()
    };
    let plan = Plan::load_model(&c, "fixture-model", &parameters).unwrap();
    assert_eq!(plan.review().purpose, Purpose::LoadModel);
    assert_eq!(plan.timeout, Duration::from_secs(120));
    let result = plan
        .approve()
        .execute(None, Cancellation::default())
        .unwrap();
    assert!(matches!(result, ProviderResult::ModelLoaded { model } if model == "fixture-model"));
    let (headers, raw) = server.captured();
    assert!(headers.starts_with("POST /api/generate "));
    let body: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        body,
        json!({"model":"fixture-model","stream":false,"keep_alive":"10m","options":{"num_ctx":8192}})
    );
}

#[test]
fn model_loading_requires_an_exact_success_acknowledgment() {
    for reply in [
        json!({"model":"fixture-model","done":false}),
        json!({"model":"other-model","done":true}),
    ] {
        let server = Server::new(200, String::new(), reply.to_string().into_bytes(), false);
        let plan = Plan::load_model(
            &connection(Provider::Ollama, &server.address),
            "fixture-model",
            &Parameters::default(),
        )
        .unwrap();
        let error = plan
            .approve()
            .execute(None, Cancellation::default())
            .unwrap_err();
        assert!(error.to_string().contains("did not confirm"));
    }
}

#[test]
fn loading_timeout_reports_loading_instead_of_a_generic_request_failure() {
    let server = Server::new(
        200,
        String::new(),
        json!({"model":"fixture-model","done":true})
            .to_string()
            .into_bytes(),
        true,
    );
    let mut plan = Plan::load_model(
        &connection(Provider::Ollama, &server.address),
        "fixture-model",
        &Parameters::default(),
    )
    .unwrap();
    plan.timeout = Duration::from_millis(150);
    let error = plan
        .approve()
        .execute(None, Cancellation::default())
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Timeout);
    assert!(error.to_string().contains("Model loading timed out"));
    assert!(error.to_string().contains("has not confirmed"));
    server.stop.store(true, Ordering::Relaxed);
}

#[test]
fn ollama_stream_decodes_fragmented_unicode_and_reports_server_metrics() {
    let mut stream = OllamaStream::default();
    let bytes = concat!(
        "{\"model\":\"fixture-model\",\"message\":{\"role\":\"assistant\",\"content\":\"fix: café\"},\"done\":false}\n",
        "{\"model\":\"fixture-model\",\"message\":{\"content\":\"\\n\\nDetails.\"},\"done\":true,\"done_reason\":\"stop\",\"prompt_eval_count\":100,\"eval_count\":20,\"eval_duration\":2000000000,\"total_duration\":3000000000}\n"
    ).as_bytes();
    for chunk in bytes.chunks(3) {
        stream.push(chunk);
    }
    let progress = stream.progress();
    assert_eq!(progress.message, "fix: café\n\nDetails.");
    assert_eq!(progress.tokens_per_second, Some(10.0));
    assert_eq!(progress.total_duration_ms, Some(3000.0));
    assert_eq!(progress.input_tokens, Some(100));
    assert_eq!(progress.output_tokens, Some(20));
    let completed = stream.finish().unwrap();
    assert_eq!(
        completion(
            Provider::Ollama,
            &completed,
            "fixture-model",
            Duration::ZERO
        )
        .unwrap()
        .content,
        "fix: café\n\nDetails."
    );
}
#[test]
fn ollama_stream_rejects_truncation_invalid_frames_and_model_changes() {
    for bytes in [
        b"not json\n".as_slice(),
        br#"{"model":"a","message":{"content":"partial"},"done":false}"#.as_slice(),
        b"{\"model\":\"a\",\"done\":false}\n{\"model\":\"b\",\"done\":true}\n".as_slice(),
        b"{\"error\":\"failure\"}\n".as_slice(),
    ] {
        let mut stream = OllamaStream::default();
        stream.push(bytes);
        assert!(stream.finish().is_err());
    }
    let mut stream = OllamaStream::default();
    stream.push(b"{\"model\":\"fixture-model\",\"message\":{\"content\":\"partial\"},\"done\":true,\"done_reason\":\"length\"}\n");
    assert_eq!(stream.progress().message, "partial");
    assert_eq!(
        completion(
            Provider::Ollama,
            &stream.finish().unwrap(),
            "fixture-model",
            Duration::ZERO
        )
        .unwrap_err()
        .kind,
        ErrorKind::OutputLimit
    );
}

#[test]
fn streamed_transport_delivers_message_before_http_completion() {
    use crate::ai::{config::Profile, context::Bundle, template::Template};
    let body = b"{\"model\":\"fixture-model\",\"message\":{\"role\":\"assistant\",\"content\":\"docs: update design documents and handoff prompts for Interstellar Executive\"},\"done\":false}\n{\"model\":\"fixture-model\",\"message\":{\"content\":\"\\n\\nDetails.\"},\"done\":true,\"done_reason\":\"stop\",\"prompt_eval_count\":100,\"eval_count\":20,\"eval_duration\":2000000000,\"total_duration\":3000000000}\n".to_vec();
    let server = Server::new(200, "X-Fixture-Stream: true\r\n".into(), body, false);
    let template = Template::default();
    let profile = Profile {
        id: "p".into(),
        revision: "r".into(),
        name: "Fixture".into(),
        connection: "c".into(),
        model: "fixture-model".into(),
        system_message: "Write a commit message.".into(),
        parameters: Parameters::default(),
        template: template.id.clone(),
        context_bytes: 16384,
    };
    let context = Bundle {
        payload: r#"{"chunks":[]}"#.into(),
        fingerprint: "context".into(),
        accounting: vec![],
        complete: true,
        warnings: vec![],
    };
    let plan = Plan::messages(
        &connection(Provider::Ollama, &server.address),
        &profile,
        context,
        "snapshot".into(),
        template,
        vec![],
    )
    .unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let worker = thread::spawn(move || {
        plan.approve().execute_stream(
            None,
            Cancellation::default(),
            Box::new(move |p| {
                let _ = send.send(p);
            }),
        )
    });
    let first = receive.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(
        first.message,
        "docs: update design documents and handoff prompts for Interstellar Executive"
    );
    assert!(first.output_tokens.is_none());
    assert!(!worker.is_finished());
    let ProviderResult::Candidates(result) = worker.join().unwrap().unwrap() else {
        panic!()
    };
    assert_eq!(
        result.candidates[0].candidate.message.subject,
        "docs: update design documents and handoff prompts for Interstellar Executive"
    );
    assert_eq!(result.candidates[0].candidate.message.body, "Details.");
    assert_eq!(result.output_tokens, Some(20));
    let (_, sent) = server.request.lock().unwrap().clone().unwrap();
    let sent: Value = serde_json::from_slice(&sent).unwrap();
    assert_eq!(sent["stream"], true);
    assert_eq!(sent["think"], false);
    assert!(sent.get("format").is_none());
}

#[test]
fn thinking_defaults_off_and_is_sent_as_an_ollama_top_level_option() {
    let parameters: Parameters = serde_json::from_str("{}").unwrap();
    assert!(!parameters.thinking);
    let c = connection(Provider::Ollama, "http://localhost/");
    for enabled in [false, true] {
        let parameters = Parameters {
            thinking: enabled,
            ..Default::default()
        };
        let plan = Plan::inference(&c, "model", "system", "user", &parameters).unwrap();
        let body: Value = serde_json::from_slice(plan.payload().unwrap()).unwrap();
        assert_eq!(body["think"], enabled);
        assert!(body.get("options").is_none());
        let saved = toml::to_string(&parameters).unwrap();
        assert_eq!(
            toml::from_str::<Parameters>(&saved).unwrap().thinking,
            enabled
        );
    }
}

#[test]
fn stored_bearer_requires_tls_outside_loopback() {
    for (endpoint, permitted) in [
        ("http://127.0.0.2/", true),
        ("http://[::1]/", true),
        ("http://localhost/", true),
        ("https://provider.example/", true),
        ("http://192.0.2.1/", false),
        ("http://[fd00::1]/", false),
        ("http://localhost.example/", false),
        ("http://provider.example/", false),
    ] {
        let mut draft = connection(Provider::Ollama, endpoint);
        assert!(Plan::discover(&draft).is_ok());
        draft.id = Some(uuid::Uuid::new_v4().to_string());
        draft.authentication = crate::ai::credentials::Authentication::StoredBearer;
        assert_eq!(Plan::discover(&draft).is_ok(), permitted, "{endpoint}");
    }
}

#[test]
fn finished_jobs_release_admission_without_expiring_recent_results() {
    use crate::ai::jobs::{Jobs, Limits, Phase};
    let mut jobs = Jobs::default();
    let mut first = None;
    for number in 0..40 {
        let server = Server::new(
            200,
            String::new(),
            br#"{"models":[{"name":"fixture-model"}]}"#.to_vec(),
            false,
        );
        let plan = Plan::discover(&connection(Provider::Ollama, &server.address)).unwrap();
        let id = jobs
            .start(
                plan.approve(),
                None,
                "fixture".into(),
                Limits::default(),
                Cancellation::default(),
            )
            .unwrap();
        first.get_or_insert_with(|| id.clone());
        let start = Instant::now();
        loop {
            let retained = jobs.list().unwrap();
            assert!(retained.len() <= 32);
            let current = retained.iter().find(|j| j.id == id).unwrap();
            if matches!(current.phase, Phase::Complete) {
                assert!(current.result.is_some());
                break;
            }
            assert!(
                !matches!(current.phase, Phase::Failed),
                "{}",
                current.error.as_deref().unwrap_or("")
            );
            assert!(start.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
        assert!(server.captured().0.starts_with("GET /api/tags"));
        if number >= 32 {
            assert!(!jobs
                .list()
                .unwrap()
                .iter()
                .any(|j| Some(&j.id) == first.as_ref()));
        }
    }
    assert!(jobs.is_idle().unwrap());
    assert_eq!(jobs.list().unwrap().len(), 32);
}
