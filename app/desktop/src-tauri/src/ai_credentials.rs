//! Endpoint-bound native Keychain storage. No global search-list/UI policy changes.
use core_foundation::{
    base::{CFType, TCFType},
    boolean::CFBoolean,
    data::CFData,
    dictionary::CFDictionary,
    string::{CFString, CFStringRef},
};
use gitfleet_core::{
    ai::{
        credentials::{Binding, SecretStore},
        provider::Secret,
    },
    Error, ErrorKind, Result,
};
use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject},
};
use security_framework_sys::{
    item::*,
    keychain_item::{SecItemAdd, SecItemCopyMatching, SecItemDelete, SecItemUpdate},
};
const SERVICE: &str = "dev.gitfleet.desktop.ai.bearer.v1";
type Pairs = Vec<(CFString, CFType)>;
#[link(name = "LocalAuthentication", kind = "framework")]
extern "C" {}

#[derive(Default)]
struct RawKeychain {
    #[cfg(test)]
    keychain: Option<std::path::PathBuf>,
}
fn unavailable() -> Error {
    Error::with_kind(
        ErrorKind::Unavailable,
        "AI secure credential store unavailable; manual workflows remain available",
    )
}
fn status(code: i32) -> Result<()> {
    match code {
        0=>Ok(()),
        -25300=>Err(Error::with_kind(ErrorKind::Unavailable,"No AI bearer token for this exact connection and endpoint; store one explicitly or select no authentication")),
        -25308|-25293=>Err(Error::with_kind(ErrorKind::Unavailable,"AI Keychain is locked or requires authorization; unlock or review access externally, then retry. No unauthenticated fallback")),
        -128=>Err(Error::with_kind(ErrorKind::Cancelled,"AI Keychain access cancelled")),
        _=>Err(Error::with_kind(ErrorKind::Unavailable,format!("AI secure credential operation failed (OS status {code}); no fallback selected"))),
    }
}
fn key(raw: CFStringRef) -> CFString {
    // SAFETY: Callers supply only immutable Security.framework string constants.
    unsafe { CFString::wrap_under_get_rule(raw) }
}
fn context() -> Result<CFType> {
    let class = AnyClass::get(c"LAContext").ok_or_else(unavailable)?;
    // SAFETY: LAContext is supplied by the linked OS framework. Its documented
    // interactionNotAllowed property is set before handing it to Keychain APIs.
    let context: Retained<AnyObject> = unsafe { msg_send![class, new] };
    unsafe {
        let _: () = msg_send![&*context,setInteractionNotAllowed:true];
    }
    // LAContext is accepted as a retained object value for kSecUseAuthenticationContext.
    Ok(unsafe { CFType::wrap_under_get_rule(Retained::as_ptr(&context).cast()) })
}
impl RawKeychain {
    fn query(&self, binding: &Binding, adding: bool) -> Result<Pairs> {
        let account = binding.account()?;
        // SAFETY: All keys/value constants below are immutable framework globals.
        let pairs = unsafe {
            vec![
                (key(kSecClass), key(kSecClassGenericPassword).as_CFType()),
                (key(kSecAttrService), CFString::new(SERVICE).as_CFType()),
                (key(kSecAttrAccount), CFString::new(&account).as_CFType()),
                (
                    key(kSecAttrSynchronizable),
                    CFBoolean::false_value().as_CFType(),
                ),
                (key(kSecUseAuthenticationContext), context()?),
            ]
        };
        #[cfg(test)]
        let mut pairs = pairs;
        #[cfg(test)]
        if let Some(path) = &self.keychain {
            use core_foundation::array::CFArray;
            let keychain = security_framework::os::macos::keychain::SecKeychain::open(path)
                .map_err(|_| unavailable())?;
            // A private fixture Keychain is never added to the global search list.
            unsafe {
                if adding {
                    pairs.push((key(kSecUseKeychain), keychain.as_CFType()));
                } else {
                    pairs.push((
                        key(kSecMatchSearchList),
                        CFArray::from_CFTypes(&[keychain]).as_CFType(),
                    ));
                }
            }
        }
        #[cfg(not(test))]
        let _ = adding;
        Ok(pairs)
    }
}
impl SecretStore for RawKeychain {
    fn get(
        &self,
        binding: &Binding,
        cancellation: gitfleet_core::inspection::Cancellation,
    ) -> Result<Secret> {
        if cancellation.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "AI Keychain lookup cancelled",
            ));
        }
        let mut pairs = self.query(binding, false)?;
        // SAFETY: Immutable framework key and an owned boolean value.
        pairs.push((
            key(unsafe { kSecReturnData }),
            CFBoolean::true_value().as_CFType(),
        ));
        let query = CFDictionary::from_CFType_pairs(&pairs);
        let mut raw = std::ptr::null();
        // SAFETY: Query owns every CF object for this synchronous call. Successful
        // CopyMatching returns an owned CF object, checked below before reading.
        status(unsafe { SecItemCopyMatching(query.as_concrete_TypeRef(), &mut raw) })?;
        if raw.is_null() {
            return Err(unavailable());
        }
        let object = unsafe { CFType::wrap_under_create_rule(raw) };
        let data = object.downcast::<CFData>().ok_or_else(unavailable)?;
        if data.bytes().len() > 4096 {
            return Err(Error::with_kind(
                ErrorKind::InvalidRequest,
                "Stored AI token exceeds its supported bound",
            ));
        }
        Secret::new(data.bytes().to_vec())
    }
    fn set(&self, binding: &Binding, secret: &Secret) -> Result<()> {
        let query = CFDictionary::from_CFType_pairs(&self.query(binding, false)?);
        let value = CFData::from_buffer(secret.expose());
        let attributes =
            CFDictionary::from_CFType_pairs(&[(key(unsafe { kSecValueData }), value.as_CFType())]);
        // SAFETY: Both dictionaries keep all query/data values alive through calls.
        let updated = unsafe {
            SecItemUpdate(
                query.as_concrete_TypeRef(),
                attributes.as_concrete_TypeRef(),
            )
        };
        if updated == -25300 {
            let mut pairs = self.query(binding, true)?;
            pairs.push((key(unsafe { kSecValueData }), value.as_CFType()));
            let added = CFDictionary::from_CFType_pairs(&pairs);
            status(unsafe { SecItemAdd(added.as_concrete_TypeRef(), std::ptr::null_mut()) })
        } else {
            status(updated)
        }
    }
    fn delete(&self, binding: &Binding) -> Result<()> {
        let query = CFDictionary::from_CFType_pairs(&self.query(binding, false)?);
        // SAFETY: Exact class/service/account/endpoint binding, with UI disabled.
        let removed = unsafe { SecItemDelete(query.as_concrete_TypeRef()) };
        if removed == -25300 {
            Ok(())
        } else {
            status(removed)
        }
    }
}
struct Helpers {
    closing: bool,
    running: std::collections::BTreeMap<String, gitfleet_core::inspection::Cancellation>,
}
static HELPERS: std::sync::Mutex<Helpers> = std::sync::Mutex::new(Helpers {
    closing: false,
    running: std::collections::BTreeMap::new(),
});
struct HelperLease {
    id: String,
    cancel: gitfleet_core::inspection::Cancellation,
}
impl HelperLease {
    fn new(cancel: gitfleet_core::inspection::Cancellation) -> Result<Self> {
        let mut helpers = HELPERS.lock().map_err(|_| unavailable())?;
        if helpers.closing {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "AI credential operation cancelled by shutdown",
            ));
        }
        if helpers.running.len() >= 32 {
            return Err(Error::with_kind(
                ErrorKind::Busy,
                "AI credential helper limit reached",
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        helpers.running.insert(id.clone(), cancel.clone());
        Ok(Self { id, cancel })
    }
}
impl Drop for HelperLease {
    fn drop(&mut self) {
        if let Ok(mut helpers) = HELPERS.lock() {
            helpers.running.remove(&self.id);
        }
    }
}
pub(super) fn shutdown_helpers() -> Result<()> {
    {
        let mut helpers = HELPERS.lock().map_err(|_| unavailable())?;
        helpers.closing = true;
        for cancel in helpers.running.values() {
            cancel.cancel();
        }
    }
    let start = std::time::Instant::now();
    loop {
        if HELPERS
            .lock()
            .map_err(|_| unavailable())?
            .running
            .is_empty()
        {
            return Ok(());
        }
        if start.elapsed() > std::time::Duration::from_secs(6) {
            return Err(Error::with_kind(
                ErrorKind::Timeout,
                "AI credential helper cleanup did not complete",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[derive(Default)]
pub(super) struct KeychainStore {
    #[cfg(test)]
    keychain: Option<std::path::PathBuf>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Get,
    Set {
        bytes: Vec<u8>,
    },
    Delete,
    #[cfg(test)]
    Hold,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerRequest {
    version: u32,
    binding: Binding,
    operation: Operation,
    #[cfg(test)]
    keychain: Option<std::path::PathBuf>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerResponse {
    bytes: Option<Vec<u8>>,
    error: Option<(ErrorKind, String)>,
}
const FRAME: &[u8] = b"GITFLEET_AI_CREDENTIAL_V1\n";
#[link(name = "proc")]
extern "C" {
    fn proc_pidpath(pid: i32, buffer: *mut std::ffi::c_void, size: u32) -> i32;
}
extern "C" {
    fn SecCodeCopySigningInformation(
        code: security_framework_sys::code_signing::SecStaticCodeRef,
        flags: u32,
        information: *mut core_foundation::dictionary::CFDictionaryRef,
    ) -> i32;
    static kSecCodeInfoUnique: CFStringRef;
}
fn code_hash(code: security_framework_sys::code_signing::SecCodeRef) -> Option<Vec<u8>> {
    use security_framework_sys::code_signing::{kSecCSNoNetworkAccess, SecCodeCheckValidity};
    // Validate the live process image before reading its CodeDirectory identity;
    // comparing filesystem paths alone would permit a renamed-executable spoof.
    if unsafe { SecCodeCheckValidity(code, kSecCSNoNetworkAccess, std::ptr::null_mut()) } != 0 {
        return None;
    }
    let mut raw = std::ptr::null();
    if unsafe { SecCodeCopySigningInformation(code.cast(), 0, &mut raw) } != 0 || raw.is_null() {
        return None;
    }
    let information = unsafe { CFDictionary::<CFString, CFType>::wrap_under_create_rule(raw) };
    let value = information.find(key(unsafe { kSecCodeInfoUnique }))?;
    let data = value.downcast::<CFData>()?;
    Some(data.bytes().to_vec())
}
fn same_live_code(parent: i32) -> bool {
    use security_framework_sys::code_signing::{
        kSecGuestAttributePid, SecCodeCopyGuestWithAttributes, SecCodeCopySelf,
    };
    let attributes = CFDictionary::from_CFType_pairs(&[(
        key(unsafe { kSecGuestAttributePid }),
        core_foundation::number::CFNumber::from(parent).as_CFType(),
    )]);
    let mut guest = std::ptr::null_mut();
    let mut own = std::ptr::null_mut();
    if unsafe {
        SecCodeCopyGuestWithAttributes(
            std::ptr::null_mut(),
            attributes.as_concrete_TypeRef(),
            0,
            &mut guest,
        )
    } != 0
        || guest.is_null()
    {
        return false;
    }
    let guest_owner = unsafe { CFType::wrap_under_create_rule(guest.cast()) };
    if unsafe { SecCodeCopySelf(0, &mut own) } != 0 || own.is_null() {
        return false;
    }
    let own_owner = unsafe { CFType::wrap_under_create_rule(own.cast()) };
    let matches = match (code_hash(guest), code_hash(own)) {
        (Some(a), Some(b)) => !a.is_empty() && a == b,
        _ => false,
    };
    drop((guest_owner, own_owner));
    matches
}
fn own_parent() -> bool {
    use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
    let mut path = [0u8; 4096];
    // SAFETY: proc_pidpath receives a valid fixed-size writable buffer. Compare
    // kernel-reported executable identity, never argv or an environment claim.
    let length = unsafe {
        proc_pidpath(
            std::process::id() as i32,
            path.as_mut_ptr().cast(),
            path.len() as u32,
        )
    };
    if length <= 0 || length as usize > path.len() {
        return false;
    }
    let own = std::path::Path::new(std::ffi::OsStr::from_bytes(
        &path[..path.iter().position(|b| *b == 0).unwrap_or(length as usize)],
    ));
    let Ok(own) = std::fs::metadata(own) else {
        return false;
    };
    let parent = unsafe { getppid() };
    if parent <= 1 {
        return false;
    }
    let length = unsafe { proc_pidpath(parent, path.as_mut_ptr().cast(), path.len() as u32) };
    if length <= 0 || length as usize > path.len() {
        return false;
    }
    let parent = std::path::Path::new(std::ffi::OsStr::from_bytes(
        &path[..path.iter().position(|b| *b == 0).unwrap_or(length as usize)],
    ));
    std::fs::metadata(parent).is_ok_and(|p| p.dev() == own.dev() && p.ino() == own.ino())
        && same_live_code(unsafe { getppid() })
}
extern "C" {
    fn getppid() -> i32;
}
fn serve_worker() -> i32 {
    use std::io::{Read, Write};
    if !own_parent() {
        return 2;
    }
    // This policy belongs only to the one-shot helper. It never changes global
    // user settings or the main application's interaction policy.
    if unsafe { security_framework_sys::keychain::SecKeychainSetUserInteractionAllowed(0) } != 0 {
        return 2;
    }
    let mut input = Vec::new();
    if std::io::stdin()
        .take(32769)
        .read_to_end(&mut input)
        .is_err()
        || input.len() > 32768
    {
        return 2;
    }
    let Ok(request) = serde_json::from_slice::<WorkerRequest>(&input) else {
        return 2;
    };
    input.fill(0);
    if request.version != 1 || request.binding.account().is_err() {
        return 2;
    }
    #[cfg(test)]
    if let Some(path) = &request.keychain {
        let Some(parent) = path.parent() else {
            return 2;
        };
        let root = parent
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !root.starts_with("gitfleet-ai-keychain-")
            || parent.parent() != Some(std::path::Path::new("/private/tmp"))
            || path.file_name() != Some(std::ffi::OsStr::new("fixture.keychain"))
        {
            return 2;
        }
    }
    let store = RawKeychain {
        #[cfg(test)]
        keychain: request.keychain,
    };
    let result = match request.operation {
        Operation::Get => store
            .get(&request.binding, Default::default())
            .map(|s| Some(s.expose().to_vec())),
        Operation::Set { bytes } => Secret::new(bytes)
            .and_then(|s| store.set(&request.binding, &s))
            .map(|_| None),
        Operation::Delete => store.delete(&request.binding).map(|_| None),
        #[cfg(test)]
        Operation::Hold => {
            let Some(path) = store.keychain.as_ref().and_then(|p| p.parent()) else {
                return 2;
            };
            let marker = path.join("worker-started");
            let Ok(mut file) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(marker)
            else {
                return 2;
            };
            if write!(file, "{}", std::process::id()).is_err() || file.sync_all().is_err() {
                return 2;
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
            Ok(None)
        }
    };
    let response = match result {
        Ok(bytes) => WorkerResponse { bytes, error: None },
        Err(e) => WorkerResponse {
            bytes: None,
            error: Some((e.kind, e.to_string())),
        },
    };
    let Ok(mut body) = serde_json::to_vec(&response) else {
        return 2;
    };
    let mut output = std::io::stdout().lock();
    let result = output
        .write_all(FRAME)
        .and_then(|_| output.write_all(&body))
        .and_then(|_| output.flush());
    body.fill(0);
    if result.is_ok() {
        0
    } else {
        2
    }
}
pub(super) fn maybe_worker() {
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("--gitfleet-ai-keychain-worker"))
    {
        std::process::exit(serve_worker());
    }
}
impl KeychainStore {
    fn invoke(&self, binding: &Binding, operation: Operation) -> Result<Option<Vec<u8>>> {
        self.invoke_cancellable(binding, operation, Default::default())
    }
    fn invoke_cancellable(
        &self,
        binding: &Binding,
        operation: Operation,
        cancellation: gitfleet_core::inspection::Cancellation,
    ) -> Result<Option<Vec<u8>>> {
        binding.account()?;
        let read = match &operation {
            Operation::Get => true,
            #[cfg(test)]
            Operation::Hold => true,
            _ => false,
        };
        let request = WorkerRequest {
            version: 1,
            binding: binding.clone(),
            operation,
            #[cfg(test)]
            keychain: self.keychain.clone(),
        };
        let mut command =
            std::process::Command::new(std::env::current_exe().map_err(|_| unavailable())?);
        command.env_clear().env("LC_ALL", "C").current_dir("/");
        #[cfg(not(test))]
        command.arg("--gitfleet-ai-keychain-worker");
        #[cfg(test)]
        command.args([
            "--ignored",
            "--exact",
            "ai_credentials::tests::credential_worker_entry",
            "--nocapture",
        ]);
        let timeout = std::time::Duration::from_secs(5);
        let lease = HelperLease::new(cancellation)?;
        let mut output = gitfleet_core::ai::credentials::run_native_worker(command,serde_json::to_vec(&request)?,timeout,lease.cancel.clone()).map_err(|e| {
            if read { Error::with_kind(e.kind,format!("{e}; no provider request sent")) }
            else { Error::with_kind(ErrorKind::UncertainOutcome,"AI Keychain change was not confirmed and may have completed; explicitly review or retry this exact connection before using it") }
        })?;
        #[cfg(not(test))]
        let body = output.strip_prefix(FRAME).ok_or_else(unavailable)?;
        // The Rust test harness writes a short prelude before entering its worker.
        #[cfg(test)]
        let body = {
            let at = output
                .windows(FRAME.len())
                .position(|w| w == FRAME)
                .ok_or_else(unavailable)?;
            &output[at + FRAME.len()..]
        };
        let response: WorkerResponse = serde_json::from_slice(body).map_err(|_| unavailable())?;
        output.fill(0);
        if let Some((kind, message)) = response.error {
            return Err(Error::with_kind(kind, message));
        }
        Ok(response.bytes)
    }
}
impl SecretStore for KeychainStore {
    fn get(
        &self,
        binding: &Binding,
        cancellation: gitfleet_core::inspection::Cancellation,
    ) -> Result<Secret> {
        Secret::new(
            self.invoke_cancellable(binding, Operation::Get, cancellation)?
                .ok_or_else(unavailable)?,
        )
    }
    fn set(&self, binding: &Binding, secret: &Secret) -> Result<()> {
        if self
            .invoke(
                binding,
                Operation::Set {
                    bytes: secret.expose().to_vec(),
                },
            )?
            .is_some()
        {
            return Err(unavailable());
        }
        Ok(())
    }
    fn delete(&self, binding: &Binding) -> Result<()> {
        if self.invoke(binding, Operation::Delete)?.is_some() {
            return Err(unavailable());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    // Security.framework serializes parts of code validation and Keychain access.
    // Keep unrelated OS fixtures from consuming a proven-running fault deadline.
    static OS_FIXTURE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    #[ignore = "Owned subprocess worker, explicitly invoked by the parent test"]
    fn credential_worker_entry() {
        std::process::exit(serve_worker());
    }

    #[test]
    fn credential_worker_requires_the_same_live_code_image() {
        let _fixture = OS_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
        assert!(same_live_code(std::process::id() as i32));
        assert!(!same_live_code(unsafe { getppid() }));
        let executable = std::env::current_exe().unwrap();
        let output = std::process::Command::new("/bin/sh")
            .args([
                "-c",
                "\"$@\"; task_worker_code=$?; exit \"$task_worker_code\"",
                "untrusted-parent",
            ])
            .arg(executable)
            .args([
                "--ignored",
                "--exact",
                "ai_credentials::tests::credential_worker_entry",
                "--nocapture",
            ])
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!output.stdout.windows(FRAME.len()).any(|w| w == FRAME));
    }
    #[test]
    fn proven_running_credential_helper_timeout_reaps_only_the_owned_child() {
        let _fixture = OS_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
        let root = std::path::PathBuf::from("/private/tmp")
            .join(format!("gitfleet-ai-keychain-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let store = KeychainStore {
            keychain: Some(root.join("fixture.keychain")),
        };
        let binding = Binding::new(
            &uuid::Uuid::new_v4().to_string(),
            "https://fixture.invalid/",
        )
        .unwrap();
        let start = std::time::Instant::now();
        let error = store.invoke(&binding, Operation::Hold).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert!(start.elapsed() < std::time::Duration::from_secs(6));
        let pid = std::fs::read_to_string(root.join("worker-started"))
            .unwrap()
            .parse::<i32>()
            .unwrap();
        extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        assert_eq!(unsafe { kill(pid, 0) }, -1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(3));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn binding_is_canonical_endpoint_specific_and_contains_no_secret() {
        let id = uuid::Uuid::new_v4().to_string();
        let a = Binding::new(&id, "http://127.0.0.1:11434").unwrap();
        let b = Binding::new(&id, "http://127.0.0.1:11435/").unwrap();
        assert_ne!(a.account().unwrap(), b.account().unwrap());
        assert_eq!(a, Binding::new(&id, "http://127.0.0.1:11434/").unwrap());
        assert!(!a.account().unwrap().contains("127.0.0.1"));
    }
    #[test]
    fn proven_running_credential_helper_cancellation_reaps_the_owned_child() {
        let _fixture = OS_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
        let root = std::path::PathBuf::from("/private/tmp")
            .join(format!("gitfleet-ai-keychain-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let marker = root.join("worker-started");
        let store = KeychainStore {
            keychain: Some(root.join("fixture.keychain")),
        };
        let binding = Binding::new(
            &uuid::Uuid::new_v4().to_string(),
            "https://fixture.invalid/",
        )
        .unwrap();
        let cancel = gitfleet_core::inspection::Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker = std::thread::spawn(move || {
            store.invoke_cancellable(&binding, Operation::Hold, worker_cancel)
        });
        let start = std::time::Instant::now();
        while !marker.exists() {
            assert!(start.elapsed() < std::time::Duration::from_secs(5));
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let pid = loop {
            if let Ok(text) = std::fs::read_to_string(&marker) {
                if let Ok(pid) = text.parse::<i32>() {
                    break pid;
                }
            }
            assert!(start.elapsed() < std::time::Duration::from_secs(5));
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        let cancelled = std::time::Instant::now();
        cancel.cancel();
        let error = worker.join().unwrap().unwrap_err();
        assert_eq!(error.kind, ErrorKind::Cancelled);
        assert!(cancelled.elapsed() < std::time::Duration::from_secs(2));
        extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        assert_eq!(unsafe { kill(pid, 0) }, -1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(3));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn actual_keychain_backed_diagnostic_requires_approval_and_keeps_token_out_of_results() {
        let _fixture = OS_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
        use gitfleet_core::{
            ai::{
                change::Change,
                config::{ConnectionDraft, Provider},
                credentials::Authentication,
            },
            application::{Application, Data, Request},
            inspection::Cancellation,
            writes::Options,
        };
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::Arc,
            time::{Duration, Instant},
        };
        let root = std::path::PathBuf::from("/private/tmp")
            .join(format!("gitfleet-ai-keychain-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let _keychain = security_framework::os::macos::keychain::CreateOptions::new()
            .password("disposable-keychain-only")
            .prompt_user(false)
            .create(root.join("fixture.keychain"))
            .unwrap();
        let store = Arc::new(KeychainStore {
            keychain: Some(root.join("fixture.keychain")),
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/", listener.local_addr().unwrap());
        let mut app = Application::open(&root.join("state"), Options::default()).unwrap();
        app.set_ai_secret_store(store.clone());
        let Data::AiSettings(initial) = app
            .request(app.epoch(), Request::AiSettings {})
            .unwrap()
            .data
        else {
            panic!()
        };
        let Data::AiSettings(saved) = app
            .request(
                app.epoch(),
                Request::SaveAiSettings {
                    fingerprint: initial.fingerprint,
                    revision: initial.settings.revision,
                    change: Change::Connection(ConnectionDraft {
                        id: None,
                        name: "Keychain fixture".into(),
                        provider: Provider::Ollama,
                        endpoint: endpoint.clone(),
                        timeout_ms: 2000,
                        authentication: Authentication::StoredBearer,
                    }),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let id = saved.settings.connections[0].id.clone();
        let binding = Binding::new(&id, &endpoint).unwrap();
        store
            .set(
                &binding,
                &Secret::new(b"synthetic-native-bound-token".to_vec()).unwrap(),
            )
            .unwrap();
        let draft = ConnectionDraft {
            id: Some(id),
            name: "Keychain fixture".into(),
            provider: Provider::Ollama,
            endpoint,
            timeout_ms: 2000,
            authentication: Authentication::StoredBearer,
        };
        let Data::Review(review) = app
            .request(
                app.epoch(),
                Request::ReviewAiDiscovery {
                    connection: draft.clone(),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let (cap, _) = app.begin_confirmation(&review.id).unwrap();
        assert!(app.confirm(cap, false, Cancellation::default()).is_err());
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        let server = std::thread::spawn(move || {
            let start = Instant::now();
            loop {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = Vec::new();
                    let mut buffer = [0; 4096];
                    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                        let n = stream.read(&mut buffer).unwrap();
                        assert!(n > 0);
                        request.extend_from_slice(&buffer[..n]);
                    }
                    let text = String::from_utf8(request).unwrap().to_ascii_lowercase();
                    assert!(text.starts_with("get /api/tags "));
                    assert!(text.contains("authorization: bearer synthetic-native-bound-token"));
                    let body = br#"{"models":[{"name":"fixture-model"}]}"#;
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
                    stream.write_all(body).unwrap();
                    break;
                }
                assert!(start.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let Data::Review(review) = app
            .request(
                app.epoch(),
                Request::ReviewAiDiscovery { connection: draft },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let (cap, _) = app.begin_confirmation(&review.id).unwrap();
        app.confirm(cap, true, Cancellation::default()).unwrap();
        let start = Instant::now();
        loop {
            let Data::AiJobs(jobs) = app.request(app.epoch(), Request::AiJobs {}).unwrap().data
            else {
                panic!()
            };
            let value = serde_json::to_value(&jobs).unwrap();
            assert!(!value.to_string().contains("synthetic-native-bound-token"));
            if value[0]["phase"] == "complete" {
                assert_eq!(value[0]["result"]["value"][0]["id"], "fixture-model");
                break;
            }
            assert!(
                value[0]["phase"] != "failed",
                "Approved Keychain-backed fixture failed: {:?}: {:?}",
                value[0]["error_kind"],
                value[0]["error"]
            );
            assert!(start.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(5));
        }
        server.join().unwrap();
        app.shutdown().unwrap();
        store.delete(&binding).unwrap();
        drop(app);
        drop(_keychain);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn actual_private_keychain_round_trip_and_wrong_endpoint_refusal() {
        let _fixture = OS_FIXTURE.lock().unwrap_or_else(|e| e.into_inner());
        use security_framework::os::macos::keychain::CreateOptions;
        let interaction_before =
            security_framework::os::macos::keychain::SecKeychain::user_interaction_allowed()
                .unwrap();
        let before = std::process::Command::new("/usr/bin/security")
            .args(["list-keychains", "-d", "user"])
            .output()
            .unwrap();
        assert!(before.status.success());
        let root = std::path::PathBuf::from("/private/tmp")
            .join(format!("gitfleet-ai-keychain-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("fixture.keychain");
        let mut _keychain = CreateOptions::new()
            .password("disposable-keychain-only")
            .prompt_user(false)
            .create(&path)
            .unwrap();
        let store = KeychainStore {
            keychain: Some(path),
        };
        let id = uuid::Uuid::new_v4().to_string();
        let binding = Binding::new(&id, "https://fixture.invalid/").unwrap();
        let first = Secret::new(b"synthetic-AI-token-only".to_vec()).unwrap();
        store.set(&binding, &first).unwrap();
        assert_eq!(
            store.get(&binding, Default::default()).unwrap().expose(),
            first.expose()
        );
        assert!(store
            .get(
                &Binding::new(&id, "https://other.invalid/").unwrap(),
                Default::default()
            )
            .is_err());
        // A locked store must fail without showing authentication UI.
        let lock_started = std::time::Instant::now();
        let locked = std::process::Command::new("/usr/bin/security")
            .arg("lock-keychain")
            .arg(store.keychain.as_ref().unwrap())
            .output()
            .unwrap();
        assert!(locked.status.success());
        let started = std::time::Instant::now();
        assert!(store.get(&binding, Default::default()).is_err());
        eprintln!(
            "private Keychain lock command: {} ms; locked read: {} ms",
            lock_started
                .elapsed()
                .as_millis()
                .saturating_sub(started.elapsed().as_millis()),
            started.elapsed().as_millis()
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        _keychain.unlock(Some("disposable-keychain-only")).unwrap();
        let second = Secret::new(b"replacement-synthetic-only".to_vec()).unwrap();
        store.set(&binding, &second).unwrap();
        assert_eq!(
            store.get(&binding, Default::default()).unwrap().expose(),
            second.expose()
        );
        store.delete(&binding).unwrap();
        assert!(store.get(&binding, Default::default()).is_err());
        store.delete(&binding).unwrap();
        // This fixture remains entirely under the owned temporary directory. No
        // default/login Keychain or global search-list setting is ever modified.
        drop(_keychain);
        std::fs::remove_dir_all(root).unwrap();
        let after = std::process::Command::new("/usr/bin/security")
            .args(["list-keychains", "-d", "user"])
            .output()
            .unwrap();
        assert_eq!(
            security_framework::os::macos::keychain::SecKeychain::user_interaction_allowed()
                .unwrap(),
            interaction_before
        );
        assert!(
            after.status.success() && before.stdout == after.stdout,
            "Private Keychain test must preserve the user search list"
        );
    }
}
