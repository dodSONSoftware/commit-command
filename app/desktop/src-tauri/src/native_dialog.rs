//! End a native confirmation before destroying its parent, so rfd releases its lease.
use tauri::{Runtime, WebviewWindow};

pub(super) fn close_review<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(feature = "native-acceptance-support")]
    super::native_acceptance::record(
        "review_host_close_requested",
        serde_json::json!({"label":window.label()}),
    );
    let window = window.clone();
    let runner = window.clone();
    runner.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        match window.ns_window() {
            Ok(pointer) => {
                // SAFETY: Tauri provides this live NSWindow handle. The owned window
                // stays alive through this closure, which runs on the AppKit main thread.
                if let Some(parent) = unsafe { pointer.cast::<objc2_app_kit::NSWindow>().as_ref() }
                {
                    if let Some(sheet) = parent.attachedSheet() {
                        // Closing the parent alone does not resolve rfd's async future.
                        parent.endSheet_returnCode(&sheet, objc2_app_kit::NSModalResponseCancel);
                        sheet.orderOut(None);
                    }
                }
            }
            Err(error) => {
                eprintln!("Review confirmation sheet could not be inspected: {error}");
            }
        }
        if let Err(error) = window.close() {
            eprintln!("Review window could not be closed: {error}");
        }
    })
}
