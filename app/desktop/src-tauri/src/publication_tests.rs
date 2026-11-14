use super::*;
use crate::test_support as support;
use gitfleet_core::{
    application::{Response, ReviewDisplay},
    writes::Options,
};
use std::fs;
use support::Fixture;
#[path = "../../../core/tests/support/tls.rs"]
mod tls;

struct Desktop {
    core: Application,
    host: Arc<Host>,
    workspace: String,
    worktree: String,
}

impl Desktop {
    fn open(f: &Fixture, options: Options) -> Self {
        let mut core = Application::open(&f.root.join("state"), options).unwrap();
        let picker = core.register_directory(&f.repo).unwrap();
        let mut desktop = Self {
            core,
            host: tests::host().0,
            workspace: String::new(),
            worktree: String::new(),
        };
        desktop
            .call(json!({"kind":"create","name":"Fixture","picker":picker}))
            .unwrap();
        let Data::Workspaces(list) = desktop.call(json!({"kind":"workspaces"})).unwrap().data
        else {
            panic!()
        };
        desktop.workspace = list.active.unwrap();
        let root = fs::canonicalize(&f.repo).unwrap();
        let mut kind = "inventory";
        let start = Instant::now();
        loop {
            let Data::Inventory(inventory) = desktop
                .call(json!({
                    "kind": kind, "workspace": desktop.workspace
                }))
                .unwrap()
                .data
            else {
                panic!("Expected workspace inventory")
            };
            if let Some(entry) = inventory
                .entries
                .iter()
                .find(|entry| entry.metadata.root.to_path().unwrap() == root)
            {
                desktop.worktree = entry.id.0.clone();
                break;
            }
            assert!(
                inventory
                    .scan
                    .as_ref()
                    .is_some_and(|s| s.progress.is_some())
                    && start.elapsed() < Duration::from_secs(90),
                "Fixture worktree missing: {:?}",
                inventory.scan
            );
            kind = "continue_scan";
        }
        desktop
    }

    fn review_main_observation(&mut self) -> ReviewDisplay {
        let response = self
            .call(json!({
                "kind": "review_remote_observation",
                "workspace": self.workspace,
                "worktree": self.worktree,
                "remote": "origin",
                "reference": "refs/heads/main"
            }))
            .unwrap();
        shown(response, "observe_branch")
    }

    fn call(&mut self, request: Value) -> gitfleet_core::Result<Response> {
        let payload = json!({"version": 1, "epoch": self.core.epoch(), "request": request});
        let envelope = decode(&payload.to_string()).unwrap();
        process_request(
            &mut self.core,
            &self.host,
            envelope.epoch,
            envelope.request,
            1,
            Cancellation::default(),
            |_| panic!("Unexpected provider transfer"),
        )
    }
}

#[test]
fn review_push_publication_preserves_remote_until_explicit_push() {
    let f = Fixture::new();
    f.commit_file();
    let remote = f.root.join("remote.git");
    let bare = remote.to_str().unwrap();
    f.git(&["clone", "-q", "--bare", ".", bare]);
    f.git(&["-C", bare, "config", "http.receivepack", "true"]);
    let server = tls::start(&f, &remote, "publication");
    f.git(&[
        "remote",
        "add",
        "origin",
        &format!("https://127.0.0.1:{}/repo.git", server.port),
    ]);
    let mut options = Options::default();
    options.remote.ca_file = Some(f.root.join("certificate.pem"));
    let mut desktop = Desktop::open(&f, options);
    let observation = desktop.review_main_observation();
    assert!(!server.marker.exists());
    let observed = execute(&mut desktop, &observation);
    let remote_head = || {
        String::from_utf8(f.git(&["-C", bare, "rev-parse", "refs/heads/main"]))
            .unwrap()
            .trim()
            .to_owned()
    };
    let before = remote_head();
    f.git(&["commit", "--allow-empty", "-qm", "outgoing"]);
    let source = String::from_utf8(f.git(&["rev-parse", "HEAD"])).unwrap();
    let request = json!({
        "kind": "review_push_publication",
        "workspace": desktop.workspace,
        "selection": {
            "worktree": desktop.worktree,
            "observation": observed["plan"]["id"],
            "source_oid": {"format": "sha1", "hex": source.trim()},
            "children": []
        },
        "hooks": "execute"
    });
    let requests = || fs::read_to_string(&server.marker).unwrap();
    let cancelled = shown(desktop.call(request.clone()).unwrap(), "push");
    assert_eq!(remote_head(), before);
    cancel(&mut desktop, &cancelled);
    assert!(desktop.call(decision(&cancelled, true)).is_err());
    assert_eq!(remote_head(), before);
    assert!(!requests().contains("git-receive-pack"));

    let plan = shown(desktop.call(request.clone()).unwrap(), "push");
    assert!(plan.summary.contains(source.trim()));
    assert!(plan.summary.contains("refs/heads/main"));
    assert!(plan.summary.contains(&before));
    let mut wrong = decision(&plan, true);
    wrong["operation"] = json!("bulk_commit");
    assert!(desktop.call(wrong).is_err());
    assert!(!requests().contains("git-receive-pack"));

    let result = execute(&mut desktop, &plan);
    assert_eq!(result["phase"], "complete", "{result}");
    assert_eq!(remote_head(), source.trim());
    assert!(desktop.call(decision(&plan, true)).is_err());
    assert_eq!(
        requests()
            .lines()
            .filter(|s| s.starts_with("/repo.git/git-receive-pack "))
            .count(),
        1
    );

    // Move the local source after preparing the next fast-forward.
    let observation = desktop.review_main_observation();
    let observed = execute(&mut desktop, &observation);
    f.git(&["commit", "--allow-empty", "-qm", "second outgoing"]);
    let mut next = request;
    next["selection"]["observation"] = observed["plan"]["id"].clone();
    next["selection"]["source_oid"]["hex"] =
        json!(String::from_utf8(f.git(&["rev-parse", "HEAD"]))
            .unwrap()
            .trim());
    let stale = shown(desktop.call(next).unwrap(), "push");
    f.git(&["commit", "--allow-empty", "-qm", "external drift"]);
    let result = execute(&mut desktop, &stale);
    assert_ne!(result["phase"], "complete");
    assert_eq!(remote_head(), source.trim());
    assert_eq!(
        requests()
            .lines()
            .filter(|s| s.contains("git-receive-pack"))
            .count(),
        2
    );
}

fn shown(response: Response, operation: &str) -> ReviewDisplay {
    let Data::Review(plan) = response.data else {
        panic!("Review executed before the explicit action")
    };
    assert_eq!(json!(plan.operation), operation);
    plan
}

fn decision(plan: &ReviewDisplay, execute: bool) -> Value {
    json!({"kind": if execute {"execute_reviewed_operation"} else {"cancel_reviewed_operation"}, "id":plan.id, "operation":plan.operation})
}

fn execute(desktop: &mut Desktop, plan: &ReviewDisplay) -> Value {
    let Data::Executed(result) = desktop.call(decision(plan, true)).unwrap().data else {
        panic!()
    };
    result.result
}

fn cancel(desktop: &mut Desktop, plan: &ReviewDisplay) {
    assert!(matches!(
        desktop.call(decision(plan, false)).unwrap().data,
        Data::Done
    ));
}

#[test]
fn review_bulk_commit_freezes_selection_and_preserves_exclusions() {
    let f = Fixture::new();
    f.commit_file();
    // An independent checkout under the workspace must never join the batch implicitly.
    f.git(&["clone", "-q", ".", "other"]);
    fs::write(f.repo.join("other/file.txt"), "other edits").unwrap();
    let other = || support::tree_bytes(&f.repo.join("other"));
    let other_before = other();
    let mut desktop = Desktop::open(&f, Options::default());
    fs::write(f.repo.join("file.txt"), "selected").unwrap();
    fs::write(f.repo.join("excluded.txt"), "excluded").unwrap();
    let head = || f.git(&["rev-parse", "HEAD"]);
    let before = head();
    let request = json!({
        "kind": "review_batch",
        "workspace": desktop.workspace,
        "selections": [{
            "worktree": desktop.worktree, "message": "Reviewed batch", "mode": "review_all",
            "excluded": [{"encoding": "utf8", "value": "excluded.txt"}],
            "hooks": "execute", "signing": "configured"
        }]
    });
    let plan = shown(desktop.call(request.clone()).unwrap(), "bulk_commit");
    assert!(plan.content.contains("file.txt"));
    assert!(!plan.content.contains("excluded.txt"));
    assert_eq!(head(), before);
    cancel(&mut desktop, &plan);
    assert!(desktop.call(decision(&plan, true)).is_err());
    assert_eq!(head(), before);
    let stale = shown(desktop.call(request.clone()).unwrap(), "bulk_commit");
    fs::write(f.repo.join("file.txt"), "edited after review").unwrap();
    assert_eq!(
        execute(&mut desktop, &stale)["record"]["phase"],
        "needs_reauthorization"
    );
    assert_eq!(head(), before);
    let plan = shown(desktop.call(request).unwrap(), "bulk_commit");
    let result = execute(&mut desktop, &plan);
    assert_eq!(result["record"]["phase"], "complete", "{result}");
    assert_eq!(f.git(&["rev-parse", "HEAD^"]), before);
    assert_eq!(f.git(&["show", "HEAD:file.txt"]), b"edited after review");
    assert!(!f
        .git_output(&["cat-file", "-e", "HEAD:excluded.txt"])
        .status
        .success());
    assert!(desktop.call(decision(&plan, true)).is_err());
    assert_eq!(other(), other_before);

    let direct = desktop.call(json!({"kind":"propose","workspace":desktop.workspace,"worktree":desktop.worktree,"request":{
        "kind":"commit","selected":[{"path":{"encoding":"utf8","value":"excluded.txt"},"source":"working_tree"}],
        "message":"Ordinary direct commit","hooks":"execute","signing":"configured"
    }})).unwrap();
    assert!(matches!(direct.data, Data::Executed(_)));
    assert_eq!(f.git(&["show", "HEAD:excluded.txt"]), b"excluded");
}

#[path = "ai_transfer_tests.rs"]
mod ai_transfer_tests;

#[test]
fn cancel_direct_stage_during_preparation() {
    use std::{os::unix::fs::PermissionsExt, process::Command};
    const MARKER: &str = "GITFLEET_STAGE_TEST_MARKER";
    let f = Fixture::new();
    let Some(marker) = std::env::var_os(MARKER) else {
        let wrapper = f.root.join("git");
        fs::write(&wrapper, "#!/bin/sh\ncase \" $* \" in *' hash-object -w --stdin '*) echo $$ > \"$0.marker\"; sleep 5;; esac\nPATH=${PATH#*:} exec git \"$@\"\n").unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        let path = std::env::var_os("PATH").unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "publication_tests::cancel_direct_stage_during_preparation",
                "--nocapture",
            ])
            .env(MARKER, f.root.join("git.marker"))
            .env(
                "PATH",
                std::env::join_paths(
                    std::iter::once(f.root.clone()).chain(std::env::split_paths(&path)),
                )
                .unwrap(),
            )
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        return;
    };
    f.commit_file();
    let mut desktop = Desktop::open(&f, Options::default());
    fs::write(f.repo.join("file.txt"), vec![42; 8 * 1024 * 1024]).unwrap();
    let head = f.git(&["rev-parse", "HEAD"]);
    let index = fs::read(f.repo.join(".git/index")).unwrap();
    let (host, rx) = tests::host();
    let app = tauri::test::mock_builder()
        .manage(host.clone())
        .invoke_handler(tauri::generate_handler![dispatch, cancel_job])
        .build(tauri::generate_context!())
        .unwrap();
    let main = WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
        .build()
        .unwrap();
    let submit = |request| {
        tests::invoke(&main, "dispatch", json!({"payload":json!({"version":1,"epoch":desktop.core.epoch(),"request":request}).to_string()})).unwrap()
    };
    let read = submit(json!({"kind":"workspaces"}));
    let stage = submit(
        json!({"kind":"propose","workspace":desktop.workspace,"worktree":desktop.worktree,"request":{"kind":"stage","paths":[{"encoding":"utf8","value":"file.txt"}],"hooks":"execute"}}),
    );
    assert_ne!(read, stage);
    let Task::Request {
        job,
        epoch,
        request,
        cancel: read_token,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!()
    };
    assert_eq!(read, job);
    assert!(matches!(
        process_request(
            &mut desktop.core,
            &host,
            epoch,
            request,
            1,
            read_token.clone(),
            |_| panic!()
        )
        .unwrap()
        .data,
        Data::Workspaces(_)
    ));
    let Task::Request {
        job,
        epoch,
        request,
        cancel: token,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!()
    };
    assert_eq!(stage, job);
    let marker = PathBuf::from(marker);
    let observer = std::thread::spawn(move || {
        let start = Instant::now();
        while !marker.exists() {
            assert!(start.elapsed() < Duration::from_secs(15));
            std::thread::sleep(Duration::from_millis(10));
        }
        tests::invoke(&main, "cancel_job", json!({"job":stage})).unwrap();
        (Instant::now(), fs::read_to_string(marker).unwrap())
    });
    let error = process_request(
        &mut desktop.core,
        &host,
        epoch,
        request,
        1,
        token,
        |_| panic!(),
    )
    .unwrap_err();
    let (cancelled_at, pid) = observer.join().unwrap();
    let elapsed = cancelled_at.elapsed();
    assert!(
        !Command::new("/bin/kill")
            .args(["-0", pid.trim()])
            .output()
            .unwrap()
            .status
            .success(),
        "Cancelled Git process still exists"
    );
    assert_eq!(error.kind, gitfleet_core::ErrorKind::Cancelled);
    assert!(
        elapsed < Duration::from_secs(3),
        "Stage occupied the worker for {elapsed:?} after Cancel"
    );
    assert!(!read_token.is_cancelled());
    assert_eq!(head, f.git(&["rev-parse", "HEAD"]));
    assert_eq!(index, fs::read(f.repo.join(".git/index")).unwrap());
    assert_eq!(
        fs::read_dir(f.root.join("state/operations"))
            .unwrap()
            .count(),
        0
    );
    assert!(matches!(
        desktop.call(json!({"kind":"workspaces"})).unwrap().data,
        Data::Workspaces(_)
    ));
}
