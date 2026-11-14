use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FolderTarget {
    workspace: String,
    worktree: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RepositoryPathAction {
    Copy,
    Reveal,
    Open,
}

#[tauri::command]
pub(super) async fn repository_path_action<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    state: tauri::State<'_, Arc<Host>>,
    target: FolderTarget,
    action: RepositoryPathAction,
) -> Result<(), String> {
    check(&window, "main")?;
    let (reply, receive) = mpsc::channel();
    queue(
        &state,
        Task::RepositoryPathAction {
            target,
            action,
            reply,
        },
    )?;
    tauri::async_runtime::spawn_blocking(move || {
        receive.recv().map_err(|_| "Folder action interrupted")?
    })
    .await
    .map_err(|e| e.to_string())?
}

impl FolderTarget {
    pub(super) fn apply(
        &self,
        core: &Application,
        action: RepositoryPathAction,
    ) -> Result<(), String> {
        let path = match (&action, self.worktree.as_deref()) {
            (RepositoryPathAction::Copy, Some(id)) => {
                core.stored_worktree_path(&self.workspace, id)
            }
            (_, worktree) => core.repository_folder(&self.workspace, worktree),
        }
        .map_err(|e| e.to_string())?;
        perform(&path, action)
    }
}

#[cfg(target_os = "macos")]
fn perform(path: &std::path::Path, action: RepositoryPathAction) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    match action {
        RepositoryPathAction::Copy => {
            let text = path
                .to_str()
                .ok_or("This path cannot be represented as clipboard text")?;
            let mut child = Command::new("/usr/bin/pbcopy")
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Could not copy repository path: {e}"))?;
            let write = child
                .stdin
                .take()
                .ok_or("Clipboard input is unavailable")?
                .write_all(text.as_bytes());
            let status = child.wait().map_err(|e| e.to_string())?;
            write.map_err(|e| e.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err("Could not copy repository path".into())
            }
        }
        RepositoryPathAction::Open | RepositoryPathAction::Reveal => {
            use objc2_app_kit::NSWorkspace;
            use objc2_foundation::NSURL;
            let finder = NSWorkspace::sharedWorkspace();
            let folder = NSURL::from_directory_path(path)
                .and_then(|url| url.path())
                .ok_or("Could not represent the folder path for Finder")?;
            let selected = if matches!(action, RepositoryPathAction::Reveal)
                || finder.isFilePackageAtPath(&folder)
            {
                Command::new("/usr/bin/open")
                    .arg("-R")
                    .arg(path)
                    .status()
                    .map_err(|e| format!("Could not reveal the repository folder: {e}"))?
                    .success()
            } else {
                finder.selectFile_inFileViewerRootedAtPath(None, &folder)
            };
            if selected {
                Ok(())
            } else {
                Err("Finder could not display the repository folder".into())
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn perform(_: &std::path::Path, _: RepositoryPathAction) -> Result<(), String> {
    Err("Repository path actions are supported on macOS".into())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::{fs, path::Path, process::Command};

    fn finder(script: &str, path: &Path) {
        let result = Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .arg(path)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[test]
    #[ignore = "opens disposable folders in the logged-in user's Finder"]
    fn finder_opens_directory_and_selects_app_without_launching() {
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("gitfleet-finder-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let workspace_dir = root.join("workspace");
        fs::create_dir(&workspace_dir).unwrap();
        let ordinary = workspace_dir.join("Ordinary repository");
        let bundle = workspace_dir.join("Harmless fixture.app");
        let marker = root.join("launched");
        // The applet has one observable effect if Launch Services starts it.
        let source = format!(
            "do shell script \"/usr/bin/touch \" & quoted form of \"{}\"",
            marker.display()
        );
        assert!(Command::new("/usr/bin/osacompile")
            .arg("-o")
            .arg(&bundle)
            .arg("-e")
            .arg(source)
            .status()
            .unwrap()
            .success());
        for repo in [&ordinary, &bundle] {
            assert!(Command::new("/usr/bin/git")
                .args(["init", "-q"])
                .arg(repo)
                .status()
                .unwrap()
                .success());
        }
        let mut core = Application::open(&root.join("state"), Default::default()).unwrap();
        let picker = core.register_directory(&workspace_dir).unwrap();
        core.request(
            core.epoch(),
            Request::Create {
                name: "Finder fixture".into(),
                picker,
            },
        )
        .unwrap();
        let Data::Workspaces(list) = core
            .request(core.epoch(), Request::Workspaces {})
            .unwrap()
            .data
        else {
            panic!()
        };
        let workspace = list.active.unwrap();
        let Data::Inventory(inventory) = core
            .request(
                core.epoch(),
                Request::Inventory {
                    workspace: workspace.clone(),
                },
            )
            .unwrap()
            .data
        else {
            panic!()
        };
        let repo = inventory
            .entries
            .iter()
            .find(|entry| entry.metadata.root.to_path().unwrap() == ordinary)
            .unwrap();
        FolderTarget {
            workspace: workspace.clone(),
            worktree: Some(repo.id.0.clone()),
        }
        .apply(&core, RepositoryPathAction::Open)
        .unwrap();
        finder(
            r#"on run argv
            set expected to POSIX file (item 1 of argv) as alias
            tell application "Finder"
                repeat 50 times
                    if (target of front Finder window as alias) is expected then return
                    delay 0.1
                end repeat
            end tell
            error "Finder did not browse the repository"
        end run"#,
            &ordinary,
        );

        let picker = core.register_directory(&bundle).unwrap();
        core.request(
            core.epoch(),
            Request::Create {
                name: "Bundle fixture".into(),
                picker,
            },
        )
        .unwrap();
        let Data::Workspaces(list) = core
            .request(core.epoch(), Request::Workspaces {})
            .unwrap()
            .data
        else {
            panic!()
        };
        FolderTarget {
            workspace: list.active.unwrap(),
            worktree: None,
        }
        .apply(&core, RepositoryPathAction::Open)
        .unwrap();
        finder(
            r#"on run argv
            set expected to POSIX file (item 1 of argv) as alias
            tell application "Finder"
                repeat 50 times
                    set chosen to selection as alias list
                    if chosen is {expected} and (target of front Finder window as alias) is (container of expected as alias) then return
                    delay 0.1
                end repeat
            end tell
            error "Finder did not reveal and select the bundle"
        end run"#,
            &bundle,
        );
        assert!(!marker.exists());
        finder(
            r#"on run argv
            set fixture to POSIX path of (POSIX file (item 1 of argv) as alias)
            tell application "Finder"
                repeat with w in (get Finder windows)
                    if (POSIX path of (target of w as alias)) starts with fixture then close w
                end repeat
            end tell
        end run"#,
            &root,
        );
        drop(core);
        fs::remove_dir_all(root).unwrap();
    }
}
