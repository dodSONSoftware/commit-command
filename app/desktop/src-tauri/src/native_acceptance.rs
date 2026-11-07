//! Opt-in fixed native test actions. No new IPC, arbitrary script/path, or approval bypass.
use super::*;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    },
};
static AUDIT: OnceLock<Mutex<File>> = OnceLock::new();
static DROP_CLOSE_ACK: AtomicBool = AtomicBool::new(false);

fn fixture_root(directory: &Path) -> Result<PathBuf, String> {
    let directory = directory
        .canonicalize()
        .map_err(|_| "Acceptance state must already exist")?;
    let root = directory.parent().ok_or("Missing fixture root")?;
    if directory.file_name().is_none_or(|n| n != "state")
        || !(root.starts_with("/private/tmp")
            || std::env::temp_dir()
                .canonicalize()
                .is_ok_and(|temp| root.starts_with(temp)))
        || !root
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("gitfleet-m3-native-"))
        || fs::read_to_string(root.join(".gitfleet-m3-fixture"))
            .ok()
            .as_deref()
            != Some("Disposable GitFleet M3 native acceptance\n")
    {
        return Err(
            "Native acceptance support requires a newly created disposable native fixture".into(),
        );
    }
    Ok(root.to_path_buf())
}
pub(super) fn initialize(directory: &Path) -> Result<(), String> {
    fixture_root(directory)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("native-acceptance-events.jsonl"))
        .map_err(|e| e.to_string())?;
    AUDIT
        .set(Mutex::new(file))
        .map_err(|_| "Native acceptance already initialized")?;
    record("startup", json!({"support":true}));
    Ok(())
}
pub(super) fn validate_workspaces(core: &mut Application, directory: &Path) -> Result<(), String> {
    let root = fixture_root(directory)?
        .join("workspace")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let Data::Workspaces(list) = core
        .request(core.epoch(), Request::Workspaces {})
        .map_err(|e| e.to_string())?
        .data
    else {
        return Err("Unexpected workspace result".into());
    };
    for workspace in list.workspaces {
        let path = workspace
            .root
            .to_path()
            .map_err(|e| e.to_string())?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(&root) {
            return Err("Acceptance workspace escapes disposable fixture".into());
        }
    }
    Ok(())
}
pub(super) fn record(event: &str, detail: Value) {
    if let Some(file) = AUDIT.get() {
        match file.lock() {
            Ok(mut file) => {
                if let Err(error) = writeln!(file, "{}", json!({"event":event,"detail":detail})) {
                    eprintln!("Acceptance audit failed: {error}");
                }
            }
            Err(_) => eprintln!("Acceptance audit lock failed"),
        }
    }
}
pub(super) fn record_probe_title(label: &str, title: &str) {
    if title.starts_with("Native acceptance:") {
        record("ipc_probe", json!({"label":label,"result":title}));
    }
}
pub(super) fn take_close_ack_fault() -> bool {
    let fault = DROP_CLOSE_ACK.swap(false, Ordering::AcqRel);
    if fault {
        record("close_ack_suppressed", json!({}));
    }
    fault
}
fn probe(command: &str, args: &str, name: &str) -> String {
    // Every caller supplies fixed host constants. Nothing comes from a renderer/repository.
    format!(
        r#"(async () => {{
      let result;
      try {{ await window.__TAURI_INTERNALS__.invoke({command:?}, {args}); result='FAIL unexpected success'; }}
      catch (error) {{ result = /not allowed|Unauthorized webview, page or origin/.test(String(error)) ? 'PASS denied' : 'FAIL unexpected error: '+String(error); }}
      const node=document.createElement('pre'); node.textContent='NATIVE ACCEPTANCE {name}: '+result;
      document.body.append(node); document.title='Native acceptance:{name}:'+result;
    }})()"#
    )
}
// Exercise AppKit's termination entry point, separately from the blocked Dock UI.
#[cfg(target_os = "macos")]
fn request_appkit_termination(app: &tauri::AppHandle) -> Result<(), String> {
    app.run_on_main_thread(|| {
        let Some(main_thread) = objc2::MainThreadMarker::new() else {
            record("appkit_terminate_unavailable", json!({}));
            return;
        };
        record("appkit_terminate_requested", json!({}));
        let application = objc2_app_kit::NSApplication::sharedApplication(main_thread);
        // AppKit must run this after Tauri releases its event-callback lock.
        // SAFETY: NSObject's run-loop selector API schedules the owned NSApplication's
        // fixed terminate: method with a nil sender. No external selector/argument exists.
        unsafe {
            let _: () = objc2::msg_send![&*application,
                performSelector: objc2::sel!(terminate:),
                withObject: std::ptr::null::<objc2::runtime::AnyObject>(),
                afterDelay: 0_f64
            ];
        }
    })
    .map_err(|error| error.to_string())
}
#[cfg(not(target_os = "macos"))]
fn request_appkit_termination(_: &tauri::AppHandle) -> Result<(), String> {
    Err("AppKit termination acceptance is macOS-only".into())
}
// WebKit's private PID property is used only by this opt-in fixture facility.
// It is checked at runtime; no PID, selector, or signal is supplied by a renderer.
#[cfg(target_os = "macos")]
fn terminate_main_renderer(window: &tauri::WebviewWindow) -> Result<(), String> {
    window
        .with_webview(|webview| {
            // SAFETY: with_webview runs on the main thread with a live WKWebView.
            // WebKit declares _webProcessIdentifier as pid_t (i32) on this platform.
            let pid = unsafe {
                let view = &*webview.inner().cast::<objc2::runtime::AnyObject>();
                let supported: bool =
                    objc2::msg_send![view, respondsToSelector: objc2::sel!(_webProcessIdentifier)];
                if !supported {
                    record("renderer_kill_unavailable", json!({}));
                    return;
                }
                objc2::msg_send![view, _webProcessIdentifier]
            };
            if !renderer_pid_is_safe(pid, std::process::id()) {
                record("renderer_kill_refused", json!({"pid":pid}));
                return;
            }
            record(
                "renderer_kill_requested",
                json!({"pid":pid,"host_pid":std::process::id()}),
            );
            match std::process::Command::new("/bin/kill")
                .args(["-KILL", &pid.to_string()])
                .status()
            {
                Ok(status) => record(
                    "renderer_kill_result",
                    json!({"pid":pid,"success":status.success()}),
                ),
                Err(error) => record("renderer_kill_error", json!({"error":error.to_string()})),
            }
        })
        .map_err(|error| error.to_string())
}
#[cfg(not(target_os = "macos"))]
fn terminate_main_renderer(_: &tauri::WebviewWindow) -> Result<(), String> {
    Err("Renderer process acceptance is macOS-only".into())
}
fn renderer_pid_is_safe(pid: i32, host: u32) -> bool {
    pid > 1 && u32::try_from(pid).is_ok_and(|pid| pid != host)
}
pub(super) fn install(app: &tauri::AppHandle, host: Arc<Host>) -> tauri::Result<()> {
    use tauri::menu::{MenuItem, Submenu};
    let submenu = Submenu::new(app, "Native acceptance", true)?;
    for (id, label) in [
        ("accept-reload", "Reload main renderer"),
        (
            "accept-appkit-terminate",
            "Request direct AppKit termination",
        ),
        (
            "accept-kill-renderer",
            "Terminate actual main renderer process",
        ),
        ("accept-main-probe", "Probe main to review confirmation"),
        ("accept-review-probe", "Probe review to main dispatch"),
        ("accept-unprivileged", "Open unprivileged caller probe"),
        ("accept-wrong-page", "Attempt wrong main page"),
        ("accept-wrong-origin", "Attempt wrong main origin"),
        ("accept-drop-ack", "Drop next close acknowledgement"),
        (
            "accept-before-publication",
            "SIGKILL next commit before ref publication",
        ),
        (
            "accept-after-publication",
            "SIGKILL next commit after ref update",
        ),
    ] {
        submenu.append(&MenuItem::with_id(app, id, label, true, None::<&str>)?)?;
    }
    app.menu()
        .ok_or(tauri::Error::WindowNotFound)?
        .append(&submenu)?;
    app.on_menu_event(move |app, event| {
        let id=event.id().as_ref();
        let main=app.get_webview_window("main");
        let result: Result<(), String> = (|| {
            match id {
                "accept-reload" => { record("reload_requested",json!({})); main.ok_or("Main missing")?.reload().map_err(|e| e.to_string())?; }
                "accept-appkit-terminate" => request_appkit_termination(app)?,
                "accept-kill-renderer" => terminate_main_renderer(&main.ok_or("Main missing")?)?,
                "accept-main-probe" => main.ok_or("Main missing")?.eval(probe("review_confirm", r#"{review:'native-acceptance-no-authority'}"#, "main-review")).map_err(|e| e.to_string())?,
                "accept-review-probe" => app.get_webview_window("review").ok_or("Review missing")?.eval(probe("dispatch", r#"{payload:JSON.stringify({version:1,epoch:0,request:{kind:'workspaces'}})}"#, "review-main")).map_err(|e| e.to_string())?,
                "accept-unprivileged" => {
                    if let Some(old)=app.get_webview_window("native-probe") { old.close().map_err(|e|e.to_string())?; return Ok(()); }
                    WebviewWindowBuilder::new(app,"native-probe",WebviewUrl::App("review.html".into()))
                        .title("Native acceptance — unprivileged caller")
                        .on_page_load(|window,payload| { if payload.event()==tauri::webview::PageLoadEvent::Finished { let _=window.eval(probe("dispatch", r#"{payload:JSON.stringify({version:1,epoch:0,request:{kind:'workspaces'}})}"#, "unprivileged-main")); } })
                        .on_document_title_changed(|window,title| record_probe_title(window.label(),&title))
                        .build().map_err(|e|e.to_string())?;
                }
                "accept-wrong-page" => main.ok_or("Main missing")?.navigate("tauri://localhost/review.html".parse().map_err(|_|"Fixed URL invalid")?).map_err(|e|e.to_string())?,
                "accept-wrong-origin" => main.ok_or("Main missing")?.navigate("tauri://native-acceptance.invalid/index.html".parse().map_err(|_|"Fixed URL invalid")?).map_err(|e|e.to_string())?,
                "accept-drop-ack" => { DROP_CLOSE_ACK.store(true,Ordering::Release); record("close_ack_fault_armed",json!({})); }
                "accept-before-publication" => queue(&host,Task::AcceptanceFault(gitfleet_core::writes::Phase::PublicationPending))?,
                "accept-after-publication" => queue(&host,Task::AcceptanceFault(gitfleet_core::writes::Phase::ReferenceUpdated))?,
                _ => return Ok(()),
            }
            record("menu_action",json!({"id":id})); Ok(())
        })();
        if let Err(error)=result { record("action_error",json!({"id":id,"error":error})); }
    });
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    #[test]
    fn fixture_guard_requires_exact_marker_and_canonical_state() {
        let root = Path::new("/private/tmp").join(format!("gitfleet-m3-native-{}", Uuid::new_v4()));
        let state = root.join("state");
        fs::create_dir_all(&state).unwrap();
        assert!(fixture_root(&state).is_err());
        fs::write(
            root.join(".gitfleet-m3-fixture"),
            "Disposable GitFleet M3 native acceptance\n",
        )
        .unwrap();
        assert_eq!(fixture_root(&state).unwrap(), root);
        let outside = root.join("outside");
        fs::create_dir(&outside).unwrap();
        let alias = root.join("alias");
        std::os::unix::fs::symlink(&outside, &alias).unwrap();
        assert!(fixture_root(&alias).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn renderer_pid_guard_refuses_groups_system_and_host() {
        for pid in [i32::MIN, -1, 0, 1, 42] {
            assert!(!renderer_pid_is_safe(pid, 42));
        }
        assert!(renderer_pid_is_safe(43, 42));
    }
    #[test]
    fn refuses_ordinary_or_unmarked_state() {
        assert!(fixture_root(Path::new(
            "/non-fixture/Application Support/dev.gitfleet.desktop"
        ))
        .is_err());
        assert!(fixture_root(Path::new("/private/tmp")).is_err());
    }
}
