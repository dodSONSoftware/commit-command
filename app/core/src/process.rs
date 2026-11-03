//! Private, bounded process transport. Callers are the fixed inspection services.
use crate::{Error, ErrorKind, Result};
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
#[cfg(feature = "application-services")]
mod reference;
#[cfg(feature = "application-services")]
pub(crate) use reference::transaction as reference_transaction;

#[derive(Debug, Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
#[derive(Debug, Clone)]
pub struct ReadOptions {
    pub timeout: Duration,
    pub max_output: usize,
    pub cancellation: Cancellation,
}
impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15),
            max_output: 8 * 1024 * 1024,
            cancellation: Cancellation::default(),
        }
    }
}
impl ReadOptions {
    pub(crate) fn check(&self, start: Instant) -> Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(Error::with_kind(
                ErrorKind::Cancelled,
                "Inspection cancelled",
            ));
        }
        if start.elapsed() >= self.timeout {
            return Err(Error::with_kind(ErrorKind::Timeout, "Inspection timed out"));
        }
        Ok(())
    }
}
static ACTIVE: Mutex<usize> = Mutex::new(0);
struct Permit;
impl Drop for Permit {
    fn drop(&mut self) {
        *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) -= 1;
    }
}
fn permit(options: &ReadOptions, start: Instant) -> Result<Permit> {
    loop {
        options.check(start)?;
        let mut count = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
        if *count < 4 {
            *count += 1;
            return Ok(Permit);
        }
        drop(count);
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn reader(
    mut input: impl Read,
    limit: usize,
    exceeded: Arc<AtomicBool>,
) -> std::io::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let available = limit.saturating_sub(result.len());
        result.extend_from_slice(&buffer[..count.min(available)]);
        if count > available {
            exceeded.store(true, Ordering::Relaxed);
        }
    }
    Ok(result)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    Owned,
    Transport,
    Inherited,
}
fn terminate(child: &mut std::process::Child, group: Group) {
    #[cfg(unix)]
    // Owned groups contain descendants; credential helper children inherit the
    // outer Git group so cancellation cannot orphan them. Hostile setsid is outside scope.
    if group != Group::Inherited {
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
    }
    let _ = child.kill();
}
fn bounded(command: Command, options: &ReadOptions) -> Result<Output> {
    bounded_input(command, options, None)
}
pub(crate) fn bounded_input(
    command: Command,
    options: &ReadOptions,
    input: Option<Vec<u8>>,
) -> Result<Output> {
    bounded_input_checked(command, options, input, || Ok(()))
}
/// Trusted callers can bound owned artifacts while a child is running.
pub(crate) fn bounded_input_checked(
    command: Command,
    options: &ReadOptions,
    input: Option<Vec<u8>>,
    check: impl Fn() -> Result<()>,
) -> Result<Output> {
    bounded_group(command, options, input, check, Group::Owned)
}
/// Transport helpers must remain inside the outer Git process group.
#[cfg(feature = "application-services")]
pub(crate) fn credential_helper(
    command: Command,
    options: &ReadOptions,
    input: Vec<u8>,
) -> Result<Output> {
    bounded_group(command, options, Some(input), || Ok(()), Group::Inherited)
}
/// A completed network operation cannot leave background helper descendants.
#[cfg(feature = "application-services")]
pub(crate) fn transport(
    command: Command,
    options: &ReadOptions,
    check: impl Fn() -> Result<()>,
) -> Result<Output> {
    bounded_group(command, options, None, check, Group::Transport)
}
fn bounded_group(
    mut command: Command,
    options: &ReadOptions,
    input: Option<Vec<u8>>,
    check: impl Fn() -> Result<()>,
    group: Group,
) -> Result<Output> {
    check()?;
    let start = Instant::now();
    let _permit = permit(options, start)?;
    if options.max_output == 0 || options.max_output > 64 * 1024 * 1024 {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "Output limit must be 1..64 MiB",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        if group != Group::Inherited {
            command.process_group(0);
        }
    }
    let mut child = command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input_thread = input.map(|bytes| {
        let mut stdin = child.stdin.take().expect("piped input");
        std::thread::spawn(move || stdin.write_all(&bytes))
    });
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let exceeded = Arc::new(AtomicBool::new(false));
    let out_limit = options.max_output;
    let out_exceeded = exceeded.clone();
    let err_exceeded = exceeded.clone();
    let out = std::thread::spawn(move || reader(stdout, out_limit, out_exceeded));
    let err = std::thread::spawn(move || reader(stderr, 64 * 1024, err_exceeded));
    let mut failure = None;
    let status = loop {
        if let Err(error) = options.check(start).and_then(|()| check()) {
            failure = Some(error);
            terminate(&mut child, group);
            break child.wait()?;
        }
        if exceeded.load(Ordering::Relaxed) {
            failure = Some(Error::with_kind(
                ErrorKind::OutputLimit,
                "Inspection output limit exceeded",
            ));
            terminate(&mut child, group);
            break child.wait()?;
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                terminate(&mut child, group);
                let _ = child.wait();
                failure = Some(error.into());
                break child.wait()?;
            }
        }
    };
    if group == Group::Transport {
        terminate(&mut child, group);
    }
    // A descendant may still own a pipe after the main process exits.
    while !out.is_finished()
        || !err.is_finished()
        || input_thread
            .as_ref()
            .is_some_and(|writer| !writer.is_finished())
    {
        if let Err(error) = options.check(start).and_then(|()| check()) {
            failure = Some(error);
            terminate(&mut child, group);
        }
        if exceeded.load(Ordering::Relaxed) {
            failure = Some(Error::with_kind(
                ErrorKind::OutputLimit,
                "Inspection output limit exceeded",
            ));
            terminate(&mut child, group);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let stdout = out
        .join()
        .map_err(|_| Error::with_kind(ErrorKind::Io, "Output reader failed"))??;
    let stderr = err
        .join()
        .map_err(|_| Error::with_kind(ErrorKind::Io, "Diagnostic reader failed"))??;
    let input_result = if let Some(writer) = input_thread {
        writer
            .join()
            .map_err(|_| Error::with_kind(ErrorKind::Io, "Input writer failed"))?
    } else {
        Ok(())
    };
    if let Some(error) = failure {
        return Err(error);
    }
    if status.success() {
        input_result?;
    }
    if exceeded.load(Ordering::Relaxed) {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "Inspection output limit exceeded",
        ));
    }
    check()?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
fn base() -> Command {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", "")
        .args([
            "--no-pager",
            "--no-optional-locks",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "core.hooksPath=",
            "-c",
            "submodule.recurse=false",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
            "-c",
            "core.quotePath=true",
            "-c",
            "diff.external=",
            "-c",
            "log.showSignature=false",
        ]);
    command
}
pub(crate) fn git(repo: &Path, args: &[OsString], options: &ReadOptions) -> Result<Output> {
    static VERSION: OnceLock<bool> = OnceLock::new();
    if VERSION.get().is_none() {
        let output = bounded(
            {
                let mut command = base();
                command.arg("--version");
                command
            },
            options,
        )?;
        let version = String::from_utf8_lossy(&output.stdout);
        let numbers: Vec<u32> = version
            .split_whitespace()
            .nth(2)
            .unwrap_or("")
            .split('.')
            .take(3)
            .filter_map(|part| part.parse().ok())
            .collect();
        let valid = output.status.success() && numbers.as_slice() >= [2, 56, 0].as_slice();
        let _ = VERSION.set(valid);
    }
    if VERSION.get() != Some(&true) {
        return Err(Error::with_kind(
            ErrorKind::UnsupportedGitVersion,
            "Git 2.56.0 or newer is required",
        ));
    }
    let mut command = base();
    command.current_dir(repo).args(args);
    bounded(command, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_permits_never_exceed_four() {
        use std::sync::atomic::AtomicUsize;
        let active = AtomicUsize::new(0);
        let maximum = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let active = &active;
                let maximum = &maximum;
                scope.spawn(move || {
                    let options = ReadOptions::default();
                    let _permit = permit(&options, Instant::now()).unwrap();
                    let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum.fetch_max(count, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(15));
                    active.fetch_sub(1, Ordering::SeqCst);
                });
            }
        });
        assert!((1..=4).contains(&maximum.load(Ordering::SeqCst)));
    }
    #[cfg(unix)]
    #[test]
    fn bounded_input_checks_writer_completion_and_descendant_timeout() {
        let mut cat = Command::new("/bin/cat");
        cat.arg("-");
        let data = vec![b'x'; 65536];
        let output = bounded_input(cat, &ReadOptions::default(), Some(data.clone())).unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, data);
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exec 3<&0; (sleep 20 <&3 >/dev/null 2>&1) & exit 0"]);
        let options = ReadOptions {
            timeout: Duration::from_millis(100),
            ..ReadOptions::default()
        };
        assert_eq!(
            bounded_input(command, &options, Some(vec![0; 1024 * 1024]))
                .unwrap_err()
                .kind,
            ErrorKind::Timeout
        );
    }
    #[cfg(unix)]
    #[test]
    fn bounded_process_limits_and_cancellation_reap_children() {
        fn shell(text: &str) -> Command {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", text]);
            command
        }
        let mut options = ReadOptions {
            timeout: Duration::from_millis(50),
            ..ReadOptions::default()
        };
        assert_eq!(
            bounded(shell("sleep 20"), &options).unwrap_err().kind,
            ErrorKind::Timeout
        );
        options.timeout = Duration::from_secs(2);
        options.max_output = 32;
        assert_eq!(
            bounded(shell("while :; do printf abcdefgh; done"), &options)
                .unwrap_err()
                .kind,
            ErrorKind::OutputLimit
        );
        let midflight = ReadOptions {
            timeout: Duration::from_secs(2),
            ..ReadOptions::default()
        };
        let token = midflight.cancellation.clone();
        let worker = std::thread::spawn(move || bounded(shell("sleep 20"), &midflight));
        std::thread::sleep(Duration::from_millis(30));
        token.cancel();
        assert_eq!(
            worker.join().unwrap().unwrap_err().kind,
            ErrorKind::Cancelled
        );
        options.cancellation.cancel();
        assert_eq!(
            bounded(shell("exit 0"), &options).unwrap_err().kind,
            ErrorKind::Cancelled
        );
    }
    #[cfg(unix)]
    #[test]
    fn trusted_artifact_check_prevents_spawn_and_reaps_running_process() {
        let root =
            std::env::temp_dir().join(format!("gitfleet-process-budget-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let marker = root.join("started");
        let command = || {
            let mut command = Command::new("/bin/sh");
            command
                .args(["-c", "printf started > \"$1\"; exec sleep 30", "fixture"])
                .arg(&marker);
            command
        };
        let blocked = || {
            Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "Fixture artifact limit",
            ))
        };
        assert_eq!(
            bounded_input_checked(command(), &ReadOptions::default(), None, blocked)
                .unwrap_err()
                .kind,
            ErrorKind::OutputLimit
        );
        assert!(!marker.exists());
        let began = Instant::now();
        let guard = || if marker.exists() { blocked() } else { Ok(()) };
        assert_eq!(
            bounded_input_checked(command(), &ReadOptions::default(), None, guard)
                .unwrap_err()
                .kind,
            ErrorKind::OutputLimit
        );
        assert!(marker.exists());
        assert!(began.elapsed() < Duration::from_secs(5));
        std::fs::remove_dir_all(root).unwrap();
    }
}
