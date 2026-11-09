//! Native-supplied AI credential boundary. No tokens in settings, IPC or exports.
use super::{config::endpoint, provider::Secret};
use crate::{domain::digest, Error, ErrorKind, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AiAuthentication")]
pub enum Authentication {
    #[default]
    None,
    StoredBearer,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiCredentialBinding")]
pub struct Binding {
    pub connection: String,
    pub endpoint: String,
}
impl Binding {
    pub fn new(connection: &str, url: &str) -> Result<Self> {
        uuid::Uuid::parse_str(connection).map_err(|_| {
            Error::with_kind(
                ErrorKind::InvalidRequest,
                "Save a connection before selecting its secure credential",
            )
        })?;
        Ok(Self {
            connection: connection.into(),
            endpoint: endpoint(url)?,
        })
    }
    pub fn account(&self) -> Result<String> {
        let checked = Self::new(&self.connection, &self.endpoint)?;
        Ok(format!(
            "{}:{}",
            checked.connection,
            digest(checked.endpoint.as_bytes())
        ))
    }
}
/// Implemented only by the native adapter. Implementations must avoid blocking OS
/// authorization prompts. Callers keep lookup off the storage owner and bound
/// shutdown while a native service is unavailable.
pub trait SecretStore: Send + Sync {
    fn get(
        &self,
        binding: &Binding,
        cancellation: crate::inspection::Cancellation,
    ) -> Result<Secret>;
    fn set(&self, binding: &Binding, secret: &Secret) -> Result<()>;
    fn delete(&self, binding: &Binding) -> Result<()>;
}

/// Native-only helper transport. This API is never reachable from request JSON.
/// The adapter supplies a fixed authenticated executable and an anonymous-pipe body.
pub fn run_native_worker(
    command: std::process::Command,
    input: Vec<u8>,
    timeout: std::time::Duration,
    cancellation: crate::inspection::Cancellation,
) -> Result<Vec<u8>> {
    if input.len() > 32 * 1024 || timeout > std::time::Duration::from_secs(5) {
        return Err(Error::with_kind(
            ErrorKind::InvalidRequest,
            "AI credential helper bounds exceeded",
        ));
    }
    let options = crate::inspection::ReadOptions {
        timeout,
        max_output: 32 * 1024,
        cancellation,
    };
    let output = crate::process::bounded_input(command, &options, Some(input))
        .map_err(|e| Error::with_kind(e.kind, "AI credential helper did not complete"))?;
    if !output.status.success() {
        return Err(Error::with_kind(
            ErrorKind::Unavailable,
            "AI credential helper refused its caller or request",
        ));
    }
    Ok(output.stdout)
}
