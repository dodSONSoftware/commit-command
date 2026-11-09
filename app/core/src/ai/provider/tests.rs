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
                        let _ = stream.write_all(&body);
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
        let approved = Plan::inference(
            &connection(Provider::Ollama, &server.address),
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
