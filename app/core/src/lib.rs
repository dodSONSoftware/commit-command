//! GitFleet Core proof of concept. All write tests use disposable Git repositories.
//! Not a production-safe Git commit engine; see docs/LIMITATIONS.md.

pub mod commit;
pub mod discovery;
pub mod error;
pub mod git;
pub mod journal;
pub mod lab;

pub use error::{Error, Result};
