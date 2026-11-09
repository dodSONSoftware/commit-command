//! Native-only secret input. The webview supplies a saved connection ID, never a token.
use super::{check, lock, queue, DialogLease, Host, Task};
use gitfleet_core::ai::{
    credentials::{Binding, SecretStore},
    provider::Secret,
};
use std::{
    sync::{atomic::Ordering, mpsc, Arc},
    time::Duration,
};
use tauri::{Runtime, WebviewWindow};
enum Answer {
    Save(Vec<u8>),
    Remove,
    Cancel,
}
fn begin<R: Runtime>(
    window: &WebviewWindow<R>,
    binding: &Binding,
    reply: mpsc::Sender<Answer>,
) -> Result<(), String> {
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSAlert, NSSecureTextField};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    let runner = window.clone();
    let window = window.clone();
    let binding = binding.clone();
    let description = format!(
        "Saved connection: {}\nExact endpoint: {}\n\nEnter an existing provider token. It stays in macOS Keychain and is never returned to the webview or exported. Saving does not enable authentication or contact the provider. Unsaved endpoint edits are not used here. Locked or authorization-required access fails without opening background OS prompts.",
        binding.connection, binding.endpoint
    );
    runner
        .run_on_main_thread(move || {
            let Some(mtm) = MainThreadMarker::new() else {
                let _ = reply.send(Answer::Cancel);
                return;
            };
            let Ok(pointer) = window.ns_window() else {
                let _ = reply.send(Answer::Cancel);
                return;
            };
            // SAFETY: Tauri owns this live window and this closure runs on AppKit's
            // main thread, without interleaving destruction while the pointer is used.
            let parent = unsafe { pointer.cast::<objc2_app_kit::NSWindow>().as_ref() };
            let Some(parent) = parent else {
                let _ = reply.send(Answer::Cancel);
                return;
            };
            let alert = NSAlert::new(mtm);
            alert.setMessageText(&NSString::from_str("Commit Command — AI bearer token"));
            alert.setInformativeText(&NSString::from_str(&description));
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(420.0, 28.0));
            let field = NSSecureTextField::initWithFrame(NSSecureTextField::alloc(mtm), frame);
            field.setPlaceholderString(Some(&NSString::from_str("Existing provider bearer token")));
            alert.setAccessoryView(Some(&field));
            alert.window().setInitialFirstResponder(Some(&field));
            alert.addButtonWithTitle(&NSString::from_str("Store token"));
            let cancel = alert.addButtonWithTitle(&NSString::from_str("Cancel"));
            cancel.setKeyEquivalent(&NSString::from_str("\u{1b}"));
            alert.addButtonWithTitle(&NSString::from_str("Remove stored token…"));
            // AppKit retains the completion block; retain its alert/field until completion.
            let keep = alert.clone();
            let handler = block2::RcBlock::new(move |response: isize| {
                let _ = &keep;
                let answer = match response {
                    1000 => Answer::Save(field.stringValue().to_string().into_bytes()),
                    1002 => Answer::Remove,
                    _ => Answer::Cancel,
                };
                field.setStringValue(&NSString::from_str(""));
                let _ = reply.send(answer);
            });
            alert.beginSheetModalForWindow_completionHandler(parent, Some(&handler));
        })
        .map_err(|_| "Native AI token field unavailable".to_owned())
}
fn dismiss<R: Runtime>(window: &WebviewWindow<R>) {
    let runner = window.clone();
    let window = window.clone();
    let _ = runner.run_on_main_thread(move || {
        if let Ok(pointer) = window.ns_window() {
            // SAFETY: Live owned main window, on the AppKit main thread.
            if let Some(parent) = unsafe { pointer.cast::<objc2_app_kit::NSWindow>().as_ref() } {
                if let Some(sheet) = parent.attachedSheet() {
                    parent.endSheet_returnCode(&sheet, objc2_app_kit::NSModalResponseCancel);
                    sheet.orderOut(None);
                }
            }
        }
    });
}
async fn binding(state: &Arc<Host>, connection: String) -> Result<Binding, String> {
    let (reply, rx) = mpsc::channel();
    queue(state, Task::AiCredentialBinding { connection, reply })?;
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|_| "AI connection unavailable".to_owned())?
    })
    .await
    .map_err(|_| "AI connection unavailable".to_owned())?
}
#[tauri::command]
pub(super) async fn manage_ai_token<R: Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    connection: String,
) -> Result<(), String> {
    check(&window, "main")?;
    if connection.len() > 80 || lock(&state.active)?.is_some() {
        return Err("Finish the active operation before managing an AI token".into());
    }
    let _lease = DialogLease::new(state.inner().clone())?;
    let generation = state.generation.load(Ordering::Acquire);
    let current = binding(state.inner(), connection.clone()).await?;
    let (reply, rx) = mpsc::channel();
    begin(&window, &current, reply)?;
    let host = state.inner().clone();
    let main = window.clone();
    let answer = tauri::async_runtime::spawn_blocking(move || loop {
        if host.generation.load(Ordering::Acquire) != generation
            || host.closing.load(Ordering::Acquire)
        {
            dismiss(&main);
            return Err("AI token operation cancelled by application context change".to_owned());
        }
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(answer) => return Ok(answer),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err("AI token entry cancelled".to_owned()),
        }
    })
    .await
    .map_err(|_| "AI token entry unavailable".to_owned())??;
    if matches!(answer, Answer::Cancel) {
        return Ok(());
    }
    if generation != state.generation.load(Ordering::Acquire)
        || current != binding(state.inner(), connection).await?
    {
        return Err("AI connection changed; no credential was modified".into());
    }
    match answer {
        Answer::Save(bytes) => {
            let secret = Secret::new(bytes).map_err(|e| e.to_string())?;
            tauri::async_runtime::spawn_blocking(move || {
                super::ai_credentials::KeychainStore::default()
                    .set(&current, &secret)
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(|_| "AI secure credential store unavailable".to_owned())?
        }
        Answer::Remove => {
            let accepted=rfd::AsyncMessageDialog::new().set_parent(&window).set_title("Remove this AI token?").set_description(format!("Remove only the stored bearer token for {} at {}? The app cannot recover it; the provider's token is not revoked. No repository or other credential is changed.",current.connection,current.endpoint)).set_buttons(rfd::MessageButtons::OkCancelCustom("Remove stored token".into(),"Cancel".into())).show().await==rfd::MessageDialogResult::Custom("Remove stored token".into());
            if !accepted {
                return Ok(());
            }
            if generation != state.generation.load(Ordering::Acquire) {
                return Err("AI token removal cancelled by context change".into());
            }
            tauri::async_runtime::spawn_blocking(move || {
                super::ai_credentials::KeychainStore::default()
                    .delete(&current)
                    .map_err(|e| e.to_string())
            })
            .await
            .map_err(|_| "AI secure credential store unavailable".to_owned())?
        }
        Answer::Cancel => Ok(()),
    }
}
