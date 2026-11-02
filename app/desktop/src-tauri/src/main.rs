#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod security;
mod shutdown;
use shutdown::close_snapshot;
mod window_state;
use gitfleet_core::{
    application::{Application, Confirmation, Data, Request, ReviewDisplay, WindowPlacement},
    inspection::Cancellation,
    writes::Options,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    epoch: u64,
    request: Request,
}
#[derive(Clone, Serialize)]
struct JobView {
    state: String,
    response: Option<Value>,
    error: Option<Value>,
}
struct Job {
    owner: String,
    generation: u64,
    created: Instant,
    cancel: Cancellation,
    view: JobView,
}
struct Host {
    sender: SyncSender<Task>,
    jobs: Mutex<BTreeMap<String, Job>>,
    review: Mutex<Option<String>>,
    closing: std::sync::atomic::AtomicBool,
    active: Mutex<Option<Cancellation>>,
    dialog: std::sync::atomic::AtomicBool,
    generation: std::sync::atomic::AtomicU64,
    reset: std::sync::atomic::AtomicBool,
    completed: std::sync::atomic::AtomicU64,
    close_state: shutdown::CloseState,
    shutdown_complete: std::sync::atomic::AtomicBool,
}
enum Task {
    Request {
        job: String,
        epoch: u64,
        request: Request,
        cancel: Cancellation,
    },
    GetReview {
        id: String,
        reply: mpsc::Sender<Result<ReviewDisplay, String>>,
    },
    Begin {
        id: String,
        reply: mpsc::Sender<Result<(Confirmation, ReviewDisplay), String>>,
    },
    Confirm {
        generation: u64,
        capability: Confirmation,
        accepted: bool,
        cancel: Cancellation,
        reply: mpsc::Sender<Result<Value, String>>,
    },
    RequestDirectory(NativeDirectory),
    CancelReview(String),
    SaveWindowPlacement(WindowPlacement),
    SavePresentation {
        snapshot: shutdown::Snapshot,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Shutdown,
}
impl Host {
    fn invalidate_presentation(&self) {
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        self.reset.store(true, std::sync::atomic::Ordering::Release);
        if let Ok(jobs) = self.jobs.lock() {
            for j in jobs.values() {
                j.cancel.cancel();
            }
        }
        if let Ok(active) = self.active.lock() {
            if let Some(cancel) = active.as_ref() {
                cancel.cancel();
            }
        }
        if let Ok(mut review) = self.review.lock() {
            *review = None;
        }
    }
}
struct DialogLease(Arc<Host>);
impl DialogLease {
    fn new(host: Arc<Host>) -> Result<Self, String> {
        if host
            .dialog
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_err()
        {
            return Err("A native dialog is already open".into());
        }
        Ok(Self(host))
    }
}
impl Drop for DialogLease {
    fn drop(&mut self) {
        self.0
            .dialog
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
fn core_error(e: gitfleet_core::Error) -> Value {
    json!({"kind":e.kind,"message":e.to_string()})
}
fn check<R: tauri::Runtime>(window: &WebviewWindow<R>, role: &str) -> Result<(), String> {
    security::authorize(
        window.label(),
        &window.url().map_err(|_| "Cannot verify caller")?,
        role,
    )
}
fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex.lock().map_err(|_| "Host state unavailable".into())
}
fn queue(host: &Host, task: Task) -> Result<(), String> {
    if host.closing.load(std::sync::atomic::Ordering::Acquire) {
        return Err("Application is closing".into());
    }
    host.sender
        .try_send(task)
        .map_err(|_| "Application busy; request queue is bounded".into())
}
#[tauri::command]
fn desktop_capabilities<R: tauri::Runtime>(window: WebviewWindow<R>) -> Result<Value, String> {
    check(&window, "main")?;
    Ok(json!({"writes_enabled":cfg!(feature="desktop-writes"),"protocol":1}))
}
fn decode(payload: &str) -> Result<Envelope, String> {
    if payload.len() > security::MAX_REQUEST {
        return Err("Request exceeds 3 MiB".into());
    }
    let input: Envelope =
        serde_json::from_str(payload).map_err(|_| "Malformed or unsupported request")?;
    if input.version != 1 {
        return Err("Unsupported protocol version".into());
    }
    // Core compatibility DTOs may ignore unknown fields; the desktop wire does not.
    let actual: Value = serde_json::from_str(payload).map_err(|_| "Malformed request")?;
    let expected = json!({"version":input.version,"epoch":input.epoch,"request":input.request});
    if !security::known_fields(&actual, &expected) {
        return Err("Unknown request field".into());
    }
    Ok(input)
}
fn mutating_request(request: &Request) -> bool {
    matches!(
        request,
        Request::ReviewTrust { .. }
            | Request::ReviewBatch { .. }
            | Request::ReviewRemoteObservation { .. }
            | Request::ReviewFetchCapture { .. }
            | Request::ReviewFetchImport { .. }
            | Request::ReviewFetchImportRecovery { .. }
            | Request::Revoke { .. }
            | Request::HookPolicy { .. }
            | Request::Propose { .. }
    )
}
#[tauri::command]
fn dispatch<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    payload: String,
) -> Result<String, String> {
    check(&window, "main")?;
    let input = decode(&payload)?;
    if !cfg!(feature = "desktop-writes") && mutating_request(&input.request) {
        return Err("Desktop writes are disabled pending native acceptance".into());
    }
    if lock(&state.active)?.is_some()
        && matches!(
            input.request,
            Request::Activate { .. }
                | Request::Create { .. }
                | Request::Update { .. }
                | Request::Remove { .. }
                | Request::Propose { .. }
                | Request::ReviewTrust { .. }
                | Request::ReviewBatch { .. }
                | Request::ReviewRemoteObservation { .. }
                | Request::ReviewRecovery { .. }
        )
    {
        return Err("Controlled write is active".into());
    }
    if matches!(input.request, Request::Revoke { .. }) {
        if let Some(cancel) = lock(&state.active)?.as_ref() {
            cancel.cancel();
        }
    }
    let id = Uuid::new_v4().to_string();
    let cancellation = Cancellation::default();
    {
        let mut jobs = lock(&state.jobs)?;
        jobs.retain(|_, j| {
            j.created.elapsed() < Duration::from_secs(600) || j.view.state == "running"
        });
        if jobs.len() >= 32 {
            return Err("Too many unconsumed jobs".into());
        }
        jobs.insert(
            id.clone(),
            Job {
                owner: window.label().into(),
                generation: state.generation.load(std::sync::atomic::Ordering::Acquire),
                created: Instant::now(),
                cancel: cancellation.clone(),
                view: JobView {
                    state: "queued".into(),
                    response: None,
                    error: None,
                },
            },
        );
    }
    if let Err(e) = queue(
        &state,
        Task::Request {
            job: id.clone(),
            epoch: input.epoch,
            request: input.request,
            cancel: cancellation,
        },
    ) {
        lock(&state.jobs)?.remove(&id);
        return Err(e);
    }
    Ok(id)
}
#[tauri::command]
fn job_status<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    job: String,
) -> Result<JobView, String> {
    check(&window, "main")?;
    if job == "native-write" {
        return Ok(JobView {
            state: if lock(&state.active)?.is_some() {
                "running"
            } else {
                "idle"
            }
            .into(),
            response: Some(
                json!({"revision": state.completed.load(std::sync::atomic::Ordering::Acquire)}),
            ),
            error: None,
        });
    }
    let mut jobs = lock(&state.jobs)?;
    let entry = jobs.get(&job).ok_or("Unknown or consumed job")?;
    if entry.owner != window.label()
        || entry.generation != state.generation.load(std::sync::atomic::Ordering::Acquire)
    {
        return Err("Job belongs to another view".into());
    }
    let view = entry.view.clone();
    if view.state == "complete" {
        jobs.remove(&job);
    }
    Ok(view)
}
#[tauri::command]
fn cancel_job<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    job: String,
) -> Result<(), String> {
    check(&window, "main")?;
    if job == "native-write" {
        if let Some(cancel) = lock(&state.active)?.as_ref() {
            cancel.cancel();
        }
        return Ok(());
    }
    let jobs = lock(&state.jobs)?;
    let entry = jobs.get(&job).ok_or("Unknown job")?;
    if entry.owner != window.label()
        || entry.generation != state.generation.load(std::sync::atomic::Ordering::Acquire)
    {
        return Err("Job belongs to another view".into());
    }
    entry.cancel.cancel();
    Ok(())
}
#[tauri::command]
async fn pick_directory<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
) -> Result<Value, String> {
    check(&window, "main")?;
    let _lease = DialogLease::new(state.inner().clone())?;
    let path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title("Choose GitFleet workspace root")
            .set_parent(&window)
            .pick_folder()
    })
    .await
    .map_err(|_| "Native picker failed")?;
    let Some(path) = path else {
        return Ok(Value::Null);
    };
    let (reply, rx) = mpsc::channel();
    // Directory paths are sent only over a Rust-only message, never accepted from IPC JSON.
    let task = NativeDirectory { path, reply };
    queue(&state, Task::RequestDirectory(task))?;
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(60))
            .map_err(|_| "Directory selection timed out".to_owned())?
    })
    .await
    .map_err(|_| "Native picker response failed".to_owned())?
}
struct NativeDirectory {
    path: PathBuf,
    reply: mpsc::Sender<Result<Value, String>>,
}
fn current_review<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    state: &Host,
    id: Option<&str>,
) -> Result<String, String> {
    check(window, "review")?;
    let active = lock(&state.review)?.clone().ok_or("No active review")?;
    if id.is_some_and(|id| id != active) {
        return Err("Review changed".into());
    }
    Ok(active)
}
#[derive(Serialize)]
struct ReviewDocument {
    #[serde(flatten)]
    review: ReviewDisplay,
    can_confirm: bool,
}
#[tauri::command]
async fn review_data<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
) -> Result<ReviewDocument, String> {
    let id = current_review(&window, &state, None)?;
    let (reply, rx) = mpsc::channel();
    queue(&state, Task::GetReview { id, reply })?;
    let review = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|_| "Review unavailable".to_owned())?
    })
    .await
    .map_err(|_| "Review unavailable".to_owned())??;
    Ok(ReviewDocument {
        review,
        can_confirm: cfg!(feature = "desktop-writes"),
    })
}
#[tauri::command]
async fn review_confirm<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    review: String,
) -> Result<Value, String> {
    if !cfg!(feature = "desktop-writes") {
        return Err("Desktop writes are disabled pending native acceptance".into());
    }
    let generation = state.generation.load(std::sync::atomic::Ordering::Acquire);
    let _lease = DialogLease::new(state.inner().clone())?;
    let id = current_review(&window, &state, Some(&review))?;
    let (reply, rx) = mpsc::channel();
    queue(
        &state,
        Task::Begin {
            id: id.clone(),
            reply,
        },
    )?;
    let (capability, display) = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|_| "Review unavailable".to_owned())?
    })
    .await
    .map_err(|_| "Review unavailable".to_owned())??;
    if generation != state.generation.load(std::sync::atomic::Ordering::Acquire) {
        return Err("Presentation session changed".into());
    }
    let accepted = tauri::async_runtime::spawn_blocking(move || {
        rfd::MessageDialog::new()
            .set_parent(&window)
            .set_title("GitFleet — Native approval")
            .set_description(&display.summary)
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::OkCancelCustom(
                "Approve exact operation".into(),
                "Cancel".into(),
            ))
            .show()
            == rfd::MessageDialogResult::Custom("Approve exact operation".into())
    })
    .await
    .map_err(|_| "Native confirmation failed")?;
    if generation != state.generation.load(std::sync::atomic::Ordering::Acquire) {
        return Err("Presentation session changed".into());
    }
    let (reply, rx) = mpsc::channel();
    let cancellation = Cancellation::default();
    {
        let mut active = lock(&state.active)?;
        if active.is_some() {
            return Err("Controlled write is active".into());
        }
        *active = Some(cancellation.clone());
    }
    if let Err(e) = queue(
        &state,
        Task::Confirm {
            generation,
            capability,
            accepted,
            cancel: cancellation,
            reply,
        },
    ) {
        *lock(&state.active)? = None;
        return Err(e);
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        rx.recv()
            .map_err(|_| "Execution response unavailable; inspect operation history".to_owned())?
    })
    .await
    .map_err(|_| "Execution response unavailable".to_owned())?;
    *lock(&state.active)? = None;
    state
        .completed
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    result
}
#[tauri::command]
fn review_cancel<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    review: String,
) -> Result<(), String> {
    let id = current_review(&window, &state, Some(&review))?;
    if let Some(cancel) = lock(&state.active)?.as_ref() {
        cancel.cancel();
    }
    queue(&state, Task::CancelReview(id))?;
    window.close().map_err(|_| "Cannot close review".into())
}
fn open_review(app: &tauri::AppHandle, host: Arc<Host>, display: &ReviewDisplay, generation: u64) {
    let id = display.id.clone();
    let app = app.clone();
    let runner = app.clone();
    let _ = runner.run_on_main_thread(move || {
        if host.generation.load(std::sync::atomic::Ordering::Acquire) != generation {
            return;
        }
        if let Some(old) = app.get_webview_window("review") {
            let _ = old.close();
        }
        if let Ok(mut current) = host.review.lock() {
            *current = Some(id.clone());
        }
        let built =
            WebviewWindowBuilder::new(&app, "review", WebviewUrl::App("review.html".into()))
                .title("GitFleet — Authoritative review")
                .inner_size(1000., 800.)
                .min_inner_size(700., 500.)
                .devtools(false)
                .on_navigation(|url| security::authorize("review", url, "review").is_ok())
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .build();
        if let Ok(window) = built {
            let sender = host.sender.clone();
            let host = host.clone();
            window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Destroyed) {
                    let _ = sender.try_send(Task::CancelReview(id.clone()));
                    if let Ok(mut current) = host.review.lock() {
                        if current.as_ref() == Some(&id) {
                            if let Ok(active) = host.active.lock() {
                                if let Some(cancel) = active.as_ref() {
                                    cancel.cancel();
                                }
                            }
                            *current = None;
                        }
                    }
                }
            });
        }
    });
}
fn worker(mut core: Application, rx: mpsc::Receiver<Task>, app: tauri::AppHandle, host: Arc<Host>) {
    loop {
        let event = rx.recv_timeout(Duration::from_millis(250));
        if host.reset.swap(false, std::sync::atomic::Ordering::AcqRel) {
            if let Err(error) = core.reset_presentation() {
                core.record_monitor_failure(&error);
            }
        }
        match event {
            Ok(Task::Request {
                job,
                epoch,
                request,
                cancel,
            }) => {
                if let Ok(mut jobs) = host.jobs.lock() {
                    if let Some(j) = jobs.get_mut(&job) {
                        j.view.state = "running".into();
                    }
                }
                let generation = host.generation.load(std::sync::atomic::Ordering::Acquire);
                let previous_epoch = core.epoch();
                let result = core.request_with_cancellation(epoch, request, cancel);
                if core.epoch() != previous_epoch {
                    if let Ok(mut review) = host.review.lock() {
                        *review = None;
                    }
                    if let Some(review) = app.get_webview_window("review") {
                        if let Err(error) = review.close() {
                            eprintln!("Obsolete review window could not be closed: {error}");
                        }
                    }
                }
                if let Ok(response) = &result {
                    if let Data::Review(display) = &response.data {
                        open_review(&app, host.clone(), display, generation);
                    }
                }
                let response = result
                    .and_then(|r| serde_json::to_value(r).map_err(gitfleet_core::Error::from));
                if let Ok(mut jobs) = host.jobs.lock() {
                    if let Some(j) = jobs.get_mut(&job) {
                        j.view = match response {
                            Ok(value)
                                if serde_json::to_vec(&value)
                                    .is_ok_and(|v| v.len() <= 16 * 1024 * 1024) =>
                            {
                                JobView {
                                    state: "complete".into(),
                                    response: Some(value),
                                    error: None,
                                }
                            }
                            Ok(_) => JobView {
                                state: "complete".into(),
                                response: None,
                                error: Some(
                                    json!({"kind":"output_limit","message":"Desktop response limit exceeded"}),
                                ),
                            },
                            Err(e) => JobView {
                                state: "complete".into(),
                                response: None,
                                error: Some(core_error(e)),
                            },
                        };
                    }
                }
            }
            Ok(Task::RequestDirectory(task)) => {
                let value = core
                    .register_directory(&task.path)
                    .map(|picker| json!({"picker":picker,"display":task.path.to_string_lossy()}))
                    .map_err(|e| e.to_string());
                let _ = task.reply.send(value);
            }
            Ok(Task::GetReview { id, reply }) => {
                let _ = reply.send(core.review(&id).map_err(|e| e.to_string()));
            }
            Ok(Task::Begin { id, reply }) => {
                let _ = reply.send(core.begin_confirmation(&id).map_err(|e| e.to_string()));
            }
            Ok(Task::Confirm {
                generation,
                capability,
                accepted,
                cancel,
                reply,
            }) => {
                let result =
                    if generation == host.generation.load(std::sync::atomic::Ordering::Acquire) {
                        core.confirm(capability, accepted, cancel)
                            .map_err(|e| e.to_string())
                    } else {
                        Err("Presentation session changed".into())
                    };
                let _ = reply.send(result);
            }
            Ok(Task::CancelReview(id)) => {
                if core.cancel_review(&id).is_ok() {
                    host.completed
                        .fetch_add(1, std::sync::atomic::Ordering::Release);
                }
            }
            Ok(Task::SavePresentation { snapshot, reply }) => {
                let _ = reply.send(snapshot.save(&mut core));
            }
            Ok(Task::SaveWindowPlacement(placement)) => {
                if let Err(error) = core.save_window_placement(placement) {
                    eprintln!("Window placement could not be saved: {:?}", error.kind);
                }
            }
            Ok(Task::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                if let Err(error) = core.shutdown() {
                    eprintln!(
                        "Application shutdown state could not be saved: {:?}",
                        error.kind
                    );
                }
                host.shutdown_complete
                    .store(true, std::sync::atomic::Ordering::Release);
                app.exit(0);
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !host.closing.load(std::sync::atomic::Ordering::Acquire) {
                    if let Err(error) = core.reconcile() {
                        core.record_monitor_failure(&error);
                    }
                }
            }
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let directory = match args.next() {
        Some(flag) if flag == "--data-dir" => {
            PathBuf::from(args.next().ok_or("--data-dir needs a path")?)
        }
        Some(_) => return Err("Only --data-dir is supported".into()),
        None => PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
            .join("Library/Application Support/dev.gitfleet.desktop"),
    };
    if args.next().is_some() {
        return Err("Unexpected arguments".into());
    }
    let mut core = Application::open(&directory, Options::default())?;
    let saved_placement = core.window_placement()?;
    let (sender, rx) = mpsc::sync_channel(8);
    let host = Arc::new(Host {
        sender,
        jobs: Mutex::new(BTreeMap::new()),
        review: Mutex::new(None),
        closing: std::sync::atomic::AtomicBool::new(false),
        active: Mutex::new(None),
        dialog: std::sync::atomic::AtomicBool::new(false),
        generation: std::sync::atomic::AtomicU64::new(1),
        reset: std::sync::atomic::AtomicBool::new(false),
        completed: std::sync::atomic::AtomicU64::new(0),
        close_state: shutdown::CloseState::default(),
        shutdown_complete: std::sync::atomic::AtomicBool::new(false),
    });
    let setup_host = host.clone();
    tauri::Builder::default()
        .manage(host.clone())
        .invoke_handler(tauri::generate_handler![
            desktop_capabilities,
            dispatch,
            pick_directory,
            job_status,
            cancel_job,
            review_data,
            review_confirm,
            review_cancel,
            close_snapshot
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
                // AppKit's predefined Quit calls terminate: directly, bypassing
                // ExitRequested and the final draft snapshot handshake.
                let menu = Menu::default(app.handle())?;
                menu.remove_at(0)?;
                let application = Submenu::with_items(
                    app,
                    "GitFleet",
                    true,
                    &[
                        &PredefinedMenuItem::about(app, Some("About GitFleet"), None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::services(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, None)?,
                        &PredefinedMenuItem::hide_others(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "gitfleet-quit",
                            "Quit GitFleet",
                            true,
                            Some("Cmd+Q"),
                        )?,
                    ],
                )?;
                menu.prepend(&application)?;
                app.set_menu(menu)?;
                let quit_host = setup_host.clone();
                app.on_menu_event(move |app, event| {
                    if event.id().as_ref() == "gitfleet-quit" {
                        shutdown::request(app.clone(), quit_host.clone());
                    }
                });
            }
            let page_host = setup_host.clone();
            let page_loaded = std::sync::atomic::AtomicBool::new(false);
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("GitFleet")
                    .visible(false)
                    .inner_size(1440., 940.)
                    .min_inner_size(window_state::MIN_WIDTH, window_state::MIN_HEIGHT)
                    .devtools(false)
                    .on_page_load(move |window, payload| {
                        if payload.event() == tauri::webview::PageLoadEvent::Started
                            && page_loaded.swap(true, std::sync::atomic::Ordering::AcqRel)
                        {
                            page_host.invalidate_presentation();
                            if let Some(review) = window.app_handle().get_webview_window("review") {
                                let _ = review.close();
                            }
                        }
                    })
                    .on_navigation(|url| security::authorize("main", url, "main").is_ok())
                    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                    .build()?;
            if let Some(placement) = saved_placement {
                window_state::restore(&window, placement)?;
            }
            window.show()?;
            let handle = app.handle().clone();
            let host = setup_host.clone();
            std::thread::Builder::new()
                .name("gitfleet-service".into())
                .spawn(move || worker(core, rx, handle, host))?;
            let handle = app.handle().clone();
            let host = setup_host.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    shutdown::request(handle.clone(), host.clone());
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(move |app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if !host
                    .shutdown_complete
                    .load(std::sync::atomic::Ordering::Acquire)
                {
                    api.prevent_exit();
                    shutdown::request(app.clone(), host.clone());
                }
            }
        });
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("GitFleet startup failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn host() -> (Arc<Host>, mpsc::Receiver<Task>) {
        let (sender, rx) = mpsc::sync_channel(8);
        (
            Arc::new(Host {
                sender,
                jobs: Mutex::new(BTreeMap::new()),
                review: Mutex::new(None),
                closing: std::sync::atomic::AtomicBool::new(false),
                active: Mutex::new(None),
                dialog: std::sync::atomic::AtomicBool::new(false),
                generation: std::sync::atomic::AtomicU64::new(1),
                reset: std::sync::atomic::AtomicBool::new(false),
                completed: std::sync::atomic::AtomicU64::new(0),
                close_state: shutdown::CloseState::default(),
                shutdown_complete: std::sync::atomic::AtomicBool::new(false),
            }),
            rx,
        )
    }
    #[test]
    fn strict_envelope_rejects_oversize_version_unknown_fields_and_approval_json() {
        for raw in [
            json!({"version":2,"epoch":1,"request":{"kind":"workspaces"}}),
            json!({"version":1,"epoch":1,"approved":true,"request":{"kind":"workspaces"}}),
            json!({"version":1,"epoch":1,"request":{"kind":"workspaces","shell":"evil"}}),
            json!({"version":1,"epoch":1,"request":{"kind":"execute","approved":true,"token":"forged"}}),
        ] {
            assert!(decode(&raw.to_string()).is_err());
        }
        let nested = json!({"version":1,"epoch":1,"request":{"kind":"propose","workspace":"w","worktree":"t","request":{"kind":"stage","paths":[{"encoding":"utf8","value":"file.txt"}],"hooks":"execute","approved":true}}});
        assert!(decode(&nested.to_string()).is_err());
        let batch = json!({"version":1,"epoch":1,"request":{"kind":"review_batch","workspace":"w","selections":[{"worktree":"t","message":"message","mode":"review_all","excluded":[],"hooks":"execute","signing":"configured","approved":true}]}});
        assert!(decode(&batch.to_string()).is_err());
        assert!(decode(&"x".repeat(security::MAX_REQUEST + 1)).is_err());
        assert!(decode(
            &json!({"version":1,"epoch":1,"request":{"kind":"workspaces"}}).to_string()
        )
        .is_ok());
    }
    #[test]
    fn bulk_review_uses_the_same_write_gate_as_individual_proposals() {
        assert!(mutating_request(&Request::ReviewBatch {
            workspace: "w".into(),
            selections: Vec::new(),
        }));
        assert!(!mutating_request(&Request::Batches {
            workspace: "w".into(),
            offset: 0,
        }));
        assert!(mutating_request(&Request::ReviewRemoteObservation {
            workspace: "w".into(),
            worktree: "t".into(),
            remote: "origin".into(),
            reference: "refs/heads/main".into(),
        }));
        assert!(mutating_request(&Request::ReviewFetchCapture {
            workspace: "w".into(),
            worktree: "t".into(),
            observation: "o".into()
        }));
        assert!(mutating_request(&Request::ReviewFetchImport {
            workspace: "w".into(),
            worktree: "t".into(),
            capture: "c".into()
        }));
        assert!(mutating_request(&Request::ReviewFetchImportRecovery {
            workspace: "w".into(),
            operation: "i".into()
        }));
        assert!(!mutating_request(&Request::FetchImports {
            workspace: "w".into(),
            offset: 0
        }));
        assert!(!mutating_request(&Request::FetchCaptures {
            workspace: "w".into(),
            worktree: None,
            offset: 0
        }));
        assert!(!mutating_request(&Request::RemoteObservations {
            workspace: "w".into(),
            worktree: None,
            offset: 0,
        }));
    }
    #[test]
    fn renderer_reload_revokes_native_context_and_requests_core_reset() {
        let (host, _) = host();
        let cancel = Cancellation::default();
        *host.active.lock().unwrap() = Some(cancel.clone());
        *host.review.lock().unwrap() = Some("old-review".into());
        let generation = host.generation.load(std::sync::atomic::Ordering::Acquire);
        host.invalidate_presentation();
        assert!(cancel.is_cancelled());
        assert!(host.review.lock().unwrap().is_none());
        assert!(host.reset.load(std::sync::atomic::Ordering::Acquire));
        assert_ne!(
            generation,
            host.generation.load(std::sync::atomic::Ordering::Acquire)
        );
    }
    #[test]
    fn production_configuration_has_no_remote_plugins_or_broad_permissions() {
        let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["app"]["security"]["assetProtocol"]["enable"], false);
        assert!(config.get("plugins").is_none());
        assert!(!config["app"]["security"]["csp"]
            .as_str()
            .unwrap()
            .contains("unsafe-eval"));
        for raw in [
            include_str!("../capabilities/main.json"),
            include_str!("../capabilities/review.json"),
        ] {
            let cap: Value = serde_json::from_str(raw).unwrap();
            assert!(cap.get("remote").is_none());
            assert!(cap.get("windows").is_none());
            assert!(!raw.contains("*"));
            assert!(!raw.contains("fs:"));
            assert!(!raw.contains("shell:"));
        }
    }
    #[test]
    fn queue_dialog_and_shutdown_boundaries_fail_closed() {
        let (host, _rx) = host();
        let lease = DialogLease::new(host.clone()).unwrap();
        assert!(DialogLease::new(host.clone()).is_err());
        drop(lease);
        for _ in 0..8 {
            assert!(queue(&host, Task::CancelReview("not-authority".into())).is_ok());
        }
        assert!(queue(&host, Task::CancelReview("overflow".into())).is_err());
        host.closing
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(queue(&host, Task::Shutdown).is_err());
    }
    fn invoke(
        window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: Value,
    ) -> Result<Value, Value> {
        tauri::test::get_ipc_response(
            window,
            tauri::webview::InvokeRequest {
                cmd: cmd.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.into(),
            },
        )
        .map(|r| r.deserialize::<Value>().unwrap())
    }
    #[test]
    fn actual_generated_command_acl_separates_main_review_and_unprivileged_views() {
        let (host, rx) = host();
        let app = tauri::test::mock_builder()
            .manage(host.clone())
            .invoke_handler(tauri::generate_handler![
                desktop_capabilities,
                dispatch,
                job_status,
                cancel_job,
                review_data,
                review_confirm,
                review_cancel,
                close_snapshot
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main =
            tauri::WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
                .build()
                .unwrap();
        let review =
            tauri::WebviewWindowBuilder::new(&app, "review", WebviewUrl::App("review.html".into()))
                .build()
                .unwrap();
        let stranger = tauri::WebviewWindowBuilder::new(
            &app,
            "stranger",
            WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let idle = invoke(&main, "job_status", json!({"job":"native-write"})).unwrap();
        assert_eq!(idle["state"], "idle");
        assert_eq!(idle["response"]["revision"], 0);
        host.completed
            .store(2, std::sync::atomic::Ordering::Release);
        let completed = invoke(&main, "job_status", json!({"job":"native-write"})).unwrap();
        assert_eq!(completed["state"], "idle");
        assert_eq!(completed["response"]["revision"], 2);
        assert!(invoke(&review, "job_status", json!({"job":"native-write"})).is_err());
        assert!(invoke(
            &main,
            "close_snapshot",
            json!({"token":"forged","payload":"{}"})
        )
        .unwrap_err()
        .to_string()
        .contains("No native close"));
        assert!(invoke(
            &review,
            "close_snapshot",
            json!({"token":"forged","payload":"{}"})
        )
        .unwrap_err()
        .to_string()
        .contains("not allowed"));
        assert!(invoke(
            &stranger,
            "close_snapshot",
            json!({"token":"forged","payload":"{}"})
        )
        .unwrap_err()
        .to_string()
        .contains("not allowed"));
        let caps = invoke(&main, "desktop_capabilities", json!({})).unwrap();
        assert_eq!(caps["writes_enabled"], cfg!(feature = "desktop-writes"));
        let denied = invoke(
            &main,
            "review_confirm",
            json!({"review":"forged","approved":true}),
        )
        .unwrap_err();
        assert!(denied.to_string().contains("not allowed"), "{denied}");
        assert!(invoke(&review, "dispatch", json!({"payload":"{}"}))
            .unwrap_err()
            .to_string()
            .contains("not allowed"));
        assert!(invoke(&stranger, "desktop_capabilities", json!({}))
            .unwrap_err()
            .to_string()
            .contains("not allowed"));
        assert!(invoke(&main, "run_command", json!({"command":"touch sentinel"})).is_err());
        let job=invoke(&main,"dispatch",json!({"payload":json!({"version":1,"epoch":1,"request":{"kind":"workspaces"}}).to_string()})).unwrap();
        assert!(job.is_string());
        let Task::Request { request, .. } = rx.recv_timeout(Duration::from_secs(1)).unwrap() else {
            panic!("Unexpected job")
        };
        assert!(matches!(request, Request::Workspaces {}));
    }
}
