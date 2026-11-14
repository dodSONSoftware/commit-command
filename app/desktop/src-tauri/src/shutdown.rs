//! One-use close snapshots persist drafts through the existing Core owner before exit.
use super::*;
use gitfleet_core::application::{BulkComposerState, DesktopState};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    version: u32,
    epoch: u64,
    workspace: Option<String>,
    navigation: Option<DesktopState>,
    bulk: Option<BulkComposerState>,
}
impl Snapshot {
    fn decode(payload: &str) -> Result<Self, String> {
        if payload.len() > 4 * 1024 * 1024 + 4096 {
            return Err("Close snapshot exceeds size limit".into());
        }
        let snapshot: Self =
            serde_json::from_str(payload).map_err(|_| "Invalid close snapshot".to_owned())?;
        let actual: Value = serde_json::from_str(payload).map_err(|_| "Invalid snapshot")?;
        let canonical = serde_json::to_value(&snapshot).map_err(|_| "Invalid snapshot")?;
        if snapshot.version != 1
            || !security::known_fields(&actual, &canonical)
            || match &snapshot.workspace {
                Some(_) => snapshot.navigation.is_none() || snapshot.bulk.is_none(),
                None => snapshot.navigation.is_some() || snapshot.bulk.is_some(),
            }
        {
            return Err("Invalid close snapshot scope or version".into());
        }
        Ok(snapshot)
    }
    pub(super) fn save(self, core: &mut Application) -> Result<(), String> {
        match (self.workspace, self.navigation, self.bulk) {
            (Some(workspace), Some(navigation), Some(bulk)) => core
                .save_presentation(self.epoch, &workspace, &navigation, &bulk)
                .map_err(|_| "Latest drafts could not be saved".to_owned()),
            (None, None, None) => Ok(()),
            _ => Err("Invalid close snapshot".into()),
        }
    }
}
pub(super) struct Submission {
    snapshot: Snapshot,
    reply: mpsc::Sender<Result<(), String>>,
}
struct Attempt {
    token: String,
    generation: u64,
    sender: SyncSender<Submission>,
}
#[derive(Default)]
pub(super) struct CloseState(Mutex<Option<Attempt>>);
impl CloseState {
    fn begin(&self, generation: u64) -> Result<(String, mpsc::Receiver<Submission>), String> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut pending = lock(&self.0)?;
        if pending.is_some() {
            return Err("Close already pending".into());
        }
        let token = Uuid::new_v4().to_string();
        *pending = Some(Attempt {
            token: token.clone(),
            generation,
            sender,
        });
        Ok((token, receiver))
    }
    fn submit(
        &self,
        token: &str,
        generation: u64,
        snapshot: Snapshot,
    ) -> Result<mpsc::Receiver<Result<(), String>>, String> {
        let mut pending = lock(&self.0)?;
        if !pending
            .as_ref()
            .is_some_and(|a| a.token == token && a.generation == generation)
        {
            return Err("Close request is stale".into());
        }
        let attempt = pending.take().ok_or("Close request is stale")?;
        let (reply, receiver) = mpsc::channel();
        attempt
            .sender
            .try_send(Submission { snapshot, reply })
            .map_err(|_| "Close request is unavailable")?;
        Ok(receiver)
    }
    fn clear(&self) {
        if let Ok(mut pending) = self.0.lock() {
            *pending = None;
        }
    }
}

#[derive(Default)]
pub(super) struct SettingsCloseState(Mutex<Option<(String, u64, SyncSender<bool>)>>);
impl SettingsCloseState {
    fn begin(&self, generation: u64) -> Result<(String, mpsc::Receiver<bool>), String> {
        let mut pending = lock(&self.0)?;
        if pending.is_some() {
            return Err("Settings close already pending".into());
        }
        let token = Uuid::new_v4().to_string();
        let (sender, receiver) = mpsc::sync_channel(1);
        *pending = Some((token.clone(), generation, sender));
        Ok((token, receiver))
    }
    pub(super) fn pending(&self) -> bool {
        self.0.lock().is_ok_and(|p| p.is_some())
    }
    fn finish(&self, token: &str, generation: u64, saved: bool) -> Result<(), String> {
        let mut pending = lock(&self.0)?;
        if !pending
            .as_ref()
            .is_some_and(|(t, g, _)| t == token && *g == generation)
        {
            return Err("Settings close request is stale".into());
        }
        let (_, _, sender) = pending.take().ok_or("Settings close request is stale")?;
        sender
            .try_send(saved)
            .map_err(|_| "Settings close request unavailable".into())
    }
    fn clear(&self) {
        if let Ok(mut p) = self.0.lock() {
            *p = None;
        }
    }
}
#[tauri::command]
pub(super) fn settings_flush_done<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    token: String,
    saved: bool,
) -> Result<(), String> {
    check(&window, "settings")?;
    if !state.closing.load(std::sync::atomic::Ordering::Acquire) {
        return Err("No native close is pending".into());
    }
    state.settings_close.finish(
        &token,
        state.generation.load(std::sync::atomic::Ordering::Acquire),
        saved,
    )
}

#[tauri::command]
pub(super) async fn close_snapshot<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    token: String,
    payload: String,
) -> Result<(), String> {
    check(&window, "main")?;
    if !state.closing.load(std::sync::atomic::Ordering::Acquire) {
        return Err("No native close is pending".into());
    }
    let snapshot = Snapshot::decode(&payload)?;
    let receiver = state.close_state.submit(
        &token,
        state.generation.load(std::sync::atomic::Ordering::Acquire),
        snapshot,
    )?;
    tauri::async_runtime::spawn_blocking(move || {
        receiver
            .recv()
            .map_err(|_| "Close response unavailable".to_owned())?
    })
    .await
    .map_err(|_| "Close response unavailable".to_owned())?
}
fn discard_confirmed(app: &tauri::AppHandle) -> bool {
    let mut dialog = rfd::MessageDialog::new()
        .set_title("GitFleet — Unsaved drafts")
        .set_description("The latest drafts or Settings edits could not be saved. Keep the app open to retry, or close without saving the latest drafts. Existing Git operation records are kept.")
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::OkCancelCustom("Keep app open".into(), "Close without saving".into()));
    if let Some(main) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&main);
    }
    dialog.show() == rfd::MessageDialogResult::Custom("Close without saving".into())
}
pub(super) fn request(app: tauri::AppHandle, host: Arc<Host>) {
    if host.closing.swap(true, std::sync::atomic::Ordering::AcqRel) {
        return;
    }
    if let Ok(jobs) = host.jobs.lock() {
        for job in jobs.values() {
            if !job.finish_on_close {
                job.cancel.cancel();
            }
        }
    }
    if let Ok(active) = host.active.lock() {
        if let Some(cancel) = active.as_ref() {
            cancel.cancel();
        }
    }
    let placement = app.get_webview_window("main").and_then(|window| {
        let _ = window.set_title("GitFleet — Closing safely…");
        match window_state::capture(&window) {
            Ok(placement) => placement,
            Err(error) => {
                eprintln!("Window placement unavailable: {error}");
                None
            }
        }
    });
    start(app, host, placement);
}
fn start(app: tauri::AppHandle, host: Arc<Host>, placement: Option<WindowPlacement>) {
    let settings = app.get_webview_window("settings").map(|window| {
        let (token, receiver) = host
            .settings_close
            .begin(host.generation.load(std::sync::atomic::Ordering::Acquire))?;
        window
            .emit("gitfleet-settings-close-request", token)
            .map_err(|e| e.to_string())?;
        Ok::<_, String>(receiver)
    });
    let attempt = host
        .close_state
        .begin(host.generation.load(std::sync::atomic::Ordering::Acquire));
    #[cfg(feature = "native-acceptance-support")]
    let suppress_ack = native_acceptance::take_close_ack_fault();
    #[cfg(not(feature = "native-acceptance-support"))]
    let suppress_ack = false;
    if let Ok((token, _)) = &attempt {
        if !suppress_ack
            && app
                .emit_to("main", "gitfleet-close-request", token)
                .is_err()
        {
            eprintln!("Renderer close request could not be delivered");
        }
    }
    std::thread::spawn(move || {
        let submission = attempt
            .ok()
            .and_then(|(_, receiver)| receiver.recv_timeout(Duration::from_secs(3)).ok());
        host.close_state.clear();
        let settings_saved = match settings {
            None => true,
            Some(Ok(receiver)) => receiver
                .recv_timeout(Duration::from_secs(30))
                .unwrap_or(false),
            Some(Err(_)) => false,
        };
        host.settings_close.clear();
        let mut reply = None;
        let saved = if let Some(submission) = submission {
            reply = Some(submission.reply);
            let (sender, receiver) = mpsc::channel();
            host.sender
                .send(Task::SavePresentation {
                    snapshot: submission.snapshot,
                    reply: sender,
                })
                .is_ok()
                && receiver.recv().is_ok_and(|result| result.is_ok())
        } else {
            false
        };
        if (!saved || !settings_saved) && !discard_confirmed(&app) {
            let _ = app.emit_to("settings", "gitfleet-close-cancelled", ());
            host.closing
                .store(false, std::sync::atomic::Ordering::Release);
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.set_title("GitFleet");
            }
            if let Some(reply) = reply {
                let _ = reply.send(Err(
                    "Close cancelled: latest drafts were not saved; the app remains open".into(),
                ));
            }
            return;
        }
        if let Some(reply) = reply {
            let _ = reply.send(Ok(()));
        }
        if let Some(placement) = placement {
            if host
                .sender
                .send(Task::SaveWindowPlacement(placement))
                .is_err()
            {
                return;
            }
        }
        let _ = host.sender.send(Task::Shutdown);
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    fn empty() -> Snapshot {
        Snapshot {
            version: 1,
            epoch: 1,
            workspace: None,
            navigation: None,
            bulk: None,
        }
    }
    #[test]
    fn settings_flush_is_scoped_one_use_and_preserves_failure() {
        let (host, rx) = super::super::tests::host();
        host.closing
            .store(true, std::sync::atomic::Ordering::Release);
        let request = || {
            serde_json::from_value(json!({"kind":"save_ai_settings","fingerprint":"f","revision":"r","change":{"kind":"connection","id":null,"name":"Fixture","provider":"ollama","endpoint":"http://127.0.0.1:1/","timeout_ms":3000,"authentication":"none"}})).unwrap()
        };
        let task = |caller: &str| Task::Request {
            job: "fixture".into(),
            caller: caller.into(),
            epoch: 1,
            request: request(),
            cancel: Cancellation::default(),
        };
        assert!(queue(&host, task("settings")).is_err());
        for saved in [false, true] {
            let (token, done) = host.settings_close.begin(1).unwrap();
            assert!(queue(&host, task("main")).is_err());
            queue(&host, task("settings")).unwrap();
            assert!(matches!(rx.try_recv().unwrap(), Task::Request { .. }));
            assert!(host.settings_close.finish("forged", 1, true).is_err());
            assert!(host.settings_close.finish(&token, 2, true).is_err());
            host.settings_close.finish(&token, 1, saved).unwrap();
            assert_eq!(done.recv().unwrap(), saved);
            assert!(host.settings_close.finish(&token, 1, true).is_err());
            assert!(queue(&host, task("settings")).is_err());
        }
    }
    #[test]
    fn close_snapshot_requires_exact_token_generation_and_is_consumed_once() {
        let state = CloseState::default();
        let (token, receiver) = state.begin(2).unwrap();
        assert!(state.submit("forged", 2, empty()).is_err());
        assert!(state.submit(&token, 1, empty()).is_err());
        assert!(state.begin(2).is_err());
        let result = state.submit(&token, 2, empty()).unwrap();
        assert!(state.submit(&token, 2, empty()).is_err());
        receiver.recv().unwrap().reply.send(Ok(())).unwrap();
        assert!(result.recv().unwrap().is_ok());
        state.clear();
        assert!(state.begin(3).is_ok());
    }
    #[test]
    fn malformed_unknown_oversized_and_incomplete_snapshots_are_rejected() {
        assert!(Snapshot::decode(&serde_json::to_string(&empty()).unwrap()).is_ok());
        for invalid in ["{\"version\":2}", "{\"version\":1,\"epoch\":1,\"workspace\":\"w\",\"navigation\":null,\"bulk\":null}", "{\"version\":1,\"epoch\":1,\"workspace\":null,\"navigation\":null,\"bulk\":null,\"approved\":true}"] {
            assert!(Snapshot::decode(invalid).is_err());
        }
        assert!(Snapshot::decode(&" ".repeat(4 * 1024 * 1024 + 4097)).is_err());
    }
}
