use tauri::{Runtime, WebviewWindow};

/// The main renderer's modal guard blocks workspace input until this owned window closes.
pub(super) async fn present_settings<R: Runtime>(
    parent: &WebviewWindow<R>,
    settings: &WebviewWindow<R>,
) -> Result<(), String> {
    let (closed, receive) = std::sync::mpsc::channel();
    settings.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            let _ = closed.send(());
        }
    });
    settings
        .show()
        .and_then(|()| settings.set_focus())
        .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        receive
            .recv()
            .map_err(|_| "Settings close was interrupted".to_owned())
    })
    .await
    .map_err(|e| e.to_string())??;
    parent.set_focus().map_err(|e| e.to_string())
}

pub(super) fn approve_ai_source<R: Runtime>(
    parent: &WebviewWindow<R>,
    review: &gitfleet_core::ai::provider::Review,
) -> bool {
    let Some(source) = &review.source else {
        return false;
    };
    let origins = source
        .accounting
        .iter()
        .filter(|row| row.included_lines > 0)
        .map(|row| row.origin.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut files = origins
        .iter()
        .take(6)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if origins.len() > 6 {
        files.push_str(&format!(" (+{} more)", origins.len() - 6));
    }
    let message = format!("Send the reviewed repository context to {}?\n\nModel: {}\nIncluded sources: {}\nRequest size: {} bytes",
        review.route, review.model.as_deref().unwrap_or("unknown"), files, review.request_bytes);
    rfd::MessageDialog::new()
        .set_parent(parent)
        .set_title("Send repository source?")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            "Send source".into(),
            "Cancel".into(),
        ))
        .show()
        == rfd::MessageDialogResult::Custom("Send source".into())
}
