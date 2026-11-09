//! Fixed provider routes with bounded, cancellable subprocess transport.
//! Source/auth travel over stdin, never argv, files, errors or persistent logs.
use super::config::{endpoint, ConnectionDraft, Parameters, Provider};
use crate::{
    domain::digest,
    inspection::{Cancellation, ReadOptions},
    Error, ErrorKind, Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    process::Command,
    time::{Duration, Instant},
};
const MAX_RESPONSE: usize = 1024 * 1024;
const MAX_REQUEST: usize = 2 * 1024 * 1024;
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AiPurpose")]
pub enum Purpose {
    Discover,
    Inference,
}
/// Frozen input. Constructing/inspecting this value cannot contact a provider.
pub struct Plan {
    provider: Provider,
    endpoint: String,
    route: String,
    timeout: Duration,
    body: Option<Vec<u8>>,
    fingerprint: String,
    purpose: Purpose,
    model: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiProviderReview")]
pub struct Review {
    pub endpoint: String,
    pub route: String,
    pub unencrypted: bool,
    pub purpose: Purpose,
    pub request_bytes: usize,
    pub fingerprint: String,
    pub model: Option<String>,
}
/// Native-confirmed, exact, one-use request. Not serializable or cloneable.
pub(crate) struct Approved {
    plan: Plan,
    at: Instant,
}
/// Ephemeral auth material from the native secure-store adapter; never serialized.
pub struct Secret(Vec<u8>);
impl Secret {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.is_empty()
            || bytes.len() > 4096
            || !bytes
                .iter()
                .all(|b| b.is_ascii_graphic() && *b != b'"' && *b != b'\\')
        {
            return Err(invalid(
                "AI bearer token must be bounded printable ASCII without quotes or backslashes",
            ));
        }
        Ok(Self(bytes))
    }
}
impl Drop for Secret {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiInstalledModel")]
pub struct InstalledModel {
    pub id: String,
    pub size: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiCompletion")]
pub struct Completion {
    pub model: String,
    pub content: String,
    pub elapsed_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[schemars(rename = "AiProviderResult")]
pub enum ProviderResult {
    Models(Vec<InstalledModel>),
    Completion(Completion),
}
fn model_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value.chars().any(|c| c.is_control() || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
    {
        return Err(invalid("An explicit bounded model identifier is required"));
    }
    Ok(())
}
impl Plan {
    fn new(
        connection: &ConnectionDraft,
        purpose: Purpose,
        body: Option<Vec<u8>>,
        model: Option<String>,
    ) -> Result<Self> {
        if !(100..=120000).contains(&connection.timeout_ms) {
            return Err(invalid("AI timeout must be 100–120000 milliseconds"));
        }
        let endpoint = endpoint(&connection.endpoint)?;
        let url = url::Url::parse(&endpoint).map_err(|_| invalid("Invalid AI endpoint"))?;
        let (prefix, leaf) = match (connection.provider, purpose) {
            (Provider::Ollama, Purpose::Discover) => ("api/", "tags"),
            (Provider::Ollama, Purpose::Inference) => ("api/", "chat"),
            (Provider::Compatible, Purpose::Discover) => ("v1/", "models"),
            (Provider::Compatible, Purpose::Inference) => ("v1/", "chat/completions"),
        };
        let path = if url.path().ends_with(&format!("/{prefix}")) {
            leaf.to_owned()
        } else {
            format!("{prefix}{leaf}")
        };
        let route = url
            .join(&path)
            .map_err(|_| invalid("Invalid AI route"))?
            .to_string();
        if body.as_ref().is_some_and(|b| b.len() > MAX_REQUEST) {
            return Err(invalid("AI request exceeds 2 MiB"));
        }
        let fingerprint = digest(&serde_json::to_vec(&(
            connection.provider,
            &endpoint,
            &route,
            connection.timeout_ms,
            &body,
            &model,
            purpose,
        ))?);
        Ok(Self {
            provider: connection.provider,
            endpoint,
            route,
            timeout: Duration::from_millis(connection.timeout_ms),
            body,
            fingerprint,
            purpose,
            model,
        })
    }
    pub fn discover(connection: &ConnectionDraft) -> Result<Self> {
        Self::new(connection, Purpose::Discover, None, None)
    }
    pub fn inference(
        connection: &ConnectionDraft,
        model: &str,
        system: &str,
        user: &str,
        parameters: &Parameters,
    ) -> Result<Self> {
        model_id(model)?;
        parameters.validate(connection.provider)?;
        if system.len() > 65536
            || user.len() > 1024 * 1024
            || system.contains('\0')
            || user.contains('\0')
        {
            return Err(invalid("AI input exceeds supported bounds"));
        }
        let mut body = json!({"model":model,"messages":[{"role":"system","content":system},{"role":"user","content":user}],"stream":false});
        let serialized = serde_json::to_value(parameters)?;
        let mut options = serde_json::Map::new();
        for (key, value) in serialized
            .as_object()
            .ok_or_else(|| invalid("Invalid AI parameters"))?
        {
            if value.is_null() {
                continue;
            }
            let key = match (connection.provider, key.as_str()) {
                (Provider::Ollama, "context_size") => "num_ctx",
                (Provider::Ollama, "output_tokens") => "num_predict",
                (Provider::Ollama, "repetition_penalty") => "repeat_penalty",
                (Provider::Compatible, "output_tokens") => "max_tokens",
                (_, k) => k,
            };
            options.insert(key.into(), value.clone());
        }
        if connection.provider == Provider::Ollama {
            body["format"] = json!("json");
            if !options.is_empty() {
                body["options"] = Value::Object(options);
            }
        } else {
            body["response_format"] = json!({"type":"json_object"});
            for (key, value) in options {
                body[key] = value;
            }
        }
        Self::new(
            connection,
            Purpose::Inference,
            Some(serde_json::to_vec(&body)?),
            Some(model.into()),
        )
    }
    pub fn review(&self) -> Review {
        Review {
            endpoint: self.endpoint.clone(),
            route: self.route.clone(),
            unencrypted: self.endpoint.starts_with("http:"),
            purpose: self.purpose,
            request_bytes: self.body.as_ref().map_or(0, Vec::len),
            fingerprint: self.fingerprint.clone(),
            model: self.model.clone(),
        }
    }
    pub fn payload(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }
    /// Only the trusted Core native-review controller calls this after exact approval.
    pub(crate) fn approve(self) -> Approved {
        Approved {
            plan: self,
            at: Instant::now(),
        }
    }
}
fn config_value(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 2);
    out.push(b'"');
    for b in bytes {
        match b {
            b'\\' | b'"' => {
                out.push(b'\\');
                out.push(*b);
            }
            b'\n' => out.extend_from_slice(b"\\n"),
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\t' => out.extend_from_slice(b"\\t"),
            b => out.push(*b),
        }
    }
    out.push(b'"');
    out
}
fn command(plan: &Plan, secret: Option<&Secret>) -> Result<(Command, Vec<u8>)> {
    let mut command = Command::new("/usr/bin/curl");
    command.env_clear().env("LC_ALL", "C").current_dir("/");
    // -q must be first: never consult curlrc, proxies, netrc or inherited credentials.
    command
        .args([
            "-q",
            "--config",
            "-",
            "--silent",
            "--show-error",
            "--globoff",
            "--noproxy",
            "*",
            "--proto",
            if plan.endpoint.starts_with("https:") {
                "=https"
            } else {
                "=http"
            },
            "--max-redirs",
            "0",
            "--connect-timeout",
            "5",
            "--max-time",
        ])
        .arg(format!("{:.3}", plan.timeout.as_secs_f64()))
        .args([
            "--max-filesize",
            &MAX_RESPONSE.to_string(),
            "--write-out",
            "\nGITFLEET_HTTP:%{http_code}",
            "--request",
            if plan.body.is_some() { "POST" } else { "GET" },
            "--url",
            &plan.route,
            "--header",
            "Content-Type: application/json",
        ]);
    let mut input = Vec::new();
    if let Some(secret) = secret {
        input.extend_from_slice(b"header = ");
        let mut header = b"Authorization: Bearer ".to_vec();
        header.extend_from_slice(&secret.0);
        input.extend(config_value(&header));
        header.fill(0);
        input.push(b'\n');
    }
    if let Some(body) = &plan.body {
        input.extend_from_slice(b"data-binary = ");
        input.extend(config_value(body));
        input.push(b'\n');
    }
    Ok((command, input))
}
impl Approved {
    pub(crate) fn review(&self) -> Review {
        self.plan.review()
    }
    pub(crate) fn execute(
        self,
        secret: Option<Secret>,
        cancel: Cancellation,
    ) -> Result<ProviderResult> {
        if self.at.elapsed() > Duration::from_secs(120) {
            return Err(Error::with_kind(
                ErrorKind::StaleReview,
                "AI destination approval expired",
            ));
        }
        let remaining = Duration::from_secs(120).saturating_sub(self.at.elapsed());
        let plan = self.plan;
        let (command, input) = command(&plan, secret.as_ref())?;
        let options = ReadOptions {
            timeout: plan.timeout.min(remaining),
            max_output: MAX_RESPONSE + 64,
            cancellation: cancel,
        };
        let start = Instant::now();
        let output =
            crate::process::bounded_input(command, &options, Some(input)).map_err(|e| {
                Error::with_kind(
                    e.kind,
                    "AI request did not complete; manual workflows remain available",
                )
            })?;
        if !output.status.success() {
            let kind = match output.status.code() {
                Some(28) => ErrorKind::Timeout,
                Some(63) => ErrorKind::OutputLimit,
                _ => ErrorKind::Unavailable,
            };
            return Err(Error::with_kind(kind,"AI provider connection failed; inspect endpoint, model and authentication, retry or cancel"));
        }
        let separator = b"\nGITFLEET_HTTP:";
        let at = output
            .stdout
            .windows(separator.len())
            .rposition(|w| w == separator)
            .ok_or_else(|| invalid("AI transport returned no HTTP status"))?;
        let status = std::str::from_utf8(&output.stdout[at + separator.len()..])
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| invalid("AI transport returned an invalid status"))?;
        if status != 200 {
            return Err(Error::with_kind(
                ErrorKind::Unavailable,
                match status {
                    300..=399 => "AI redirect refused; approve the intended endpoint explicitly",
                    401 | 403 => "AI authentication refused; review the selected connection",
                    404 => "AI endpoint or model is unavailable; no fallback selected",
                    _ => "AI provider rejected the request; no fallback selected",
                },
            ));
        }
        if at > MAX_RESPONSE {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "AI response exceeds 1 MiB",
            ));
        }
        let value: Value = serde_json::from_slice(&output.stdout[..at])
            .map_err(|_| invalid("AI provider returned malformed JSON"))?;
        match plan.purpose {
            Purpose::Discover => Ok(ProviderResult::Models(models(plan.provider, &value)?)),
            Purpose::Inference => Ok(ProviderResult::Completion(completion(
                plan.provider,
                &value,
                plan.model.as_deref().unwrap_or_default(),
                start.elapsed(),
            )?)),
        }
    }
}
fn models(provider: Provider, value: &Value) -> Result<Vec<InstalledModel>> {
    let (array, key) = if provider == Provider::Ollama {
        ("models", "name")
    } else {
        ("data", "id")
    };
    let values = value
        .get(array)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("AI provider returned no model list"))?;
    if values.len() > 2048 {
        return Err(invalid("AI model list exceeds 2048 entries"));
    }
    let mut models = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for value in values {
        let id = value
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("AI provider returned an invalid model"))?;
        model_id(id)?;
        if ids.insert(id) {
            models.push(InstalledModel {
                id: id.into(),
                size: value.get("size").and_then(Value::as_u64),
            });
        }
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}
fn completion(
    provider: Provider,
    value: &Value,
    expected_model: &str,
    elapsed: Duration,
) -> Result<Completion> {
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("AI provider omitted its model identifier"))?;
    model_id(model)?;
    if model != expected_model {
        return Err(invalid(
            "AI provider returned a different model; no fallback accepted",
        ));
    }
    let (message, input_tokens, output_tokens) = if provider == Provider::Ollama {
        if value.get("done") != Some(&json!(true))
            || value.get("done_reason").is_some_and(|v| v != "stop")
        {
            return Err(invalid("AI inference did not finish normally"));
        }
        (
            &value["message"],
            value.get("prompt_eval_count").and_then(Value::as_u64),
            value.get("eval_count").and_then(Value::as_u64),
        )
    } else {
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("AI inference omitted choices"))?;
        if choices.len() != 1 || choices[0].get("finish_reason") != Some(&json!("stop")) {
            return Err(invalid("AI inference was incomplete or ambiguous"));
        }
        (
            &choices[0]["message"],
            value["usage"]["prompt_tokens"].as_u64(),
            value["usage"]["completion_tokens"].as_u64(),
        )
    };
    if message.get("role") != Some(&json!("assistant"))
        || message
            .get("tool_calls")
            .is_some_and(|v| !v.is_null() && !v.as_array().is_some_and(Vec::is_empty))
        || message.get("function_call").is_some_and(|v| !v.is_null())
    {
        return Err(invalid("AI tools and function calls are not permitted"));
    }
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("AI inference returned no message"))?;
    if content.is_empty() || content.len() > 65536 || content.contains('\0') {
        return Err(invalid("AI inference text exceeds limits or is empty"));
    }
    Ok(Completion {
        model: model.into(),
        content: content.into(),
        elapsed_ms: elapsed.as_millis().min(u64::MAX as u128) as u64,
        input_tokens,
        output_tokens,
    })
}
#[cfg(test)]
mod tests;
