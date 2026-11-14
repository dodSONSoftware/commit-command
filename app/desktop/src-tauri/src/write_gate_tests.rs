//! Exercise the generated IPC handler and worker with the same eligible Stage in both builds.
use super::*;
use crate::test_support as support;
use gitfleet_core::writes::Options;
use std::fs;

#[test]
fn eligible_stage_obeys_build_gate_and_preserves_head_and_worktree() {
    let f = support::Fixture::new();
    f.commit_file();
    let mut core = Application::open(&f.root.join("state"), Options::default()).unwrap();
    let picker = core.register_directory(&f.repo).unwrap();
    let request = |core: &mut Application, value: Value| {
        let input =
            decode(&json!({"version":1,"epoch":core.epoch(),"request":value}).to_string()).unwrap();
        core.request(input.epoch, input.request).unwrap()
    };
    request(
        &mut core,
        json!({"kind":"create","name":"Gate fixture","picker":picker}),
    );
    let Data::Workspaces(list) = request(&mut core, json!({"kind":"workspaces"})).data else {
        panic!()
    };
    let workspace = list.active.unwrap();
    let Data::Inventory(inventory) =
        request(&mut core, json!({"kind":"inventory","workspace":workspace})).data
    else {
        panic!()
    };
    let worktree = &inventory.entries[0].id.0;
    fs::write(f.repo.join("file.txt"), "gate fixture change\n").unwrap();
    let head = f.git(&["rev-parse", "HEAD"]);
    let index = f.git(&["show", ":file.txt"]);
    let working = fs::read(f.repo.join("file.txt")).unwrap();
    let (host, rx) = tests::host();
    let app = tauri::test::mock_builder()
        .manage(host.clone())
        .invoke_handler(tauri::generate_handler![dispatch, desktop_capabilities])
        .build(tauri::generate_context!())
        .unwrap();
    let main = WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
        .build()
        .unwrap();
    let caps = tests::invoke(&main, "desktop_capabilities", json!({})).unwrap();
    assert_eq!(caps["writes_enabled"], cfg!(feature = "desktop-writes"));
    let payload = json!({"payload":json!({"version":1,"epoch":core.epoch(),"request":{
        "kind":"propose","workspace":workspace,"worktree":worktree,"request":{
            "kind":"stage","paths":[{"encoding":"utf8","value":"file.txt"}],"hooks":"execute"}
    }}).to_string()});
    let settings =
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
            .build()
            .unwrap();
    let stranger =
        WebviewWindowBuilder::new(&app, "stranger", WebviewUrl::App("index.html".into()))
            .build()
            .unwrap();
    assert_eq!(
        tests::invoke(&settings, "desktop_capabilities", json!({})).unwrap()["protocol"],
        1
    );
    assert!(tests::invoke(&settings, "dispatch", payload.clone())
        .unwrap_err()
        .to_string()
        .contains("not available in Settings"));
    assert!(tests::invoke(&stranger, "dispatch", payload.clone())
        .unwrap_err()
        .to_string()
        .contains("not allowed"));
    assert!(rx.try_recv().is_err());
    assert_eq!(f.git(&["show", ":file.txt"]), index);
    let result = tests::invoke(&main, "dispatch", payload);
    if cfg!(feature = "desktop-writes") {
        result.unwrap();
        let Task::Request {
            epoch,
            request,
            cancel,
            ..
        } = rx.recv_timeout(Duration::from_secs(2)).unwrap()
        else {
            panic!()
        };
        process_request(&mut core, &host, epoch, request, 1, cancel, |_| {
            panic!("Unexpected source transfer")
        })
        .unwrap();
        assert_eq!(f.git(&["show", ":file.txt"]), working);
    } else {
        assert!(result.unwrap_err().to_string().contains("read-only build"));
        assert!(rx.try_recv().is_err());
        assert_eq!(f.git(&["show", ":file.txt"]), index);
    }
    assert_eq!(f.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(fs::read(f.repo.join("file.txt")).unwrap(), working);
}
