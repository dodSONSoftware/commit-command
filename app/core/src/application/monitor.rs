use crate::{
    discovery::DiscoveryOptions,
    workspace::{StoredWorktree, Workspace},
    Result,
};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::{Duration, Instant},
};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitorStatus {
    pub running: bool,
    pub epoch: u64,
    pub watched: usize,
    pub degraded: Option<String>,
}
pub(super) struct Due {
    pub discovery: bool,
    pub status: bool,
}
#[derive(Debug, Clone)]
pub struct MonitoringOptions {
    pub debounce: Duration,
    pub maximum_coalescing: Duration,
    pub status_interval: Duration,
    pub discovery_interval: Duration,
}
impl Default for MonitoringOptions {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(250),
            maximum_coalescing: Duration::from_secs(2),
            status_interval: Duration::from_secs(10),
            discovery_interval: Duration::from_secs(60),
        }
    }
}
pub struct Monitor {
    timing: MonitoringOptions,
    watcher: Option<RecommendedWatcher>,
    events: Option<Receiver<bool>>,
    overflow: Arc<AtomicBool>,
    status: MonitorStatus,
    first: Option<Instant>,
    last_event: Option<Instant>,
    discovery_pending: bool,
    last_status: Instant,
    last_scan: Instant,
    discovery_retry_at: Option<Instant>,
}
impl Default for Monitor {
    fn default() -> Self {
        Self {
            timing: MonitoringOptions::default(),
            watcher: None,
            events: None,
            overflow: Arc::new(AtomicBool::new(false)),
            status: MonitorStatus {
                running: false,
                epoch: 0,
                watched: 0,
                degraded: None,
            },
            first: None,
            last_event: None,
            discovery_pending: false,
            last_status: Instant::now(),
            last_scan: Instant::now(),
            discovery_retry_at: None,
        }
    }
}
fn relevant(path: &Path, root: &Path, options: &DiscoveryOptions) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let names = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();
    if names
        .windows(2)
        .any(|p| p[0] == ".git" && p[1] == "objects")
    {
        return false;
    }
    !names
        .iter()
        .any(|n| options.excluded_names.iter().any(|e| e == n))
        && !options
            .excluded_patterns
            .iter()
            .any(|p| glob::Pattern::new(p).is_ok_and(|p| p.matches_path(relative)))
}
impl Monitor {
    pub fn with_options(timing: MonitoringOptions) -> Result<Self> {
        if timing.debounce.is_zero()
            || timing.maximum_coalescing < timing.debounce
            || timing.status_interval.is_zero()
            || timing.discovery_interval.is_zero()
        {
            return Err(crate::Error::with_kind(
                crate::ErrorKind::InvalidRequest,
                "Invalid monitoring schedule",
            ));
        }
        Ok(Self {
            timing,
            ..Self::default()
        })
    }
    pub(super) fn failure(&mut self, error: &crate::Error) {
        self.status.degraded = Some(format!(
            "Reconciliation incomplete: {:?}: {}",
            error.kind, error
        ));
    }

    pub(super) fn retry_discovery(&mut self) {
        self.discovery_retry_at = Some(Instant::now() + Duration::from_secs(5));
    }
    pub fn stop(&mut self) {
        self.watcher = None;
        self.events = None;
        self.status.running = false;
        self.discovery_retry_at = None;
        self.first = None;
        self.last_event = None;
        self.discovery_pending = false;
    }
    pub fn status(&self) -> MonitorStatus {
        self.status.clone()
    }
    pub fn activate(
        &mut self,
        workspace: &Workspace,
        entries: &[StoredWorktree],
        epoch: u64,
    ) -> Result<()> {
        self.stop();
        let root = workspace.root.to_path()?;
        let options = workspace.discovery.clone();
        let (tx, rx) = mpsc::sync_channel(128);
        let overflow = Arc::new(AtomicBool::new(false));
        let flag = overflow.clone();
        let filtered_root = root.clone();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            let discovery = match event {
                Ok(e) => {
                    if !e
                        .paths
                        .iter()
                        .any(|p| relevant(p, &filtered_root, &options))
                    {
                        return;
                    }
                    matches!(
                        e.kind,
                        EventKind::Create(_)
                            | EventKind::Remove(_)
                            | EventKind::Modify(notify::event::ModifyKind::Name(_))
                    )
                }
                Err(_) => true,
            };
            if tx.try_send(discovery).is_err() {
                flag.store(true, Ordering::Relaxed);
            }
        });
        self.status = MonitorStatus {
            running: true,
            epoch,
            watched: 0,
            degraded: None,
        };
        match watcher {
            Ok(mut watcher) => {
                if watcher.watch(&root, RecursiveMode::Recursive).is_ok() {
                    self.status.watched += 1;
                } else {
                    self.status.degraded =
                        Some("Root watch unavailable; periodic reconciliation active".into());
                }
                let mut paths = std::collections::BTreeSet::new();
                for e in entries.iter().filter(|e| e.superseded_by.is_none()) {
                    for p in [&e.metadata.git_dir, &e.metadata.common_dir] {
                        let p = p.to_path()?;
                        if !p.starts_with(&root) {
                            paths.insert(p.clone());
                            paths.insert(p.join("refs"));
                        }
                    }
                }
                for path in paths {
                    if path.exists() {
                        let mode = if path.ends_with("refs") {
                            RecursiveMode::Recursive
                        } else {
                            RecursiveMode::NonRecursive
                        };
                        if watcher.watch(&path, mode).is_ok() {
                            self.status.watched += 1;
                        } else {
                            self.status.degraded = Some(
                                "Metadata watch unavailable; periodic reconciliation active".into(),
                            );
                        }
                    }
                }
                self.watcher = Some(watcher);
            }
            Err(_) => {
                self.status.degraded =
                    Some("Notification backend unavailable; periodic reconciliation active".into());
            }
        }
        self.events = Some(rx);
        self.overflow = overflow;
        self.last_status = Instant::now();
        self.last_scan = Instant::now();
        Ok(())
    }
    pub(super) fn due(&mut self) -> Due {
        if !self.status.running {
            return Due {
                discovery: false,
                status: false,
            };
        }
        if let Some(events) = &self.events {
            while let Ok(discovery) = events.try_recv() {
                let now = Instant::now();
                self.first.get_or_insert(now);
                self.last_event = Some(now);
                self.discovery_pending |= discovery;
            }
        }
        if self.overflow.swap(false, Ordering::Relaxed) {
            self.first = Some(Instant::now() - self.timing.maximum_coalescing);
            self.discovery_pending = true;
        }
        let ready = self
            .first
            .is_some_and(|at| at.elapsed() >= self.timing.maximum_coalescing)
            || self
                .last_event
                .is_some_and(|at| at.elapsed() >= self.timing.debounce);
        let discovery = self.last_scan.elapsed() >= self.timing.discovery_interval
            || self
                .discovery_retry_at
                .is_some_and(|at| Instant::now() >= at)
            || (ready && self.discovery_pending);
        let status = self.last_status.elapsed() >= self.timing.status_interval || ready;
        // A periodic status refresh does not consume an event still debouncing.
        // Discovery (or a ready status-only burst) consumes the queued events.
        if discovery || ready {
            self.first = None;
            self.last_event = None;
            self.discovery_pending = false;
        }
        if discovery || status {
            self.last_status = Instant::now();
        }
        if discovery {
            self.discovery_retry_at = None;
            self.last_scan = Instant::now();
        }
        Due { discovery, status }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periodic_status_tick_preserves_debouncing_repository_discovery() {
        let mut m = Monitor::default();
        m.status.running = true;
        let (sender, receiver) = mpsc::sync_channel(8);
        m.events = Some(receiver);
        m.last_status = Instant::now() - m.timing.status_interval;
        sender.send(true).unwrap();
        let first = m.due();
        assert!(first.status && !first.discovery);
        assert!(m.discovery_pending, "status-only tick discarded discovery");
        assert!(m.first.is_some() && m.last_event.is_some());
        m.last_event = Some(Instant::now() - m.timing.debounce);
        let next = m.due();
        assert!(next.discovery && next.status);
        assert!(!m.discovery_pending && m.first.is_none() && m.last_event.is_none());
        assert!(!m.due().discovery);
    }

    #[test]
    fn status_tick_then_debounce_discovers_new_repository_in_disposable_workspace() {
        use crate::application::{Application, Request};
        use std::{fs, process::Command};
        let root = std::env::temp_dir().join(format!("monitor-debounce-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let mut app = Application::open(&root.join("state"), Default::default()).unwrap();
        let picker = app.register_directory(&workspace).unwrap();
        app.request(
            app.epoch(),
            Request::Create {
                name: "Debounce".into(),
                picker,
            },
        )
        .unwrap();
        let id = app.writes.workspace.active().unwrap().unwrap();
        app.monitor.stop();
        app.monitor.status.running = true;
        let (sender, receiver) = mpsc::sync_channel(8);
        app.monitor.events = Some(receiver);
        let repo = workspace.join("new-repo");
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        app.monitor.last_status = Instant::now() - app.monitor.timing.status_interval;
        sender.send(true).unwrap();
        app.reconcile().unwrap();
        assert!(app.writes.workspace.inventory(&id).unwrap().is_empty());
        app.monitor.last_event = Some(Instant::now() - app.monitor.timing.debounce);
        assert!(app.reconcile().unwrap());
        let entries = app.writes.workspace.inventory(&id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].metadata.root.to_path().unwrap(),
            fs::canonicalize(repo).unwrap()
        );
        app.shutdown().unwrap();
        drop(app);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventory_work_is_linear_across_monitor_passes() {
        use crate::{
            application::{Application, Data, Request},
            workspace::INVENTORY_WORK,
        };
        use std::{fs, process::Command};
        for count in [20, 40] {
            let root = std::env::temp_dir().join(format!("monitor-scale-{}", uuid::Uuid::new_v4()));
            let workspace = root.join("workspace");
            fs::create_dir_all(&workspace).unwrap();
            for i in 0..count {
                let repo = workspace.join(format!("repo-{i}"));
                assert!(Command::new("git")
                    .args(["init", "-q"])
                    .arg(&repo)
                    .status()
                    .unwrap()
                    .success());
                fs::write(repo.join("untracked"), b"changed").unwrap();
            }
            let mut app = Application::open(&root.join("state"), Default::default()).unwrap();
            let picker = app.register_directory(&workspace).unwrap();
            app.request(
                app.epoch(),
                Request::Create {
                    name: "Scale".into(),
                    picker,
                },
            )
            .unwrap();
            let id = app.writes.workspace.active().unwrap().unwrap();
            while app.writes.workspace.can_continue_scan(&id) {
                app.request(
                    app.epoch(),
                    Request::ContinueScan {
                        workspace: id.0.clone(),
                    },
                )
                .unwrap();
            }
            INVENTORY_WORK.set((0, 0));
            for entry in app.writes.workspace.inventory(&id).unwrap() {
                app.writes.workspace.refresh(&id, &entry.id).unwrap();
            }
            let baseline = INVENTORY_WORK.get();
            eprintln!(
                "{count} worktrees before: {} inventory loads, {} decoded rows",
                baseline.0, baseline.1
            );
            assert_eq!(baseline, (count + 1, count * (count + 1)));
            app.monitor.stop();
            app.monitor.status.running = true;
            app.monitor.last_status = Instant::now() - Duration::from_secs(11);
            INVENTORY_WORK.set((0, 0));
            assert!(app.reconcile().unwrap());
            let counts = INVENTORY_WORK.get();
            eprintln!(
                "{count} worktrees: {} inventory loads, {} decoded rows",
                counts.0, counts.1
            );
            assert_eq!(counts, (1, count));
            assert!(matches!(
                app.request(app.epoch(), Request::Workspaces {})
                    .unwrap()
                    .data,
                Data::Workspaces(_)
            ));
            let mut ticks = 1;
            while app.status_pass.is_some() {
                assert!(matches!(
                    app.request(app.epoch(), Request::Workspaces {})
                        .unwrap()
                        .data,
                    Data::Workspaces(_)
                ));
                assert!(app.reconcile().unwrap());
                ticks += 1;
            }
            assert_eq!(ticks, count);
            assert_eq!(INVENTORY_WORK.get(), (1, count));
            let Data::Inventory(view) = app
                .request(
                    app.epoch(),
                    Request::Inventory {
                        workspace: id.0.clone(),
                    },
                )
                .unwrap()
                .data
            else {
                panic!()
            };
            assert_eq!(view.entries.len(), count);
            assert!(view
                .entries
                .iter()
                .all(|e| e.status.as_ref().unwrap().counts.untracked == 1));
            app.shutdown().unwrap();
            drop(app);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn burst_coalescing_and_periodic_fallback_are_bounded() {
        let mut m = Monitor::default();
        m.status.running = true;
        let (sender, receiver) = mpsc::sync_channel(128);
        m.events = Some(receiver);
        for _ in 0..128 {
            sender.try_send(false).unwrap();
        }
        assert!(!m.due().status);
        m.first = Some(Instant::now() - Duration::from_secs(3));
        sender.try_send(true).unwrap();
        assert!(m.due().discovery);
        m.last_status = Instant::now() - Duration::from_secs(11);
        assert!(m.due().status);
        m.last_scan = Instant::now() - Duration::from_secs(61);
        assert!(m.due().discovery);
        m.stop();
        assert!(!m.due().discovery);
    }
    #[test]
    fn unfinished_discovery_retries_before_the_periodic_scan_and_clears_on_stop() {
        let mut m = Monitor::default();
        m.status.running = true;
        m.retry_discovery();
        assert!(!m.due().discovery);
        m.discovery_retry_at = Some(Instant::now());
        assert!(m.due().discovery);
        assert!(!m.due().discovery);
        m.retry_discovery();
        m.stop();
        assert!(m.discovery_retry_at.is_none());
        assert!(!m.due().discovery);
    }
    #[test]
    fn filtering_and_overflow_do_not_schedule_writes() {
        let mut m = Monitor::default();
        m.status.running = true;
        m.overflow.store(true, Ordering::Relaxed);
        assert!(m.due().discovery);
        assert!(!relevant(
            Path::new("/repo/.git/objects/aa/x"),
            Path::new("/repo"),
            &DiscoveryOptions::default()
        ));
        assert!(!relevant(
            Path::new("/repo/node_modules/a"),
            Path::new("/repo"),
            &DiscoveryOptions::default()
        ));
        m.stop();
        assert!(!m.due().status);
    }
}

#[cfg(all(test, unix))]
mod slow_discovery_tests {
    use super::*;
    use crate::{
        application::{Application, Data, Request},
        ErrorKind,
    };
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

    #[test]
    fn slow_discovery_allows_manual_and_scheduled_sibling_progress() {
        const CHILD: &str = "GITFLEET_SLOW_SCAN_FIXTURE";
        let Some(root) = std::env::var_os(CHILD) else {
            let root = std::env::temp_dir().join(format!("slow-scan-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            let bin = root.join("bin");
            fs::create_dir(&bin).unwrap();
            let wrapper = bin.join("git");
            fs::write(&wrapper,"#!/bin/sh\ncase \"$PWD\" in */a-slow) if [ -f ../blocked ]; then sleep 5; fi;; esac\nPATH=${PATH#*:} exec git \"$@\"\n").unwrap();
            fs::set_permissions(wrapper, fs::Permissions::from_mode(0o700)).unwrap();
            let path = std::env::var_os("PATH").unwrap();
            let child=Command::new(std::env::current_exe().unwrap()).args(["--exact","application::monitor::slow_discovery_tests::slow_discovery_allows_manual_and_scheduled_sibling_progress","--nocapture"])
                .env(CHILD,&root).env("PATH",std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&path))).unwrap()).output().unwrap();
            fs::remove_dir_all(root).unwrap();
            assert!(
                child.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&child.stdout),
                String::from_utf8_lossy(&child.stderr)
            );
            return;
        };
        let root = fs::canonicalize(PathBuf::from(root)).unwrap();
        for scheduled in [false, true] {
            let case = root.join(if scheduled { "scheduled" } else { "manual" });
            let workspace = case.join("workspace");
            fs::create_dir_all(&workspace).unwrap();
            for name in ["a-slow", "z-healthy"] {
                assert!(Command::new("git")
                    .args(["init", "-q"])
                    .arg(workspace.join(name))
                    .status()
                    .unwrap()
                    .success());
            }
            fs::write(workspace.join("blocked"), b"blocked").unwrap();
            let mut app = Application::open(&case.join("state"), Default::default()).unwrap();
            app.read_options.timeout = Duration::from_secs(2);
            let picker = app.register_directory(&workspace).unwrap();
            app.request(
                app.epoch(),
                Request::Create {
                    name: "Slow fixture".into(),
                    picker,
                },
            )
            .unwrap();
            let id = app.writes.workspace.active().unwrap().unwrap();
            for _ in 0..2 {
                if scheduled {
                    app.monitor.discovery_retry_at = Some(Instant::now());
                    assert!(app.reconcile().unwrap());
                } else {
                    app.request(
                        app.epoch(),
                        Request::ContinueScan {
                            workspace: id.0.clone(),
                        },
                    )
                    .unwrap();
                }
            }
            let Data::Inventory(view) = app
                .request(
                    app.epoch(),
                    Request::Inventory {
                        workspace: id.0.clone(),
                    },
                )
                .unwrap()
                .data
            else {
                panic!()
            };
            let scan = view.scan.unwrap();
            assert!(!scan.complete && scan.progress.is_some());
            assert!(scan
                .issues
                .iter()
                .any(|i| i.path.to_path().unwrap() == workspace.join("a-slow")
                    && i.error.kind == ErrorKind::Timeout));
            assert!(
                view.entries
                    .iter()
                    .any(|e| e.metadata.root.to_path().unwrap() == workspace.join("z-healthy")),
                "healthy sibling was starved, scheduled={scheduled}"
            );
            let healthy_id = view
                .entries
                .iter()
                .find(|e| e.metadata.root.to_path().unwrap() == workspace.join("z-healthy"))
                .unwrap()
                .id
                .clone();
            let options = crate::inspection::ReadOptions::default();
            options.cancellation.cancel();
            assert_eq!(
                app.request_with_cancellation(
                    app.epoch(),
                    Request::ContinueScan {
                        workspace: id.0.clone()
                    },
                    options.cancellation
                )
                .unwrap_err()
                .kind,
                ErrorKind::Cancelled
            );
            fs::remove_file(workspace.join("blocked")).unwrap();
            app.request(
                app.epoch(),
                Request::ContinueScan {
                    workspace: id.0.clone(),
                },
            )
            .unwrap();
            let Data::Inventory(view) = app
                .request(
                    app.epoch(),
                    Request::Inventory {
                        workspace: id.0.clone(),
                    },
                )
                .unwrap()
                .data
            else {
                panic!()
            };
            assert_eq!(view.entries.len(), 2);
            assert!(view.entries.iter().any(|e| e.id == healthy_id));
            let direct = crate::discovery::scan(&workspace, &Default::default()).unwrap();
            assert!(direct.complete);
            let mut found = view
                .entries
                .iter()
                .map(|e| e.metadata.root.to_path().unwrap())
                .collect::<Vec<_>>();
            let mut expected = direct
                .repositories
                .iter()
                .map(|e| e.metadata.root.to_path().unwrap())
                .collect::<Vec<_>>();
            found.sort();
            expected.sort();
            assert_eq!(found, expected);
            assert!(view.scan.unwrap().complete);
            app.shutdown().unwrap();
        }
    }
}
