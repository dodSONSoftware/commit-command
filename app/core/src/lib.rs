//! Headless inspection, separately gated controlled writes and a disposable commit experiment.
//! See `docs/architecture/core-poc/LIMITATIONS.md` at the project root.

pub mod activity;
#[cfg(feature = "experimental-writes")]
pub mod commit;
pub mod diff;
pub mod discovery;
pub mod domain;
pub mod error;
#[cfg(feature = "experimental-writes")]
pub mod git;
pub mod graph;
pub mod history;
pub mod inspection;
#[cfg(feature = "experimental-writes")]
pub mod journal;
#[cfg(feature = "experimental-writes")]
pub mod lab;
mod locking;
mod process;
pub mod workspace;
#[cfg(feature = "controlled-writes")]
pub mod writes;

pub use error::{Error, ErrorKind, Result};

#[cfg(feature = "application-services")]
pub mod application;
#[cfg(feature = "application-services")]
pub mod remote;

#[cfg(feature = "application-services")]
pub mod credentials;

#[cfg(feature = "application-services")]
pub mod authentication;
