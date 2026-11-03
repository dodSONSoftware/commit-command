//! A prepared Git transaction lets the owner check ref type while Git holds its lock.
use super::*;
use std::io::BufRead;
use std::sync::mpsc;

// A guard may run bounded Git reads. Serialize transactions before taking a
// process permit so multiple prepared transactions cannot exhaust all permits.
static TRANSACTION: Mutex<()> = Mutex::new(());
pub(crate) fn transaction(
    mut command: Command,
    options: &ReadOptions,
    reference: &str,
    new: &str,
    old: &str,
    prepared: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let start = Instant::now();
    let _serial = loop {
        options.check(start)?;
        match TRANSACTION.try_lock() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(_) => {
                return Err(Error::with_kind(
                    ErrorKind::Storage,
                    "Reference transaction owner unavailable",
                ))
            }
        }
    };
    let _permit = permit(options, start)?;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = child.stdin.take();
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let exceeded = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::sync_channel(8);
    let out_flag = exceeded.clone();
    let out = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout).take(4097);
        let mut total = 0;
        loop {
            let mut line = Vec::new();
            match reader.read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(n) => {
                    total += n;
                    if total > 4096 || sender.try_send(line).is_err() {
                        out_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                Err(_) => {
                    out_flag.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    });
    let err_flag = exceeded.clone();
    let err = std::thread::spawn(move || reader(stderr, 64 * 1024, err_flag));
    let mut failure = None;
    let initial = format!("start\nupdate {reference} {new} {old}\nprepare\n");
    if input.as_mut().is_none_or(|s| {
        s.write_all(initial.as_bytes())
            .and_then(|()| s.flush())
            .is_err()
    }) {
        failure = Some(Error::with_kind(
            ErrorKind::UncertainOutcome,
            "Cannot start reference transaction",
        ));
    }
    let mut guard = Some(prepared);
    let mut guard_error = None;
    let mut stage = 0;
    let mut status = None;
    let mut drained = false;
    let mut terminated = false;
    let mut wait_failed = false;
    let mut abort_started: Option<Instant> = None;
    loop {
        // Once abort is sent, allow Git a bounded cleanup interval even when
        // cancellation/deadline triggered it. Killing immediately can strand its lock.
        let interrupted = abort_started.map_or_else(
            || options.check(start).is_err(),
            |at| at.elapsed() >= Duration::from_millis(500),
        );
        if interrupted || exceeded.load(Ordering::Relaxed) {
            failure = Some(Error::with_kind(
                ErrorKind::UncertainOutcome,
                "Reference transaction interrupted; inspect actual ref and locks",
            ));
        }
        if failure.is_some() && !terminated {
            terminate(&mut child, Group::Owned);
            input.take();
            terminated = true;
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(line) if failure.is_none() => {
                let valid = match (stage, line.as_slice()) {
                    (0, b"start: ok\n") => {
                        stage = 1;
                        true
                    }
                    (1, b"prepare: ok\n") => {
                        guard_error = guard.take().expect("one prepared callback")()
                            .and_then(|()| options.check(start))
                            .err();
                        let decision = if guard_error.is_some() {
                            abort_started = Some(Instant::now());
                            b"abort\n".as_slice()
                        } else {
                            b"commit\n".as_slice()
                        };
                        if input
                            .as_mut()
                            .is_none_or(|s| s.write_all(decision).and_then(|()| s.flush()).is_err())
                        {
                            false
                        } else {
                            input.take();
                            stage = 2;
                            true
                        }
                    }
                    (2, b"commit: ok\n") if guard_error.is_none() => {
                        stage = 3;
                        true
                    }
                    (2, b"abort: ok\n") if guard_error.is_some() => {
                        stage = 3;
                        true
                    }
                    _ => false,
                };
                if !valid {
                    failure = Some(Error::with_kind(
                        ErrorKind::UncertainOutcome,
                        "Invalid reference transaction response",
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => drained = true,
            _ => {}
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(Some(value)) => status = Some(value),
                Ok(None) => {}
                Err(_) => {
                    terminate(&mut child, Group::Owned);
                    status = child.wait().ok();
                    wait_failed = status.is_none();
                    failure = Some(Error::with_kind(
                        ErrorKind::UncertainOutcome,
                        "Reference transaction wait failed",
                    ));
                }
            }
        }
        if (status.is_some() || wait_failed) && out.is_finished() && err.is_finished() && drained {
            break;
        }
    }
    input.take();
    out.join().map_err(|_| {
        Error::with_kind(
            ErrorKind::UncertainOutcome,
            "Reference transaction reader failed",
        )
    })?;
    if !matches!(err.join(), Ok(Ok(_))) {
        failure = Some(Error::with_kind(
            ErrorKind::UncertainOutcome,
            "Reference transaction diagnostic reader failed",
        ));
    }
    if failure.is_some() || !status.is_some_and(|s| s.success()) || stage != 3 {
        return Err(failure.unwrap_or_else(|| {
            Error::with_kind(
                ErrorKind::UncertainOutcome,
                "Reference transaction outcome requires inspection",
            )
        }));
    }
    if let Some(error) = guard_error {
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use std::fs;
    fn repo() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("gitfleet-ref-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        assert!(Command::new("git")
            .args(["-c", "init.templateDir=", "init", "--bare", "-q"])
            .arg(&path)
            .status()
            .unwrap()
            .success());
        path
    }
    fn git(path: &Path, args: &[&str]) -> Vec<u8> {
        let result = Command::new("git")
            .current_dir(path)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    }
    fn command(path: &Path) -> Command {
        let mut command = Command::new("git");
        command
            .current_dir(path)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "core.hooksPath=",
                "update-ref",
                "--no-deref",
                "--stdin",
            ]);
        command
    }
    #[test]
    fn prepared_guard_aborts_symbolic_ref_even_when_resolved_oid_matches() {
        let path = repo();
        let oid = String::from_utf8(git(&path, &["hash-object", "-w", "--stdin"])).unwrap();
        let oid = oid.trim();
        git(&path, &["update-ref", "refs/test/direct", oid]);
        git(
            &path,
            &["symbolic-ref", "refs/test/alias", "refs/test/direct"],
        );
        let error = transaction(
            command(&path),
            &ReadOptions::default(),
            "refs/test/alias",
            oid,
            oid,
            || {
                assert!(path.join("refs/test/alias.lock").is_file());
                assert_eq!(
                    git(&path, &["symbolic-ref", "refs/test/alias"]),
                    b"refs/test/direct\n"
                );
                Err(Error::with_kind(ErrorKind::StaleReview, "symbolic ref"))
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::StaleReview);
        assert!(!path.join("refs/test/alias.lock").exists());
        assert_eq!(
            git(&path, &["symbolic-ref", "refs/test/alias"]),
            b"refs/test/direct\n"
        );
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn prepared_cancellation_aborts_and_reaps_without_publication() {
        let path = repo();
        let oid = String::from_utf8(git(&path, &["hash-object", "-w", "--stdin"])).unwrap();
        let token = Cancellation::default();
        let options = ReadOptions {
            cancellation: token.clone(),
            ..ReadOptions::default()
        };
        let result = transaction(
            command(&path),
            &options,
            "refs/test/new",
            oid.trim(),
            &"0".repeat(40),
            || {
                assert!(path.join("refs/test/new.lock").is_file());
                token.cancel();
                Ok(())
            },
        );
        assert_eq!(result.unwrap_err().kind, ErrorKind::Cancelled);
        assert!(!path.join("refs/test/new.lock").exists());
        assert!(!path.join("refs/test/new").exists());
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn foreign_lock_is_never_removed() {
        let path = repo();
        let oid = String::from_utf8(git(&path, &["hash-object", "-w", "--stdin"])).unwrap();
        fs::create_dir_all(path.join("refs/test")).unwrap();
        fs::write(path.join("refs/test/new.lock"), b"foreign").unwrap();
        let result = transaction(
            command(&path),
            &ReadOptions::default(),
            "refs/test/new",
            oid.trim(),
            &"0".repeat(40),
            || panic!("must not prepare"),
        );
        assert!(result.is_err());
        assert_eq!(
            fs::read(path.join("refs/test/new.lock")).unwrap(),
            b"foreign"
        );
        fs::remove_dir_all(path).unwrap();
    }
}

#[cfg(test)]
mod abort_deadline_test {
    use super::*;
    #[test]
    fn unresponsive_abort_is_bounded_and_uncertain() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c","read a; read b; read c; printf 'start: ok\nprepare: ok\n'; read decision; exec sleep 20"]);
        let started = Instant::now();
        let result = transaction(
            command,
            &ReadOptions::default(),
            "refs/test/x",
            &"a".repeat(40),
            &"0".repeat(40),
            || Err(Error::with_kind(ErrorKind::Storage, "abort fixture")),
        );
        assert_eq!(result.unwrap_err().kind, ErrorKind::UncertainOutcome);
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
