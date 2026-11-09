//! Ephemeral AI jobs run independently of the serialized application/storage owner.
use super::provider::{Approved, ProviderResult, Review};
use crate::{inspection::Cancellation, Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub global: usize,
    pub connection: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            global: 1,
            connection: 1,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AiJobPhase")]
pub enum Phase {
    Queued,
    Running,
    Complete,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiJob")]
pub struct Job {
    pub id: String,
    pub review: Review,
    pub phase: Phase,
    pub result: Option<ProviderResult>,
    pub error: Option<String>,
    pub error_kind: Option<ErrorKind>,
}
struct Entry {
    view: Job,
    cancel: Cancellation,
    created: Instant,
    finished: bool,
}
#[derive(Default)]
struct Inner {
    jobs: BTreeMap<String, Entry>,
    active: usize,
    connections: BTreeMap<String, usize>,
    generation: u64,
    limits: Limits,
}
#[derive(Default)]
pub(crate) struct Jobs {
    inner: Arc<Mutex<Inner>>,
}
fn unavailable() -> Error {
    Error::with_kind(ErrorKind::Unavailable, "AI job state unavailable")
}
impl Jobs {
    pub(crate) fn start(
        &mut self,
        approved: Approved,
        store: Option<Arc<dyn super::credentials::SecretStore>>,
        key: String,
        limits: Limits,
        cancel: Cancellation,
    ) -> Result<String> {
        if !(1..=4).contains(&limits.global) || !(1..=2).contains(&limits.connection) {
            return Err(Error::with_kind(
                ErrorKind::InvalidRequest,
                "Invalid AI request limits",
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let review = approved.review();
        let generation;
        {
            let mut inner = self.inner.lock().map_err(|_| unavailable())?;
            inner.jobs.retain(|_, e| {
                e.created.elapsed() < Duration::from_secs(600)
                    || matches!(e.view.phase, Phase::Queued | Phase::Running)
            });
            if inner.jobs.len() >= 32 {
                return Err(Error::with_kind(
                    ErrorKind::Busy,
                    "AI job limit reached; cancel or wait before retrying",
                ));
            }
            inner.limits = limits;
            generation = inner.generation;
            inner.jobs.insert(
                id.clone(),
                Entry {
                    view: Job {
                        id: id.clone(),
                        review,
                        phase: Phase::Queued,
                        result: None,
                        error: None,
                        error_kind: None,
                    },
                    cancel: cancel.clone(),
                    created: Instant::now(),
                    finished: false,
                },
            );
        }
        let shared = self.inner.clone();
        let job = id.clone();
        let spawn = std::thread::Builder::new()
            .name("gitfleet-ai-request".into())
            .spawn(move || {
                let start = Instant::now();
                let admitted = loop {
                    if cancel.is_cancelled() || start.elapsed() > Duration::from_secs(120) {
                        break false;
                    }
                    let Ok(mut inner) = shared.lock() else { return };
                    if inner.generation != generation || !inner.jobs.contains_key(&job) {
                        return;
                    }
                    if inner.active < inner.limits.global
                        && inner.connections.get(&key).copied().unwrap_or(0)
                            < inner.limits.connection
                    {
                        inner.active += 1;
                        *inner.connections.entry(key.clone()).or_default() += 1;
                        if let Some(entry) = inner.jobs.get_mut(&job) {
                            entry.view.phase = Phase::Running;
                        }
                        break true;
                    }
                    drop(inner);
                    std::thread::sleep(Duration::from_millis(5));
                };
                let result = if admitted {
                    let secret = match approved.binding() {
                        Some(binding) => store.as_ref().ok_or_else(|| Error::with_kind(ErrorKind::Unavailable, "Native AI secure store is unavailable; no unauthenticated fallback selected")).and_then(|store| store.get(binding, cancel.clone())).map(Some),
                        None => Ok(None),
                    };
                    match secret {
                        Ok(secret) if !cancel.is_cancelled() => approved.execute(secret, cancel.clone()),
                        Ok(_) => Err(Error::with_kind(ErrorKind::Cancelled, "AI request cancelled before credential use")),
                        Err(error) => Err(error),
                    }
                } else {
                    Err(Error::with_kind(
                        if cancel.is_cancelled() {
                            ErrorKind::Cancelled
                        } else {
                            ErrorKind::Timeout
                        },
                        "AI request cancelled or queue deadline expired",
                    ))
                };
                if let Ok(mut inner) = shared.lock() {
                    if admitted {
                        inner.active = inner.active.saturating_sub(1);
                        if let Some(active) = inner.connections.get_mut(&key) {
                            *active = active.saturating_sub(1);
                        }
                    }
                    if inner.generation != generation {
                        return;
                    }
                    if let Some(entry) = inner.jobs.get_mut(&job) {
                        entry.finished = true;
                        match result {
                            Ok(value) if !cancel.is_cancelled() => {
                                entry.view.phase = Phase::Complete;
                                entry.view.result = Some(value);
                            }
                            Ok(_) => {
                                entry.view.phase = Phase::Cancelled;
                                entry.view.error =
                                    Some("AI request cancelled; result discarded".into());
                                entry.view.error_kind = Some(ErrorKind::Cancelled);
                            }
                            Err(e) => {
                                entry.view.phase = if e.kind == ErrorKind::Cancelled {
                                    Phase::Cancelled
                                } else {
                                    Phase::Failed
                                };
                                entry.view.error_kind = Some(e.kind);
                                entry.view.error = Some(e.to_string());
                            }
                        }
                    }
                }
            });
        if spawn.is_err() {
            if let Ok(mut inner) = self.inner.lock() {
                inner.jobs.remove(&id);
            }
            return Err(unavailable());
        }
        Ok(id)
    }
    pub(crate) fn set_limits(&mut self, limits: Limits) -> Result<()> {
        if !(1..=4).contains(&limits.global) || !(1..=2).contains(&limits.connection) {
            return Err(Error::with_kind(
                ErrorKind::InvalidRequest,
                "Invalid AI request limits",
            ));
        }
        self.inner.lock().map_err(|_| unavailable())?.limits = limits;
        Ok(())
    }
    pub(crate) fn is_idle(&self) -> Result<bool> {
        let inner = self.inner.lock().map_err(|_| unavailable())?;
        Ok(inner.active == 0 && inner.jobs.values().all(|entry| entry.finished))
    }
    pub(crate) fn list(&self) -> Result<Vec<Job>> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| unavailable())?
            .jobs
            .values()
            .map(|e| e.view.clone())
            .collect())
    }
    pub(crate) fn cancel(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|_| unavailable())?;
        let entry = inner.jobs.get_mut(id).ok_or_else(unavailable)?;
        entry.cancel.cancel();
        entry.view.result = None;
        entry.view.phase = Phase::Cancelled;
        entry.view.error = Some("AI request cancelled; result discarded".into());
        entry.view.error_kind = Some(ErrorKind::Cancelled);
        Ok(())
    }
    pub(crate) fn reset(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.generation = inner.generation.wrapping_add(1);
            for entry in inner.jobs.values() {
                entry.cancel.cancel();
            }
            inner.jobs.clear();
        }
    }
    pub(crate) fn wait_shutdown(&self) -> Result<()> {
        let start = Instant::now();
        loop {
            if self.inner.lock().map_err(|_| unavailable())?.active == 0 {
                return Ok(());
            }
            if start.elapsed() > Duration::from_secs(5) {
                return Err(Error::with_kind(
                    ErrorKind::Timeout,
                    "AI requests have not stopped; retry closing",
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Jobs {
    fn drop(&mut self) {
        self.reset();
        let _ = self.wait_shutdown();
    }
}

#[cfg(test)]
mod idle_tests {
    use super::*;
    #[test]
    fn cancelled_display_state_is_not_idle_until_the_owned_worker_finishes() {
        let jobs = Jobs::default();
        let review =
            super::super::provider::Plan::discover(&super::super::config::ConnectionDraft {
                id: None,
                name: "Idle fixture".into(),
                provider: super::super::config::Provider::Ollama,
                endpoint: "http://127.0.0.1:1/".into(),
                timeout_ms: 100,
                authentication: Default::default(),
            })
            .unwrap()
            .review();
        let cancel = Cancellation::default();
        cancel.cancel();
        jobs.inner.lock().unwrap().jobs.insert(
            "fixture".into(),
            Entry {
                view: Job {
                    id: "fixture".into(),
                    review,
                    phase: Phase::Cancelled,
                    result: None,
                    error: None,
                    error_kind: Some(ErrorKind::Cancelled),
                },
                cancel,
                created: Instant::now(),
                finished: false,
            },
        );
        assert!(!jobs.is_idle().unwrap());
        jobs.inner
            .lock()
            .unwrap()
            .jobs
            .get_mut("fixture")
            .unwrap()
            .finished = true;
        assert!(jobs.is_idle().unwrap());
        jobs.inner.lock().unwrap().active = 1;
        assert!(!jobs.is_idle().unwrap());
        jobs.inner.lock().unwrap().active = 0;
    }
}
