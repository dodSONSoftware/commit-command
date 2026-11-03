use super::*;
#[test]
fn git_initialization_probe_is_transient_and_never_accepted_as_a_completed_artifact() {
    probe_case(false);
}
#[test]
fn persistent_initialization_symlink_is_still_refused_before_object_import() {
    probe_case(true);
}
fn probe_case(persistent: bool) {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("gitfleet-init-probe-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let bin = root.join("bin");
    fs::create_dir(&bin).unwrap();
    let exec = std::process::Command::new("git")
        .arg("--exec-path")
        .output()
        .unwrap();
    assert!(exec.status.success());
    let real =
        fs::canonicalize(PathBuf::from(String::from_utf8(exec.stdout).unwrap().trim()).join("git"))
            .unwrap();
    let marker = root.join("probe-created");
    let release = root.join("release");
    let script=format!("#!/usr/bin/python3\nimport os,sys,time\nfrom pathlib import Path\nif sys.argv[-1].endswith('/loose.git') and 'init' in sys.argv:\n p=Path(sys.argv[-1]);p.mkdir(parents=True,exist_ok=True)\n link=p/'tFixtureProbe';link.symlink_to('testing')\n Path({:?}).write_text('real init protocol probe held')\n until=time.monotonic()+10\n while not Path({:?}).exists() and time.monotonic()<until:time.sleep(.002)\n link.unlink()\nos.execv({:?},[{:?}]+sys.argv[1:])\n",marker.to_str().unwrap(),release.to_str().unwrap(),real.to_str().unwrap(),real.to_str().unwrap());
    let script = if persistent {
        script.replace(" link.unlink()\n", " # leave persistent fixture link\n")
    } else {
        script
    };
    fs::write(bin.join("git"), script).unwrap();
    fs::set_permissions(bin.join("git"), fs::Permissions::from_mode(0o700)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "artifact_initialization::artifact_init_probe_worker",
            "--exact",
            "--ignored",
            "--nocapture",
        ])
        .env("PATH", path)
        .env("GITFLEET_INIT_PROBE_ROOT", &root)
        .env(
            "GITFLEET_INIT_PROBE_PERSISTENT",
            if persistent { "yes" } else { "no" },
        )
        .output()
        .unwrap();
    let created = marker.exists();
    fs::remove_dir_all(root).unwrap();
    assert!(created, "Probe was not exercised");
    assert!(
        output.status.success(),
        "worker stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
#[ignore = "subprocess-only controlled Git initialization probe"]
fn artifact_init_probe_worker() {
    let root =
        PathBuf::from(std::env::var_os("GITFLEET_INIT_PROBE_ROOT").expect("parent worker fixture"));
    let mut t = setup(true);
    trust(&mut t);
    let r = review(&mut t, "refs/heads/main");
    let observed = confirm(&mut t, &r, Cancellation::default());
    assert_eq!(observed.phase, ObservationPhase::Complete);
    let r = fetch_review(&mut t, &observed.plan.id);
    let captured = fetch_confirm(&mut t, &r, Cancellation::default());
    assert_eq!(
        captured.phase,
        gitfleet_core::remote::fetch::FetchCapturePhase::Complete
    );
    let before = tree_bytes(&t.f.repo);
    let r = import_review(&mut t, &captured.plan.transport.id);
    let waiter = std::thread::spawn(move || {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !root.join("probe-created").exists() && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(root.join("probe-created").exists());
        std::thread::sleep(std::time::Duration::from_millis(150));
        fs::write(root.join("release"), b"release").unwrap();
    });
    let result = import_confirm(&mut t, &r);
    waiter.join().unwrap();
    if std::env::var("GITFLEET_INIT_PROBE_PERSISTENT").unwrap() == "yes" {
        assert_eq!(
            result.phase,
            gitfleet_core::remote::fetch::FetchImportPhase::Blocked
        );
        assert_eq!(
            result.issue.unwrap().kind,
            ErrorKind::UnsupportedRepositoryState
        );
        assert_eq!(before, tree_bytes(&t.f.repo));
    } else {
        assert_eq!(
            result.phase,
            gitfleet_core::remote::fetch::FetchImportPhase::Complete,
            "{:?}",
            result.issue
        );
    }
}
