use super::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

fn record(listener: &TcpListener, response: &str) -> String {
    let start = Instant::now();
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    && start.elapsed() < Duration::from_secs(5) =>
            {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(e) => panic!("Provider request missing: {e}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    let mut length = 0;
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap();
        }
        request.push_str(&line);
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    request.push_str(std::str::from_utf8(&body).unwrap());
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.len(),
        response
    )
    .unwrap();
    request
}

#[test]
fn settings_source_transfer_and_saved_fill() {
    let f = Fixture::new();
    f.commit_file();
    fs::write(
        f.repo.join("file.txt"),
        "F08 ordinary repository sentinel\n",
    )
    .unwrap();
    let mut desktop = Desktop::open(&f, Options::default());
    let (host, rx) = tests::host();
    let app = tauri::test::mock_builder()
        .manage(host.clone())
        .invoke_handler(tauri::generate_handler![dispatch])
        .build(tauri::generate_context!())
        .unwrap();
    let settings =
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
            .build()
            .unwrap();
    let main = WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
        .build()
        .unwrap();
    let dispatch = |window, request, epoch| {
        tests::invoke(
            window,
            "dispatch",
            json!({"payload":json!({"version":1,"epoch":epoch,"request":request}).to_string()}),
        )
    };
    let saved = TcpListener::bind("127.0.0.1:0").unwrap();
    let alternate = TcpListener::bind("127.0.0.1:0").unwrap();
    for server in [&saved, &alternate] {
        server.set_nonblocking(true).unwrap();
    }
    let connection = |server: &TcpListener| json!({"id":null,"name":"fixture","provider":"ollama","authentication":"none","endpoint":format!("http://{}/",server.local_addr().unwrap()),"timeout_ms":5000});
    let Data::AiSettings(snapshot) = desktop.call(json!({"kind":"ai_settings"})).unwrap().data
    else {
        panic!()
    };
    desktop.call(json!({"kind":"save_ai_settings","revision":snapshot.settings.revision,"fingerprint":snapshot.fingerprint,"change":{"kind":"single_setup","connection":connection(&saved),"model":"fixture-model","parameters":{},"system_message":"Describe changes"}})).unwrap();
    let spec = json!({"selected":[{"path":{"encoding":"utf8","value":"file.txt"},"source":"working_tree"}],"nearby_source":[],"documentation":[],"branch_commits":0,"file_commits":0,"policy":{"max_bytes":16384,"excluded_patterns":[]}});
    let preview = json!({"kind":"preview_ai_context","workspace":desktop.workspace,"worktree":desktop.worktree,"spec":spec});
    let Data::AiContext(captured) = desktop.call(preview.clone()).unwrap().data else {
        panic!()
    };
    let editor = json!({"kind":"review_ai_editor_test","workspace":desktop.workspace,"worktree":desktop.worktree,"spec":spec,"context_fingerprint":captured.bundle.fingerprint,"snapshot_fingerprint":captured.snapshot_fingerprint,"connection":connection(&alternate),"profile":{"id":null,"name":"Unsaved","connection":"draft","model":"fixture-model","system_message":"Describe changes","parameters":{},"template":"draft","context_bytes":16384},"template":gitfleet_core::ai::template::Template::default(),"instructions":[]});
    let mut fill = json!({"kind":"review_ai_messages","workspace":desktop.workspace,"worktree":desktop.worktree,"spec":spec,"context_fingerprint":captured.bundle.fingerprint,"snapshot_fingerprint":captured.snapshot_fingerprint,"one_off_profile":null,"instructions":"Describe selected changes"});
    let suggestions = WebviewWindowBuilder::new(
        &app,
        "ai-suggestions",
        WebviewUrl::App("settings.html".into()),
    )
    .build()
    .unwrap();
    for request in [&preview, &fill, &editor] {
        assert!(dispatch(&settings, request.clone(), desktop.core.epoch())
            .unwrap_err()
            .to_string()
            .contains("not available in Settings"));
    }
    for window in [&main, &suggestions] {
        assert!(dispatch(window, editor.clone(), desktop.core.epoch())
            .unwrap_err()
            .to_string()
            .contains("saved AI profile"));
    }
    assert!(rx.try_recv().is_err());
    let idle = |listener: &TcpListener| {
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        )
    };
    idle(&alternate);
    idle(&saved);

    let mut run =
        |window, request, approve: &mut dyn FnMut(&gitfleet_core::ai::provider::Review) -> bool| {
            dispatch(window, request, desktop.core.epoch()).unwrap();
            let Task::Request {
                epoch,
                request,
                cancel,
                ..
            } = rx.try_recv().unwrap()
            else {
                panic!()
            };
            process_request(&mut desktop.core, &host, epoch, request, 1, cancel, approve)
        };
    let discovery = json!({"kind":"review_ai_discovery","connection":connection(&alternate)});
    let discovery_result = run(&settings, discovery, &mut |_| {
        panic!("Discovery requested source approval")
    })
    .unwrap();
    assert!(matches!(discovery_result.data, Data::Executed(_)));
    let probe = record(&alternate, r#"{"models":[{"name":"fixture-model"}]}"#);
    assert!(probe.starts_with("GET /api/tags "));
    assert!(!probe.contains("F08 ordinary repository sentinel"));
    let check_review = |review: &gitfleet_core::ai::provider::Review| {
        idle(&saved);
        assert_eq!(
            review.endpoint,
            connection(&saved)["endpoint"].as_str().unwrap()
        );
        assert!(review
            .source
            .as_ref()
            .unwrap()
            .accounting
            .iter()
            .any(|row| row.origin == "file.txt"));
    };
    let decline = run(&main, fill.clone(), &mut |review| {
        check_review(review);
        false
    })
    .unwrap_err();
    assert_eq!(decline.kind, gitfleet_core::ErrorKind::Cancelled);
    idle(&saved);
    let changed = run(&main, fill.clone(), &mut |review| {
        check_review(review);
        fs::write(f.repo.join("file.txt"), "changed during approval").unwrap();
        true
    })
    .unwrap_err();
    assert_eq!(changed.kind, gitfleet_core::ErrorKind::StaleReview);
    idle(&saved);
    fs::write(
        f.repo.join("file.txt"),
        "F08 ordinary repository sentinel\n",
    )
    .unwrap();
    let Data::AiContext(refreshed) = run(&main, preview, &mut |_| {
        panic!("Preview requested transfer")
    })
    .unwrap()
    .data
    else {
        panic!()
    };
    fill["snapshot_fingerprint"] = json!(refreshed.snapshot_fingerprint);
    fill["context_fingerprint"] = json!(refreshed.bundle.fingerprint);
    let Data::Executed(result) = run(&main, fill, &mut |review| {
        check_review(review);
        true
    })
    .unwrap()
    .data
    else {
        panic!()
    };
    let job = result.ai_job_id.unwrap();
    let request = record(
        &saved,
        r#"{"model":"fixture-model","done":true,"message":{"role":"assistant","content":"Describe the fixture change"}}"#,
    );
    assert!(request.starts_with("POST /api/chat "));
    assert!(request.contains("F08 ordinary repository sentinel"));
    let start = Instant::now();
    loop {
        let Data::AiJobs(jobs) = desktop.call(json!({"kind":"ai_jobs"})).unwrap().data else {
            panic!()
        };
        let entry = jobs.iter().find(|item| item.id == job).unwrap();
        if json!(entry.phase) == "complete" {
            break;
        }
        assert!(
            entry.error.is_none() && start.elapsed() < Duration::from_secs(5),
            "{:?}",
            entry.error
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    idle(&alternate);
    idle(&saved);
}

#[test]
fn settings_can_keep_testing_provider_after_thirty_two_completions() {
    let fixture = Fixture::new();
    fixture.commit_file();
    let mut desktop = Desktop::open(&fixture, Options::default());
    for _ in 0..40 {
        let provider = TcpListener::bind("127.0.0.1:0").unwrap();
        provider.set_nonblocking(true).unwrap();
        let response=desktop.call(json!({"kind":"review_ai_discovery","connection":{
            "id":null,"name":"Capacity fixture","provider":"ollama","endpoint":format!("http://{}/",provider.local_addr().unwrap()),"timeout_ms":3000,"authentication":"none"
        }})).unwrap();
        let Data::Executed(result) = response.data else {
            panic!()
        };
        let id = result.ai_job_id.unwrap();
        assert!(
            record(&provider, r#"{"models":[{"name":"fixture-model"}]}"#)
                .starts_with("GET /api/tags ")
        );
        let start = Instant::now();
        loop {
            let Data::AiJobs(jobs) = desktop.call(json!({"kind":"ai_jobs"})).unwrap().data else {
                panic!()
            };
            assert!(jobs.len() <= 32);
            let job = jobs.iter().find(|job| job.id == id).unwrap();
            if json!(job.phase) == "complete" {
                assert!(job.result.is_some());
                break;
            }
            assert!(
                job.error.is_none() && start.elapsed() < Duration::from_secs(3),
                "{:?}",
                job.error
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
