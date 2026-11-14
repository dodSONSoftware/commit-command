//! Finder launches do not inherit the user's shell PATH.
use std::{
    env::{join_paths, split_paths, JoinPathsError},
    ffi::{OsStr, OsString},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

const SYSTEM_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";
const HOMEBREW: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

fn executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

fn fallback_path(
    inherited: &OsStr,
    available: impl Fn(&Path) -> bool,
) -> Result<Option<OsString>, JoinPathsError> {
    let paths: Vec<_> = split_paths(inherited).collect();
    let selected = paths
        .iter()
        .map(|dir| dir.join("git"))
        .find(|git| available(git));
    // Preserve a shell-selected/custom Git, including test wrappers. Only the
    // system Git (or no Git) needs the conventional macOS installation fallback.
    if selected.is_some_and(|git| git != Path::new("/usr/bin/git") && git != Path::new("/bin/git"))
    {
        return Ok(None);
    }
    let Some(directory) = HOMEBREW
        .iter()
        .map(PathBuf::from)
        .find(|dir| available(&dir.join("git")))
    else {
        return Ok(None);
    };
    let ordered =
        std::iter::once(directory.clone()).chain(paths.into_iter().filter(|dir| dir != &directory));
    join_paths(ordered).map(Some)
}

pub(super) fn initialize() -> Result<(), JoinPathsError> {
    let inherited = std::env::var_os("PATH").unwrap_or_else(|| SYSTEM_PATH.into());
    if let Some(path) = fallback_path(&inherited, executable)? {
        // Called at the very start of main, before credential/application workers
        // or Tauri threads exist. All Git children then inherit the same PATH.
        std::env::set_var("PATH", path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn available<'a>(paths: &'a [&'a str]) -> impl Fn(&Path) -> bool + 'a {
        move |path| paths.iter().any(|candidate| path == Path::new(candidate))
    }
    #[test]
    fn finder_system_git_falls_back_to_apple_silicon_homebrew() {
        let next = fallback_path(
            OsStr::new(SYSTEM_PATH),
            available(&["/usr/bin/git", "/opt/homebrew/bin/git"]),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            next,
            OsStr::new("/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin")
        );
    }
    #[test]
    fn intel_homebrew_and_missing_system_git_are_supported() {
        for existing in [
            vec!["/usr/bin/git", "/usr/local/bin/git"],
            vec!["/usr/local/bin/git"],
        ] {
            let next = fallback_path(OsStr::new(SYSTEM_PATH), available(&existing))
                .unwrap()
                .unwrap();
            assert_eq!(
                next,
                OsStr::new("/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            );
        }
    }
    #[test]
    fn custom_git_and_existing_homebrew_priority_are_preserved() {
        for path in ["/custom/bin:/usr/bin", "/opt/homebrew/bin:/usr/bin"] {
            assert!(fallback_path(
                OsStr::new(path),
                available(&["/custom/bin/git", "/opt/homebrew/bin/git", "/usr/bin/git"])
            )
            .unwrap()
            .is_none());
        }
    }
    #[test]
    fn fallback_moves_a_late_homebrew_entry_without_duplicates() {
        let next = fallback_path(
            OsStr::new("/usr/bin:/opt/homebrew/bin:/bin"),
            available(&["/usr/bin/git", "/opt/homebrew/bin/git"]),
        )
        .unwrap()
        .unwrap();
        assert_eq!(next, OsStr::new("/opt/homebrew/bin:/usr/bin:/bin"));
    }
    #[test]
    fn no_installed_fallback_leaves_version_errors_to_core() {
        assert!(
            fallback_path(OsStr::new(SYSTEM_PATH), available(&["/usr/bin/git"]))
                .unwrap()
                .is_none()
        );
    }
}
