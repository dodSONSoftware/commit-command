#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(target_os = "macos")]
mod ai_credentials;
#[cfg(target_os = "macos")]
mod ai_token_dialog;
#[cfg(target_os = "macos")]
use ai_token_dialog::manage_ai_token;
mod host_options;
#[cfg(feature = "native-acceptance-support")]
mod native_acceptance;
mod native_dialog;
mod repository_folder;
use repository_folder::{FolderTarget, RepositoryPathAction};
#[cfg(target_os = "macos")]
mod native_termination;
mod security;
mod shutdown;
use shutdown::close_snapshot;
mod window_state;
use gitfleet_core::{
    application::{Application, Data, Request, WindowPlacement},
    inspection::Cancellation,
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
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
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
    finish_on_close: bool,
    generation: u64,
    created: Instant,
    cancel: Cancellation,
    view: JobView,
}
struct Host {
    sender: SyncSender<Task>,
    jobs: Mutex<BTreeMap<String, Job>>,
    closing: std::sync::atomic::AtomicBool,
    active: Mutex<Option<Cancellation>>,
    dialog: std::sync::atomic::AtomicBool,
    generation: std::sync::atomic::AtomicU64,
    reset: std::sync::atomic::AtomicBool,
    completed: std::sync::atomic::AtomicU64,
    ui_scale: std::sync::atomic::AtomicU16,
    ai_suggestion: Mutex<Option<AiSuggestion>>,
    settings_placement: Mutex<Option<WindowPlacement>>,
    close_state: shutdown::CloseState,
    settings_close: shutdown::SettingsCloseState,
    shutdown_complete: std::sync::atomic::AtomicBool,
}
enum Task {
    SaveUiScale {
        percent: u16,
        reply: mpsc::Sender<Result<(), String>>,
    },
    AiCredentialBinding {
        connection: String,
        reply: mpsc::Sender<Result<gitfleet_core::ai::credentials::Binding, String>>,
    },
    #[cfg(feature = "native-acceptance-support")]
    AcceptanceFault(gitfleet_core::writes::Phase),
    Request {
        job: String,
        caller: String,
        epoch: u64,
        request: Request,
        cancel: Cancellation,
    },
    RequestDirectory(NativeDirectory),
    RepositoryPathAction {
        target: FolderTarget,
        action: RepositoryPathAction,
        reply: mpsc::Sender<Result<(), String>>,
    },
    RequestAiImport {
        path: std::path::PathBuf,
        reply: mpsc::Sender<Result<Value, String>>,
    },
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
fn check_settings_or_main<R: tauri::Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    check(
        window,
        match window.label() {
            "settings" => "settings",
            "ai-suggestions" => "ai-suggestions",
            _ => "main",
        },
    )
}
#[tauri::command]
async fn open_settings<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    workspace: Option<String>,
    worktree: Option<String>,
    theme: String,
) -> Result<(), String> {
    check(&window, "main")?;
    if !["system", "light", "dark"].contains(&theme.as_str()) {
        return Err("Invalid theme".into());
    }
    if window
        .app_handle()
        .get_webview_window("ai-suggestions")
        .is_some()
    {
        return Err("Close the current dialog first".into());
    }
    if let Some(settings) = window.app_handle().get_webview_window("settings") {
        settings.show().map_err(|e| e.to_string())?;
        return settings.set_focus().map_err(|e| e.to_string());
    }
    let context = json!({"workspace":workspace,"worktree":worktree,"theme":theme,"uiScale":state.ui_scale.load(std::sync::atomic::Ordering::Acquire)});
    present_owned_dialog(&window, &state, context, "Commit Command — Settings").await
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AiSuggestion {
    message: String,
    body_only: bool,
}
#[tauri::command]
async fn open_ai_suggestions<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    context: Value,
) -> Result<Option<AiSuggestion>, String> {
    check(&window, "main")?;
    if ["settings", "ai-suggestions"]
        .iter()
        .any(|label| window.app_handle().get_webview_window(label).is_some())
    {
        return Err("Close the current dialog first".into());
    }
    if context.to_string().len() > security::MAX_REQUEST {
        return Err("AI selection is too large".into());
    }
    *lock(&state.ai_suggestion)? = None;
    let context = json!({"ai": context, "theme": context["theme"], "uiScale": state.ui_scale.load(std::sync::atomic::Ordering::Acquire)});
    present_owned_dialog(&window, &state, context, "Commit Command — AI message").await?;
    Ok(lock(&state.ai_suggestion)?.take())
}
#[tauri::command]
fn use_ai_suggestion<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    suggestion: AiSuggestion,
) -> Result<(), String> {
    check(&window, "ai-suggestions")?;
    if suggestion.message.len() > security::MAX_REQUEST {
        return Err("AI message is too large".into());
    }
    *lock(&state.ai_suggestion)? = Some(suggestion);
    window.close().map_err(|e| e.to_string())
}
async fn present_owned_dialog<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    state: &Host,
    context: Value,
    title: &str,
) -> Result<(), String> {
    let role = if context.get("ai").is_some() {
        "ai-suggestions"
    } else {
        "settings"
    };
    let settings = WebviewWindowBuilder::new(
        window.app_handle(),
        role,
        WebviewUrl::App("settings.html".into()),
    )
    .title(title)
    .inner_size(960., 760.)
    .min_inner_size(620., 420.)
    .resizable(true)
    .decorations(true)
    .minimizable(false)
    .maximizable(false)
    .visible(false)
    .parent(window)
    .map_err(|e| e.to_string())?
    .initialization_script(format!("window.__COMMIT_COMMAND_SETTINGS__={context};"))
    .on_navigation(move |url| security::authorize(role, url, role).is_ok())
    .build()
    .map_err(|e| e.to_string())?;
    if context.get("ai").is_none() {
        if let Some(placement) = *lock(&state.settings_placement)? {
            window_state::restore_settings(&settings, placement).map_err(|e| e.to_string())?;
        }
        let geometry = settings.clone();
        let host = window.state::<Arc<Host>>().inner().clone();
        settings.on_window_event(move |event| {
            if matches!(
                event,
                tauri::WindowEvent::Moved(_)
                    | tauri::WindowEvent::Resized(_)
                    | tauri::WindowEvent::CloseRequested { .. }
            ) {
                match window_state::capture(&geometry) {
                    Ok(Some(placement)) => {
                        if let Ok(mut saved) = host.settings_placement.lock() {
                            *saved = Some(placement);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => eprintln!("Settings placement unavailable: {error}"),
                }
            }
        });
    }
    settings
        .set_zoom(f64::from(state.ui_scale.load(std::sync::atomic::Ordering::Acquire)) / 100.)
        .map_err(|e| e.to_string())?;
    native_dialog::present_settings(window, &settings).await
}
#[tauri::command]
fn set_settings_theme<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    theme: String,
) -> Result<(), String> {
    check(&window, "settings")?;
    if !["system", "light", "dark"].contains(&theme.as_str()) {
        return Err("Invalid theme".into());
    }
    window
        .app_handle()
        .emit_to("main", "commit-command-theme", theme)
        .map_err(|e| e.to_string())
}
fn settings_request(request: &Request) -> bool {
    matches!(
        request,
        Request::AppPreferences {}
            | Request::SaveAppPreference { .. }
            | Request::AiSettings {}
            | Request::SaveAiSettings {
                change: gitfleet_core::ai::change::Change::SingleSetup { .. }
                    | gitfleet_core::ai::change::Change::Connection(_),
                ..
            }
            | Request::ReviewAiDiscovery { .. }
            | Request::AiJobs {}
    )
}
fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    mutex.lock().map_err(|_| "Host state unavailable".into())
}
fn queue(host: &Host, task: Task) -> Result<(), String> {
    if host.closing.load(std::sync::atomic::Ordering::Acquire)
        && !matches!(&task, Task::Request { caller, request:Request::SaveAiSettings{..},.. } if caller=="settings" && host.settings_close.pending())
        && !matches!(
            &task,
            Task::Request {
                request: Request::SaveAppPreference { .. },
                ..
            }
        )
    {
        return Err("Application is closing".into());
    }
    host.sender
        .try_send(task)
        .map_err(|_| "Application busy; request queue is bounded".into())
}
#[tauri::command]
fn open_project_repository<R: tauri::Runtime>(window: WebviewWindow<R>) -> Result<(), String> {
    check(&window, "main")?;
    #[cfg(target_os = "macos")]
    {
        // Fixed application metadata only; IPC cannot supply a URL or shell command.
        let status = std::process::Command::new("/usr/bin/open")
            .arg("https://github.com/dodSONSoftware/commit-command")
            .status()
            .map_err(|_| "Could not open the project repository".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("Could not open the project repository".into())
        }
    }
    #[cfg(not(target_os = "macos"))]
    Err("Opening the project repository is supported on macOS".into())
}
#[tauri::command]
fn desktop_capabilities<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
) -> Result<Value, String> {
    check_settings_or_main(&window)?;
    Ok(
        json!({"writes_enabled":cfg!(feature="desktop-writes"),"protocol":1,"ui_scale_percent":state.ui_scale.load(std::sync::atomic::Ordering::Acquire)}),
    )
}
#[tauri::command]
async fn set_ui_scale<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    percent: u16,
) -> Result<u16, String> {
    check_settings_or_main(&window)?;
    if !(75..=200).contains(&percent) || !percent.is_multiple_of(5) {
        return Err("UI scale must be 75–200 percent in increments of five".into());
    }
    let previous = state.ui_scale.load(std::sync::atomic::Ordering::Acquire);
    let windows: Vec<_> = ["main", "settings", "review"]
        .iter()
        .filter_map(|label| window.app_handle().get_webview_window(label))
        .collect();
    let restore = |error: String| -> String {
        let failures: Vec<_> = windows
            .iter()
            .filter_map(|view| {
                view.set_zoom(f64::from(previous) / 100.)
                    .err()
                    .map(|e| e.to_string())
            })
            .collect();
        if failures.is_empty() {
            error
        } else {
            format!(
                "{error}; restoring UI scale failed: {}",
                failures.join("; ")
            )
        }
    };
    for view in &windows {
        if let Err(error) = view.set_zoom(f64::from(percent) / 100.) {
            return Err(restore(error.to_string()));
        }
    }
    let (reply, rx) = mpsc::channel();
    let result = match queue(&state, Task::SaveUiScale { percent, reply }) {
        Ok(()) => tauri::async_runtime::spawn_blocking(move || {
            rx.recv()
                .map_err(|_| "UI scale save was interrupted".to_owned())?
        })
        .await
        .map_err(|_| "UI scale save failed".to_owned())
        .and_then(|result| result),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        return Err(restore(error));
    }
    state
        .ui_scale
        .store(percent, std::sync::atomic::Ordering::Release);
    Ok(percent)
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
        Request::ExecuteReviewedOperation { .. }
            | Request::CancelReviewedOperation { .. }
            | Request::ReviewVersionAction { .. }
            | Request::ReviewVersionRecovery { .. }
            | Request::ReviewVersionStage { .. }
            | Request::ReviewAuthentication { .. }
            | Request::ReviewTrust { .. }
            | Request::ReviewBatch { .. }
            | Request::ReviewRemoteObservation { .. }
            | Request::ReviewPushCheck { .. }
            | Request::ReviewPushPublication { .. }
            | Request::ReviewPushBatchItem { .. }
            | Request::ReviewPushPublicationRecovery { .. }
            | Request::ReviewFetchCapture { .. }
            | Request::ReviewFetchImport { .. }
            | Request::ReviewFetchImportRecovery { .. }
            | Request::ReviewUpstreamSetup { .. }
            | Request::ReviewUpstreamRecovery { .. }
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
    check_settings_or_main(&window)?;
    let input = decode(&payload)?;
    if window.label() == "settings" && !settings_request(&input.request) {
        return Err("Request is not available in Settings".into());
    }
    if matches!(input.request, Request::ReviewAiEditorTest { .. }) {
        return Err("Desktop source transfers require a saved AI profile".into());
    }
    if window.label() == "ai-suggestions"
        && !matches!(
            input.request,
            Request::AppPreferences {}
                | Request::SaveAppPreference { .. }
                | Request::PreviewAiContext { .. }
                | Request::ReviewAiMessages { .. }
                | Request::AiJobs {}
                | Request::CancelAiJob { .. }
        )
    {
        return Err("Request is not available in AI suggestions".into());
    }
    if !cfg!(feature = "desktop-writes") && mutating_request(&input.request) {
        return Err("Desktop writes are disabled in this read-only build".into());
    }
    if lock(&state.active)?.is_some()
        && matches!(
            input.request,
            Request::ExecuteReviewedOperation { .. }
                | Request::Activate { .. }
                | Request::Create { .. }
                | Request::Update { .. }
                | Request::CloseInterruptedPushBatch { .. }
                | Request::Remove { .. }
                | Request::Propose { .. }
                | Request::ReviewAiDiscovery { .. }
                | Request::ReviewAiModelLoad { .. }
                | Request::ReviewAiInference { .. }
                | Request::ReviewAiMessages { .. }
                | Request::ReviewAiEditorTest { .. }
                | Request::ReviewAiImport { .. }
                | Request::PreviewAiImport { .. }
                | Request::ReviewVersionAction { .. }
                | Request::ReviewVersionRecovery { .. }
                | Request::ReviewVersionStage { .. }
                | Request::ReviewAuthentication { .. }
                | Request::ReviewTrust { .. }
                | Request::ReviewBatch { .. }
                | Request::ReviewRemoteObservation { .. }
                | Request::ReviewPushCheck { .. }
                | Request::ReviewPushPublication { .. }
                | Request::ReviewPushBatchItem { .. }
                | Request::ReviewPushPublicationRecovery { .. }
                | Request::ReviewRecovery { .. }
                | Request::ReviewFetchCapture { .. }
                | Request::ReviewFetchImport { .. }
                | Request::ReviewFetchImportRecovery { .. }
                | Request::ReviewUpstreamSetup { .. }
                | Request::ReviewUpstreamRecovery { .. }
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
                finish_on_close: matches!(input.request, Request::SaveAppPreference { .. })
                    || (window.label() == "settings"
                        && matches!(input.request, Request::SaveAiSettings { .. })),
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
            caller: window.label().into(),
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
    check_settings_or_main(&window)?;
    if job == "native-write" && window.label() != "main" {
        return Err("Native operation is owned by the main window".into());
    }
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
    check_settings_or_main(&window)?;
    if job == "native-write" && window.label() != "main" {
        return Err("Native operation is owned by the main window".into());
    }
    if job == "native-write" {
        if let Some(cancel) = lock(&state.active)?.as_ref() {
            cancel.cancel();
        }
        return Ok(());
    }
    let jobs = lock(&state.jobs)?;
    let Some(entry) = jobs.get(&job) else {
        return Ok(());
    };
    if entry.owner != window.label()
        || entry.generation != state.generation.load(std::sync::atomic::Ordering::Acquire)
    {
        return Err("Job belongs to another view".into());
    }
    entry.cancel.cancel();
    Ok(())
}
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum DirectoryPickerPurpose {
    Workspace,
    Backup,
}
#[tauri::command]
async fn pick_ai_import<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
) -> Result<Value, String> {
    check_settings_or_main(&window)?;
    if lock(&state.active)?.is_some() {
        return Err("Controlled write is active".into());
    }
    let _lease = DialogLease::new(state.inner().clone())?;
    let path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title("Choose an AI configuration backup")
            .set_parent(&window)
            .add_filter("Commit Command backup", &["zip"])
            .pick_file()
    })
    .await
    .map_err(|_| "Native archive picker failed")?;
    let Some(path) = path else {
        return Ok(Value::Null);
    };
    let (reply, rx) = mpsc::channel();
    queue(&state, Task::RequestAiImport { path, reply })?;
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(60))
            .map_err(|_| "Archive inspection timed out".to_owned())?
    })
    .await
    .map_err(|_| "Archive inspection response failed".to_owned())?
}
#[tauri::command]
async fn pick_directory<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    purpose: Option<DirectoryPickerPurpose>,
) -> Result<Value, String> {
    check_settings_or_main(&window)?;
    if window.label() == "settings" && !matches!(purpose, Some(DirectoryPickerPurpose::Backup)) {
        return Err("Settings can only choose backup destinations".into());
    }
    let _lease = DialogLease::new(state.inner().clone())?;
    let path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(match purpose.unwrap_or(DirectoryPickerPurpose::Workspace) {
                DirectoryPickerPurpose::Workspace => "Choose Commit Command workspace root",
                DirectoryPickerPurpose::Backup => "Choose parent folder for Commit Command backups",
            })
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
#[tauri::command]
async fn pick_authentication_file<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    kind: String,
) -> Result<Option<String>, String> {
    check(&window, "main")?;
    if lock(&state.active)?.is_some() {
        return Err("Controlled write is active".into());
    }
    let title = match kind.as_str() {
        "helper" => "Choose credential helper executable",
        "known_hosts" => "Choose verified known_hosts file",
        "public_identity" => "Choose Ed25519 public key (.pub)",
        "ca_file" => "Choose certificate authority file",
        _ => return Err("Unknown authentication file kind".into()),
    };
    let _lease = DialogLease::new(state.inner().clone())?;
    let path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(title)
            .set_parent(&window)
            .pick_file()
    })
    .await
    .map_err(|_| "Native picker failed")?;
    path.map(|p| {
        p.into_os_string()
            .into_string()
            .map_err(|_| "Authentication paths require UTF-8".into())
    })
    .transpose()
}
struct NativeDirectory {
    path: PathBuf,
    reply: mpsc::Sender<Result<Value, String>>,
}
fn run_active<T>(
    host: &Host,
    cancel: &Cancellation,
    run: impl FnOnce() -> gitfleet_core::Result<T>,
) -> gitfleet_core::Result<T> {
    {
        let mut active = host.active.lock().map_err(|_| {
            gitfleet_core::Error::with_kind(
                gitfleet_core::ErrorKind::Unavailable,
                "Operation state unavailable",
            )
        })?;
        if active.is_some() {
            return Err(gitfleet_core::Error::with_kind(
                gitfleet_core::ErrorKind::Busy,
                "Another operation is running",
            ));
        }
        *active = Some(cancel.clone());
    }
    let result = run();
    if let Ok(mut active) = host.active.lock() {
        *active = None;
    }
    host.completed
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    result
}

fn process_request(
    core: &mut Application,
    host: &Host,
    epoch: u64,
    request: Request,
    generation: u64,
    cancel: Cancellation,
    approve_source: impl FnOnce(&gitfleet_core::ai::provider::Review) -> bool,
) -> gitfleet_core::Result<gitfleet_core::application::Response> {
    let check_context = || {
        if generation != host.generation.load(std::sync::atomic::Ordering::Acquire)
            || cancel.is_cancelled()
        {
            return Err(gitfleet_core::Error::with_kind(
                gitfleet_core::ErrorKind::Cancelled,
                "Operation cancelled because the application context changed",
            ));
        }
        Ok(())
    };
    let writes_disabled = || {
        gitfleet_core::Error::with_kind(
            gitfleet_core::ErrorKind::Unavailable,
            "Desktop writes are disabled in this read-only build",
        )
    };
    if matches!(request, Request::ExecuteReviewedOperation { .. }) {
        if !cfg!(feature = "desktop-writes") {
            return Err(writes_disabled());
        }
        return run_active(host, &cancel, || {
            check_context()?;
            core.request_with_cancellation(epoch, request, cancel.clone())
        });
    }
    let mut response = core.request_with_cancellation(epoch, request, cancel.clone())?;
    let Data::Review(display) = &response.data else {
        return Ok(response);
    };
    let id = display.id.clone();
    if !cfg!(feature = "desktop-writes") && !display.allow_in_read_only_build {
        core.cancel_review(&id)?;
        return Err(writes_disabled());
    }
    if let Err(error) = check_context() {
        core.cancel_review(&id)?;
        return Err(error);
    }
    if display.operation.is_some() {
        return Ok(response);
    }
    if let Some(review) = core.reviewed_ai_source(&id)? {
        if !approve_source(&review) {
            core.cancel_review(&id)?;
            return Err(gitfleet_core::Error::with_kind(
                gitfleet_core::ErrorKind::Cancelled,
                "Repository source transfer was declined",
            ));
        }
        if let Err(error) = check_context() {
            core.cancel_review(&id)?;
            return Err(error);
        }
    }
    let result = run_active(host, &cancel, || core.execute_prepared(&id, cancel.clone()))?;
    response.data = Data::Executed(result);
    response.epoch = core.epoch();
    Ok(response)
}
fn worker(mut core: Application, rx: mpsc::Receiver<Task>, app: tauri::AppHandle, host: Arc<Host>) {
    let mut saved_settings = core.settings_window_placement().ok().flatten();
    let mut settings_saved_at = Instant::now();
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
                caller,
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
                #[cfg(feature = "native-acceptance-support")]
                native_acceptance::record(
                    "request_started",
                    json!({"job":job,"kind":serde_json::to_value(&request).ok().and_then(|v|v.get("kind").cloned())}),
                );
                let result = process_request(
                    &mut core,
                    &host,
                    epoch,
                    request,
                    generation,
                    cancel,
                    |review| {
                        app.get_webview_window(&caller)
                            .is_some_and(|parent| native_dialog::approve_ai_source(&parent, review))
                    },
                );
                #[cfg(feature = "native-acceptance-support")]
                native_acceptance::record(
                    "request_finished",
                    json!({"job":job,"ok":result.is_ok(),"epoch":core.epoch()}),
                );
                let response = result.and_then(|mut r| {
                    if caller == "settings" {
                        if let Data::AiJobs(jobs) = &mut r.data {
                            jobs.retain(|job| {
                                job.review.purpose == gitfleet_core::ai::provider::Purpose::Discover
                            });
                        }
                    }
                    let value = serde_json::to_value(&r)?;
                    if serde_json::to_vec(&value)?.len() > 16 * 1024 * 1024 {
                        // The user cannot act on a plan the renderer never received.
                        if let Data::Review(display) = &r.data {
                            core.cancel_review(&display.id)?;
                        }
                        return Err(gitfleet_core::Error::with_kind(
                            gitfleet_core::ErrorKind::OutputLimit,
                            "Desktop response limit exceeded",
                        ));
                    }
                    Ok(value)
                });
                if let Ok(mut jobs) = host.jobs.lock() {
                    if let Some(j) = jobs.get_mut(&job) {
                        j.view = match response {
                            Ok(value) => JobView {
                                state: "complete".into(),
                                response: Some(value),
                                error: None,
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
            Ok(Task::SaveUiScale { percent, reply }) => {
                let _ = reply.send(
                    core.save_ui_scale_percent(percent)
                        .map_err(|e| e.to_string()),
                );
            }
            Ok(Task::RepositoryPathAction {
                target,
                action,
                reply,
            }) => {
                let _ = reply.send(target.apply(&core, action));
            }
            Ok(Task::RequestDirectory(task)) => {
                let value = core
                    .register_directory(&task.path)
                    .map(|picker| json!({"picker":picker,"display":task.path.to_string_lossy()}))
                    .map_err(|e| e.to_string());
                let _ = task.reply.send(value);
            }
            Ok(Task::RequestAiImport { path, reply }) => {
                let value = core
                    .register_ai_import(&path)
                    .map(|source| json!({"source":source.source,"display":source.path}))
                    .map_err(|e| e.to_string());
                let _ = reply.send(value);
            }
            #[cfg(feature = "native-acceptance-support")]
            Ok(Task::AcceptanceFault(at)) => {
                native_acceptance::record("commit_fault_armed", json!({"phase":at}));
                core.set_write_test_fault(gitfleet_core::writes::TestFault {
                    at,
                    kind: gitfleet_core::writes::TestFaultKind::Sigkill,
                });
            }
            Ok(Task::AiCredentialBinding { connection, reply }) => {
                let _ = reply.send(
                    core.ai_credential_binding(&connection)
                        .map_err(|e| e.to_string()),
                );
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
                if let Ok(saved) = host.settings_placement.lock() {
                    if let Some(placement) = *saved {
                        if let Err(error) = core.save_settings_window_placement(placement) {
                            eprintln!("Settings placement could not be saved: {:?}", error.kind);
                        }
                    }
                }
                if let Err(error) = core.shutdown() {
                    eprintln!(
                        "Application shutdown state could not be saved: {:?}",
                        error.kind
                    );
                }
                #[cfg(target_os = "macos")]
                if let Err(error) = ai_credentials::shutdown_helpers() {
                    eprintln!("AI credential helper shutdown failed: {:?}", error.kind);
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
        if settings_saved_at.elapsed() >= Duration::from_millis(250) {
            let current = host.settings_placement.lock().ok().and_then(|saved| *saved);
            if current != saved_settings {
                if let Some(placement) = current {
                    match core.save_settings_window_placement(placement) {
                        Ok(()) => saved_settings = current,
                        Err(error) => {
                            eprintln!("Settings placement could not be saved: {:?}", error.kind)
                        }
                    }
                }
            }
            settings_saved_at = Instant::now();
        }
    }
}
#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn manage_ai_token<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    connection: String,
) -> Result<(), String> {
    check_settings_or_main(&window)?;
    let _ = connection;
    Err("AI secure authentication is not supported on this platform".into())
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, options) =
        host_options::parse(std::env::args_os().skip(1), std::env::var_os("HOME"))?;
    #[cfg(feature = "native-acceptance-support")]
    native_acceptance::initialize(&directory)?;
    let mut core = Application::open(&directory, options)?;
    #[cfg(target_os = "macos")]
    core.set_ai_secret_store(Arc::new(ai_credentials::KeychainStore::default()));
    #[cfg(feature = "native-acceptance-support")]
    native_acceptance::validate_workspaces(&mut core, &directory)?;
    let saved_placement = core.window_placement()?;
    let settings_placement = core.settings_window_placement()?;
    let saved_ui_scale = core.ui_scale_percent()?;
    let (sender, rx) = mpsc::sync_channel(8);
    let host = Arc::new(Host {
        sender,
        jobs: Mutex::new(BTreeMap::new()),
        closing: std::sync::atomic::AtomicBool::new(false),
        active: Mutex::new(None),
        dialog: std::sync::atomic::AtomicBool::new(false),
        generation: std::sync::atomic::AtomicU64::new(1),
        reset: std::sync::atomic::AtomicBool::new(false),
        completed: std::sync::atomic::AtomicU64::new(0),
        ui_scale: std::sync::atomic::AtomicU16::new(saved_ui_scale),
        ai_suggestion: Mutex::new(None),
        settings_placement: Mutex::new(settings_placement),
        close_state: shutdown::CloseState::default(),
        settings_close: shutdown::SettingsCloseState::default(),
        shutdown_complete: std::sync::atomic::AtomicBool::new(false),
    });
    let setup_host = host.clone();
    tauri::Builder::default()
        .manage(host.clone())
        .invoke_handler(tauri::generate_handler![
            open_settings,
            open_ai_suggestions,
            use_ai_suggestion,
            set_settings_theme,
            desktop_capabilities,
            set_ui_scale,
            manage_ai_token,
            open_project_repository,
            repository_folder::repository_path_action,
            dispatch,
            pick_directory,
            pick_ai_import,
            pick_authentication_file,
            job_status,
            cancel_job,
            close_snapshot,
            shutdown::settings_flush_done
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
                    "Commit Command",
                    true,
                    &[
                        &MenuItem::with_id(
                            app,
                            "gitfleet-about",
                            "About Commit Command",
                            true,
                            None::<&str>,
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::services(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, None)?,
                        &PredefinedMenuItem::hide_others(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "gitfleet-quit",
                            "Quit Commit Command",
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
                    } else if event.id().as_ref() == "gitfleet-about" {
                        if let Some(main) = app.get_webview_window("main") {
                            let _ = main.set_focus();
                            let _ = main.emit("commit-command-about", ());
                        }
                    }
                });
            }
            #[cfg(target_os = "macos")]
            native_termination::install(app.handle().clone(), setup_host.clone())?;
            #[cfg(feature = "native-acceptance-support")]
            native_acceptance::install(app.handle(), setup_host.clone())?;
            let page_host = setup_host.clone();
            let page_loaded = std::sync::atomic::AtomicBool::new(false);
            let initially_shown = std::sync::atomic::AtomicBool::new(false);
            #[cfg(feature = "native-acceptance-support")]
            let page_started = Instant::now();
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("Commit Command")
                    .visible(false)
                    .inner_size(1440., 940.)
                    .min_inner_size(window_state::MIN_WIDTH, window_state::MIN_HEIGHT)
                    .devtools(false)
                    .on_page_load(move |window, payload| {
                        #[cfg(feature = "native-acceptance-support")]
                        native_acceptance::record(
                            "main_page_load",
                            json!({
                                "event": format!("{:?}", payload.event()),
                                "visible": window.is_visible().ok(),
                                "elapsed_ms": page_started.elapsed().as_millis(),
                            }),
                        );
                        if payload.event() == tauri::webview::PageLoadEvent::Started
                            && page_loaded.swap(true, std::sync::atomic::Ordering::AcqRel)
                        {
                            #[cfg(feature = "native-acceptance-support")]
                            native_acceptance::record("main_renderer_reset", json!({}));
                            page_host.invalidate_presentation();
                        }
                        // WebKit loads asynchronously. Showing during setup exposes
                        // an empty surface, particularly on a cold/crash restart.
                        // Later renderer reloads must not unhide the user's window.
                        if payload.event() == tauri::webview::PageLoadEvent::Finished
                            && !initially_shown.swap(true, std::sync::atomic::Ordering::AcqRel)
                        {
                            // On macOS, show makes the window key but does not activate
                            // the app. A background launch can leave WebKit's mounted
                            // document hidden until the first native menu interaction.
                            // Activate only this initial display, never a later reload.
                            if let Err(error) = window.show().and_then(|()| window.set_focus()) {
                                eprintln!("Commit Command window display failed: {error}");
                            }
                            #[cfg(feature = "native-acceptance-support")]
                            native_acceptance::record(
                                "main_initial_display",
                                json!({"visible":window.is_visible().ok()}),
                            );
                        }
                    })
                    .on_navigation(|url| {
                        let allowed = security::authorize("main", url, "main").is_ok();
                        #[cfg(feature = "native-acceptance-support")]
                        native_acceptance::record(
                            "navigation",
                            json!({"label":"main","allowed":allowed}),
                        );
                        allowed
                    })
                    .on_document_title_changed(|window, title| {
                        #[cfg(feature = "native-acceptance-support")]
                        native_acceptance::record_probe_title(window.label(), &title);
                        #[cfg(not(feature = "native-acceptance-support"))]
                        let _ = (window, title);
                    })
                    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                    .build()?;
            if let Some(placement) = saved_placement {
                window_state::restore(&window, placement)?;
            }
            window.set_zoom(f64::from(saved_ui_scale) / 100.)?;
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
    #[cfg(target_os = "macos")]
    ai_credentials::maybe_worker();
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("--gitfleet-credential-bridge"))
    {
        std::process::exit(gitfleet_core::credentials::bridge_main());
    }
    if let Err(error) = run() {
        eprintln!("Commit Command startup failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn host() -> (Arc<Host>, mpsc::Receiver<Task>) {
        let (sender, rx) = mpsc::sync_channel(8);
        (
            Arc::new(Host {
                sender,
                jobs: Mutex::new(BTreeMap::new()),
                closing: std::sync::atomic::AtomicBool::new(false),
                active: Mutex::new(None),
                dialog: std::sync::atomic::AtomicBool::new(false),
                generation: std::sync::atomic::AtomicU64::new(1),
                reset: std::sync::atomic::AtomicBool::new(false),
                completed: std::sync::atomic::AtomicU64::new(0),
                ui_scale: std::sync::atomic::AtomicU16::new(100),
                ai_suggestion: Mutex::new(None),
                settings_placement: Mutex::new(None),
                close_state: shutdown::CloseState::default(),
                settings_close: shutdown::SettingsCloseState::default(),
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
        for request in [
            Request::ExecuteReviewedOperation {
                id: "review".into(),
                operation: gitfleet_core::application::ReviewedOperation::Push,
            },
            Request::CancelReviewedOperation {
                id: "review".into(),
                operation: gitfleet_core::application::ReviewedOperation::BulkCommit,
            },
        ] {
            assert!(mutating_request(&request));
            assert!(!settings_request(&request));
        }
        assert!(mutating_request(&Request::ReviewAuthentication {
            change: gitfleet_core::authentication::Change::Delete {
                id: "profile".into()
            }
        }));
        assert!(!mutating_request(&Request::AuthenticationSettings {}));
        assert!(mutating_request(&Request::ReviewPushCheck {
            workspace: "w".into(),
            selection: gitfleet_core::remote::push::PushPlanningSelection {
                worktree: "t".into(),
                observation: "o".into(),
                source_oid: gitfleet_core::domain::ObjectId {
                    format: gitfleet_core::domain::ObjectFormat::Sha1,
                    hex: "a".repeat(40)
                },
                children: vec![]
            },
            hooks: gitfleet_core::writes::HookPolicy::Execute
        }));
        assert!(mutating_request(&Request::ReviewPushPublication {
            workspace: "w".into(),
            selection: gitfleet_core::remote::push::PushPlanningSelection {
                worktree: "t".into(),
                observation: "o".into(),
                source_oid: gitfleet_core::domain::ObjectId {
                    format: gitfleet_core::domain::ObjectFormat::Sha1,
                    hex: "a".repeat(40)
                },
                children: vec![]
            },
            hooks: gitfleet_core::writes::HookPolicy::Execute
        }));
        assert!(mutating_request(&Request::ReviewPushBatchItem {
            workspace: "w".into(),
            batch: "queue".into(),
            selection: gitfleet_core::remote::push::PushPlanningSelection {
                worktree: "t".into(),
                observation: "o".into(),
                source_oid: gitfleet_core::domain::ObjectId {
                    format: gitfleet_core::domain::ObjectFormat::Sha1,
                    hex: "a".repeat(40)
                },
                children: vec![]
            }
        }));
        assert!(!mutating_request(&Request::CreatePushBatch {
            workspace: "w".into(),
            selections: vec![]
        }));
        assert!(mutating_request(&Request::ReviewPushPublicationRecovery {
            workspace: "w".into(),
            operation: "p".into(),
            observation: "o".into()
        }));
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
        assert!(mutating_request(&Request::ReviewUpstreamSetup {
            workspace: "w".into(),
            worktree: "t".into(),
            observation: "o".into()
        }));
        assert!(mutating_request(&Request::ReviewUpstreamRecovery {
            workspace: "w".into(),
            operation: "o".into()
        }));
        assert!(!mutating_request(&Request::PushPreview {
            workspace: "w".into(),
            worktree: "t".into(),
            observation: "o".into()
        }));
        assert!(mutating_request(&Request::ReviewVersionAction {
            workspace: "w".into(),
            worktree: "t".into(),
            path: gitfleet_core::domain::NativePath::Utf8("version".into()),
            increment: gitfleet_core::actions::semver::Increment::Patch,
            prerelease: gitfleet_core::actions::semver::PrereleasePolicy::Reject
        }));
        assert!(mutating_request(&Request::ReviewVersionRecovery {
            workspace: "w".into(),
            operation: "o".into()
        }));
        assert!(mutating_request(&Request::ReviewVersionStage {
            workspace: "w".into(),
            operation: "o".into()
        }));
        assert!(!mutating_request(&Request::AiJobs {}));
        assert!(!mutating_request(&Request::PreviewAiImport {
            source: "native-source".into(),
            choices: Default::default()
        }));
        assert!(!mutating_request(&Request::ReviewAiImport {
            preview: "frozen-preview".into(),
            fingerprint: "digest".into()
        }));
        assert!(!mutating_request(&Request::ExportAiSettings {
            picker: "native-picker".into(),
            fingerprint: "snapshot".into()
        }));
        assert!(!mutating_request(&Request::ReviewAiEditorTest {
            workspace: "w".into(),
            worktree: "t".into(),
            spec: gitfleet_core::ai::capture::Spec {
                selected: vec![],
                nearby_source: vec![],
                documentation: vec![],
                branch_commits: 0,
                file_commits: 0,
                policy: Default::default()
            },
            snapshot_fingerprint: "snapshot".into(),
            context_fingerprint: "context".into(),
            connection: gitfleet_core::ai::config::ConnectionDraft {
                id: None,
                name: "test".into(),
                provider: gitfleet_core::ai::config::Provider::Ollama,
                endpoint: "http://127.0.0.1:1/".into(),
                timeout_ms: 100,
                authentication: Default::default()
            },
            profile: Box::new(gitfleet_core::ai::config::ProfileDraft {
                id: None,
                name: "test".into(),
                connection: "".into(),
                model: "fixture".into(),
                system_message: "".into(),
                parameters: Default::default(),
                template: "".into(),
                context_bytes: 1024
            }),
            template: Box::default(),
            instructions: vec![],
        }));
        assert!(!mutating_request(&Request::CancelAiJob {
            id: "fixture".into()
        }));
        assert!(!mutating_request(&Request::ReviewAiDiscovery {
            connection: gitfleet_core::ai::config::ConnectionDraft {
                id: None,
                name: "fixture".into(),
                provider: gitfleet_core::ai::config::Provider::Ollama,
                endpoint: "http://127.0.0.1:1/".into(),
                timeout_ms: 100,
                authentication: Default::default(),
            }
        }));
        assert!(!mutating_request(&Request::AiSettings {}));
        assert!(!mutating_request(&Request::SaveAiSettings {
            fingerprint: "f".into(),
            revision: "r".into(),
            change: gitfleet_core::ai::change::Change::Concurrency {
                global: 1,
                per_connection: 1
            }
        }));
        assert!(!mutating_request(&Request::AiTemplatePreview {
            template: Default::default()
        }));
        assert!(!mutating_request(&Request::VersionRetainedFile {
            workspace: "w".into(),
            operation: "o".into()
        }));
        assert!(!mutating_request(&Request::VersionActions {
            workspace: "w".into(),
            worktree: None,
            offset: 0
        }));
        assert!(!mutating_request(&Request::PreviewVersionFile {
            workspace: "w".into(),
            worktree: "t".into(),
            path: gitfleet_core::domain::NativePath::Utf8("version".into()),
            increment: gitfleet_core::actions::semver::Increment::Patch,
            prerelease: gitfleet_core::actions::semver::PrereleasePolicy::Reject,
        }));
        assert!(!mutating_request(&serde_json::from_value::<Request>(json!({
            "kind":"child_reachability", "workspace":"ws", "worktree":"parent",
            "selection":{
                "parent_observation":"parent-observation", "source_oid":{"format":"sha1","hex":"a".repeat(40)},
                "path":[99], "child_oid":{"format":"sha1","hex":"b".repeat(40)},
                "gitmodules_blob":{"format":"sha1","hex":"c".repeat(40)},
                "child_worktree":"child", "child_observation":"child-observation"
            }
        })).unwrap()));
        assert!(!mutating_request(&Request::PushDependencyPlan {
            workspace: "w".into(),
            selections: Vec::new()
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
        let generation = host.generation.load(std::sync::atomic::Ordering::Acquire);
        host.invalidate_presentation();
        assert!(cancel.is_cancelled());
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
        let settings: Value =
            serde_json::from_str(include_str!("../capabilities/settings.json")).unwrap();
        assert_eq!(settings["webviews"], json!(["settings"]));
        let permissions = settings["permissions"].as_array().unwrap();
        // onCloseRequested destroys the window after the async save handler allows closing.
        for permission in [
            "core:window:allow-close",
            "core:window:allow-destroy",
            "core:event:allow-listen",
            "core:event:allow-unlisten",
        ] {
            assert!(
                permissions.contains(&json!(permission)),
                "Settings needs {permission}"
            );
        }
        for raw in [
            include_str!("../capabilities/main.json"),
            include_str!("../capabilities/settings.json"),
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
            assert!(queue(&host, Task::Shutdown).is_ok());
        }
        assert!(queue(&host, Task::Shutdown).is_err());
        host.closing
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(queue(&host, Task::Shutdown).is_err());
    }
    pub(super) fn invoke(
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
    fn actual_generated_command_acl_separates_main_settings_and_unprivileged_views() {
        let (host, rx) = host();
        let app = tauri::test::mock_builder()
            .manage(host.clone())
            .invoke_handler(tauri::generate_handler![
                desktop_capabilities,
                set_ui_scale,
                manage_ai_token,
                open_project_repository,
                repository_folder::repository_path_action,
                dispatch,
                job_status,
                cancel_job,
                close_snapshot,
                shutdown::settings_flush_done,
                pick_authentication_file,
                pick_ai_import
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main =
            tauri::WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
                .build()
                .unwrap();
        let review = tauri::WebviewWindowBuilder::new(
            &app,
            "native-probe",
            WebviewUrl::App("settings.html".into()),
        )
        .build()
        .unwrap();
        let settings = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            WebviewUrl::App("settings.html".into()),
        )
        .build()
        .unwrap();
        assert!(invoke(
            &main,
            "settings_flush_done",
            json!({"token":"forged","saved":true})
        )
        .is_err());
        assert!(invoke(
            &settings,
            "settings_flush_done",
            json!({"token":"forged","saved":true})
        )
        .unwrap_err()
        .to_string()
        .contains("No native close"));
        assert!(invoke(&settings, "desktop_capabilities", json!({})).is_ok());
        assert!(invoke(&settings, "job_status", json!({"job":"native-write"})).is_err());
        assert!(invoke(&settings, "dispatch", json!({"payload":serde_json::to_string(&json!({"version":1,"epoch":0,"request":{"kind":"activate","workspace":"forbidden"}})).unwrap()})).unwrap_err().to_string().contains("not available in Settings"));
        let stranger = tauri::WebviewWindowBuilder::new(
            &app,
            "stranger",
            WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let idle = invoke(&main, "job_status", json!({"job":"native-write"})).unwrap();
        let path_action = json!({"path":{"encoding":"utf8","value":"relative"},"action":"reveal"});
        assert!(invoke(&main, "repository_path_action", path_action.clone())
            .unwrap_err()
            .to_string()
            .contains("target"));
        let folder_action =
            json!({"target":{"workspace":"fixture","worktree":"registered"},"action":"reveal"});
        for window in [&settings, &stranger] {
            assert!(
                invoke(window, "repository_path_action", folder_action.clone())
                    .unwrap_err()
                    .to_string()
                    .contains("not allowed")
            );
        }

        std::thread::scope(|scope| {
            let call = scope.spawn(|| invoke(&main, "repository_path_action", folder_action));
            let Task::RepositoryPathAction {
                action: RepositoryPathAction::Reveal,
                reply,
                ..
            } = rx.recv_timeout(Duration::from_secs(2)).unwrap()
            else {
                panic!("Unexpected folder operation")
            };
            reply.send(Ok(())).unwrap();
            assert!(call.join().unwrap().is_ok());
        });

        assert!(invoke(&main, "set_ui_scale", json!({"percent":70}))
            .unwrap_err()
            .to_string()
            .contains("UI scale must be"));
        assert!(invoke(&review, "set_ui_scale", json!({"percent":125})).is_err());
        assert!(invoke(&stranger, "set_ui_scale", json!({"percent":125})).is_err());
        *host.active.lock().unwrap() = Some(Cancellation::default());
        assert!(invoke(&main, "pick_ai_import", json!({}))
            .unwrap_err()
            .to_string()
            .contains("Controlled write is active"));
        assert!(invoke(&review, "pick_ai_import", json!({})).is_err());
        assert!(invoke(&stranger, "pick_ai_import", json!({})).is_err());
        *host.active.lock().unwrap() = None;
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
        assert!(invoke(
            &review,
            "pick_authentication_file",
            json!({"kind":"invalid"})
        )
        .unwrap_err()
        .to_string()
        .contains("not allowed"));
        assert!(
            invoke(&main, "pick_authentication_file", json!({"kind":"invalid"}))
                .unwrap_err()
                .to_string()
                .contains("Unknown authentication file kind")
        );
        let caps = invoke(&main, "desktop_capabilities", json!({})).unwrap();
        assert_eq!(caps["writes_enabled"], cfg!(feature = "desktop-writes"));
        assert!(invoke(&review, "dispatch", json!({"payload":"{}"}))
            .unwrap_err()
            .to_string()
            .contains("not allowed"));
        assert!(invoke(&stranger, "desktop_capabilities", json!({}))
            .unwrap_err()
            .to_string()
            .contains("not allowed"));
        for window in [&review, &stranger] {
            assert!(invoke(window, "open_project_repository", json!({}))
                .unwrap_err()
                .to_string()
                .contains("not allowed"));
        }
        for window in [&review, &stranger] {
            assert!(
                invoke(window, "manage_ai_token", json!({"connection":"fixture"}))
                    .unwrap_err()
                    .to_string()
                    .contains("not allowed")
            );
        }

        assert!(invoke(&main, "run_command", json!({"command":"touch sentinel"})).is_err());
        let job=invoke(&main,"dispatch",json!({"payload":json!({"version":1,"epoch":1,"request":{"kind":"workspaces"}}).to_string()})).unwrap();
        assert!(job.is_string());
        let Task::Request { request, .. } = rx.recv_timeout(Duration::from_secs(1)).unwrap() else {
            panic!("Unexpected job")
        };
        assert!(matches!(request, Request::Workspaces {}));
    }
}

#[cfg(all(test, target_os = "macos", feature = "desktop-writes"))]
mod publication_tests;

#[cfg(all(test, target_os = "macos"))]
mod write_gate_tests;

#[cfg(all(test, target_os = "macos"))]
#[allow(dead_code)]
#[path = "../../../core/tests/support/mod.rs"]
mod test_support;
