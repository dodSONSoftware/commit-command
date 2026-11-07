//! Fixed fixture action delivered outside Tauri callbacks, including modal run-loop modes.
use super::record;
use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObject, NSObjectProtocol},
    sel, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{NSApplication, NSEventTrackingRunLoopMode, NSModalPanelRunLoopMode};
use objc2_foundation::{NSArray, NSDefaultRunLoopMode, NSRunLoop};
use serde_json::json;

define_class!(
    // SAFETY: NSObject has no extra subclass requirements; no Rust ivars or Drop.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct AcceptanceTerminationRequest;

    // SAFETY: No additional NSObjectProtocol requirements.
    unsafe impl NSObjectProtocol for AcceptanceTerminationRequest {}

    impl AcceptanceTerminationRequest {
        #[unsafe(method(fire:))]
        fn fire(&self, _sender: Option<&AnyObject>) {
            let application = NSApplication::sharedApplication(self.mtm());
            let attached_sheet = application.windows().iter().any(|window| window.attachedSheet().is_some());
            let mode = NSRunLoop::currentRunLoop().currentMode().map(|mode| mode.to_string());
            record("appkit_terminate_dispatched", json!({"mode":mode,"attached_sheet":attached_sheet}));
            application.terminate(None);
            record("appkit_terminate_returned", json!({}));
        }
    }
);

pub(super) fn schedule() {
    let Some(main_thread) = MainThreadMarker::new() else {
        record("appkit_terminate_unavailable", json!({}));
        return;
    };
    let request = AcceptanceTerminationRequest::alloc(main_thread);
    // SAFETY: NSObject init initializes this allocated, ivar-free main-thread object.
    let request: Retained<AcceptanceTerminationRequest> = unsafe { msg_send![request, init] };
    // SAFETY: These are framework-owned, immutable run-loop mode constants. The
    // fixed fire: selector accepts a nil sender. Foundation retains this receiver
    // until delivery; delay zero defers it beyond Tauri's locked event callback.
    unsafe {
        let modes = NSArray::from_slice(&[
            NSDefaultRunLoopMode,
            NSModalPanelRunLoopMode,
            NSEventTrackingRunLoopMode,
        ]);
        record(
            "appkit_terminate_requested",
            json!({"modes":"default,modal,event_tracking"}),
        );
        let _: () = msg_send![&*request,
            performSelector: sel!(fire:),
            withObject: std::ptr::null::<AnyObject>(),
            afterDelay: 0_f64,
            inModes: &*modes
        ];
    }
}
