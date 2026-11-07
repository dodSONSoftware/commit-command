//! Preserve Tao's application delegate while routing native termination through shutdown.
use super::{shutdown, Host};
use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, NSObjectProtocol, ProtocolObject, Sel},
    DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{NSApplication, NSApplicationDelegate, NSApplicationTerminateReply};
use std::{
    cell::RefCell,
    sync::{atomic::Ordering, Arc},
};

struct DelegateState {
    original: Retained<ProtocolObject<dyn NSApplicationDelegate>>,
    app: tauri::AppHandle,
    host: Arc<Host>,
}

define_class!(
    // SAFETY: NSObject has no additional subclassing requirements; this object
    // and its retained delegate are created/used only on the AppKit main thread.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateState]
    struct TerminationDelegate;

    // SAFETY: All implemented selectors use their declared Objective-C signatures.
    unsafe impl NSObjectProtocol for TerminationDelegate {
        #[unsafe(method(respondsToSelector:))]
        fn responds_to_selector(&self, selector: Sel) -> bool {
            // Retain NSObject/our own methods and advertise the wrapped delegate's
            // optional methods so AppKit continues delivering Tao's normal callbacks.
            let own: bool = unsafe { msg_send![super(self), respondsToSelector: selector] };
            own || self.ivars().original.respondsToSelector(selector)
        }
    }

    impl TerminationDelegate {
        #[unsafe(method_id(forwardingTargetForSelector:))]
        fn forwarding_target(&self, _selector: Sel) -> Option<Retained<ProtocolObject<dyn NSApplicationDelegate>>> {
            Some(self.ivars().original.clone())
        }
    }

    // SAFETY: The return type matches AppKit's NSUInteger-backed termination reply.
    unsafe impl NSApplicationDelegate for TerminationDelegate {
        #[unsafe(method(applicationShouldTerminate:))]
        fn should_terminate(&self, _sender: &NSApplication) -> NSApplicationTerminateReply {
            if self.ivars().host.shutdown_complete.load(Ordering::Acquire) {
                return NSApplicationTerminateReply::TerminateNow;
            }
            #[cfg(feature = "native-acceptance-support")]
            super::native_acceptance::record("appkit_termination_intercepted", serde_json::json!({}));
            // Cancel this native request. The existing handshake can keep the app
            // open on save failure, or exit normally after Core has shut down.
            shutdown::request(self.ivars().app.clone(), self.ivars().host.clone());
            NSApplicationTerminateReply::TerminateCancel
        }
    }
);

thread_local! {
    // NSApplication's delegate property is weak. Keep this wrapper alive without
    // making a main-thread-only Objective-C object Send/Sync or leaking it.
    static DELEGATE: RefCell<Option<Retained<TerminationDelegate>>> = const { RefCell::new(None) };
}

pub(super) fn install(app: tauri::AppHandle, host: Arc<Host>) -> Result<(), String> {
    let main_thread =
        MainThreadMarker::new().ok_or("Termination delegate requires AppKit main thread")?;
    let application = NSApplication::sharedApplication(main_thread);
    let original = application
        .delegate()
        .ok_or("Native application delegate is unavailable")?;
    let wrapper = TerminationDelegate::alloc(main_thread).set_ivars(DelegateState {
        original,
        app,
        host,
    });
    // SAFETY: NSObject's init establishes the allocated wrapper; ivars are initialized.
    let wrapper: Retained<TerminationDelegate> = unsafe { msg_send![super(wrapper), init] };
    application.setDelegate(Some(ProtocolObject::from_ref(&*wrapper)));
    DELEGATE.with(|slot| *slot.borrow_mut() = Some(wrapper));
    Ok(())
}

#[cfg(feature = "native-acceptance-support")]
pub(super) fn probe_forwarding() {
    DELEGATE.with(|slot| {
        let slot = slot.borrow();
        let Some(wrapper) = slot.as_ref() else { return };
        let selector = objc2::sel!(applicationSupportsSecureRestorableState:);
        let original_responds = wrapper.ivars().original.respondsToSelector(selector);
        let wrapped_responds = wrapper.respondsToSelector(selector);
        if !original_responds || !wrapped_responds {
            super::native_acceptance::record("delegate_forward_probe", serde_json::json!({"passed":false}));
            return;
        }
        let app = NSApplication::sharedApplication(wrapper.mtm());
        // SAFETY: Both objects advertise Tao's pinned optional delegate selector,
        // whose signature returns BOOL and takes the current NSApplication.
        let original: bool = unsafe { msg_send![&*wrapper.ivars().original, applicationSupportsSecureRestorableState: &*app] };
        let forwarded: bool = unsafe { msg_send![&**wrapper, applicationSupportsSecureRestorableState: &*app] };
        super::native_acceptance::record("delegate_forward_probe", serde_json::json!({"passed":original==forwarded,"original":original,"forwarded":forwarded}));
    });
}
