use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct Fixture {
    pub root: PathBuf,
    pub repo: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "gitfleet-m0-{}-{stamp}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        let fixture = Self { root, repo };
        fixture.git(&["-c", "init.templateDir=", "init", "-q", "-b", "main"]);
        for (name, value) in [
            ("user.name", "M0 Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
        ] {
            fixture.git(&["config", "--local", name, value]);
        }
        fixture
    }

    pub fn git_output(&self, args: &[&str]) -> Output {
        let mut command = Command::new("git");
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                command.env_remove(key);
            }
        }
        command
            .current_dir(&self.repo)
            .args(["-c", "maintenance.auto=false", "-c", "gc.auto=0"])
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }

    pub fn git(&self, args: &[&str]) -> Vec<u8> {
        let output = self.git_output(args);
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    pub fn commit_file(&self) {
        fs::write(self.repo.join("file.txt"), b"original\n").unwrap();
        self.git(&["add", "--", "file.txt"]);
        self.git(&["commit", "-qm", "baseline"]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

/// Capture every entry, including .git objects, index, refs, locks and worktree
/// bytes. Do not follow symlinks; access times are intentionally excluded.
pub fn tree_bytes(root: &Path) -> BTreeMap<PathBuf, (u32, Vec<u8>)> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, (u32, Vec<u8>)>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        };
        #[cfg(not(unix))]
        let mode = u32::from(metadata.permissions().readonly());
        let bytes = if metadata.is_symlink() {
            fs::read_link(path)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec()
        } else if metadata.is_file() {
            fs::read(path).unwrap()
        } else {
            Vec::new()
        };
        entries.insert(path.strip_prefix(root).unwrap().to_owned(), (mode, bytes));
        if metadata.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}
