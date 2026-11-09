//! Optional AI services. Configuration and generated text confer no write authority.
pub mod change;
pub mod config;
pub(crate) mod jobs;
pub mod provider;
pub mod store;
pub mod template;
#[cfg(test)]
mod tests;
