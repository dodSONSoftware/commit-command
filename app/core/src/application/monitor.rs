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

    pub fn stop(&mut self) {
        self.watcher = None;
        self.events = None;
        self.status.running = false;
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
                for e in entries {
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
            || (ready && self.discovery_pending);
        let status = self.last_status.elapsed() >= self.timing.status_interval || ready;
        if discovery || status {
            self.first = None;
            self.last_event = None;
            self.discovery_pending = false;
            self.last_status = Instant::now();
        }
        if discovery {
            self.last_scan = Instant::now();
        }
        Due { discovery, status }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
