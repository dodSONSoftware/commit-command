//! Committed declarations are data, never transport or credential authority.
use super::*;
use crate::domain::ObjectId;
use crate::remote::{push::HistoricalMapping, Destination, Transport};
use std::collections::BTreeMap;
#[derive(Default)]
struct Module {
    paths: Vec<Vec<u8>>,
    urls: Vec<Vec<u8>>,
}
pub(super) struct MappingFile {
    blob: Option<ObjectId>,
    modules: BTreeMap<Vec<u8>, Module>,
    issue: Option<String>,
}
impl MappingFile {
    pub(super) fn read(s: &inspection::Session<'_>, tree: &ObjectId) -> Result<Self> {
        let mut result = Self {
            blob: None,
            modules: BTreeMap::new(),
            issue: None,
        };
        let entry = s.git(&["ls-tree", "-z", &tree.hex, "--", ".gitmodules"])?;
        if entry.is_empty() {
            result.issue = Some("No committed .gitmodules mapping".into());
            return Ok(result);
        }
        let tab = entry
            .iter()
            .position(|b| *b == b'\t')
            .ok_or_else(|| inspection::data_error("Invalid historical mapping tree entry"))?;
        let fields = std::str::from_utf8(&entry[..tab])
            .map_err(|_| inspection::data_error("Invalid historical mapping header"))?
            .split(' ')
            .collect::<Vec<_>>();
        if fields.len() != 3 || &entry[tab + 1..] != b".gitmodules\0" {
            return Err(inspection::data_error(
                "Invalid historical mapping tree entry",
            ));
        }
        result.blob = Some(ObjectId::parse(fields[2], &s.format)?);
        if !matches!(fields[0], "100644" | "100755") || fields[1] != "blob" {
            result.issue = Some("Historical .gitmodules is not a regular blob".into());
            return Ok(result);
        }
        let args = [
            "config",
            "--null",
            "--no-includes",
            "--blob",
            fields[2],
            "--list",
        ];
        let output = s.output(
            &args
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>(),
        )?;
        if !output.status.success() {
            result.issue = Some("Historical .gitmodules cannot be parsed".into());
            return Ok(result);
        }
        let mut count = 0;
        for record in output.stdout.split(|b| *b == 0).filter(|v| !v.is_empty()) {
            count += 1;
            if count > 8192 {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Historical mapping exceeds 8,192 fields",
                ));
            }
            let newline = record
                .iter()
                .position(|b| *b == b'\n')
                .unwrap_or(record.len());
            let value = record.get(newline + 1..).unwrap_or_default();
            let Some(key) = record[..newline].strip_prefix(b"submodule.") else {
                continue;
            };
            let (name, path) = if let Some(name) = key.strip_suffix(b".path") {
                (name, true)
            } else if let Some(name) = key.strip_suffix(b".url") {
                (name, false)
            } else {
                continue;
            };
            let module = result.modules.entry(name.to_vec()).or_default();
            if path {
                module.paths.push(value.to_vec());
            } else {
                module.urls.push(value.to_vec());
            }
        }
        Ok(result)
    }
    pub(super) fn mapping(&self, path: &[u8], commit: &ObjectId, base: &str) -> HistoricalMapping {
        let mut result = HistoricalMapping {
            declared_url: None,
            resolution_base: None,
            gitmodules_blob: self.blob.clone(),
            example_commit: commit.clone(),
            destination: None,
            issue: self.issue.clone(),
        };
        if result.issue.is_some() {
            return result;
        }
        let matches = self
            .modules
            .values()
            .filter(|m| m.paths.iter().any(|p| p == path))
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].paths.len() != 1 || matches[0].urls.len() != 1 {
            result.issue =
                Some("Historical child path requires one unambiguous module and URL".into());
            return result;
        }
        let Ok(url) = std::str::from_utf8(&matches[0].urls[0]) else {
            result.issue = Some("Historical child URL requires UTF-8".into());
            return result;
        };
        if url.len() > 4096 {
            result.issue = Some("Historical child URL exceeds 4,096 bytes; value withheld".into());
            return result;
        }
        let relative = url.starts_with("./") || url.starts_with("../");
        let resolved = if relative {
            let Some(value) = crate::remote::urls::relative_url(base, url) else {
                result.issue = Some("Relative child URL cannot be resolved safely against the selected parent destination".into());
                return result;
            };
            result.resolution_base = Some(base.into());
            value
        } else {
            url.into()
        };
        if resolved.len() > 4096 {
            result.issue = Some("Resolved child URL exceeds 4,096 bytes".into());
            return result;
        }
        let destination: Destination = crate::remote::destination(&resolved);
        if destination.credentials_hidden
            || !destination.restrictions.is_empty()
            || !matches!(
                destination.transport,
                Transport::Https | Transport::Ssh | Transport::Local
            )
            || (!resolved.contains("://")
                && !Path::new(&resolved).is_absolute()
                && crate::remote::urls::scp_parts(&resolved).is_none())
        {
            result.issue = Some("Historical child URL requires a supported destination without credentials or ambiguous path spelling".into());
        } else {
            result.declared_url = Some(url.into());
        }
        result.destination = Some(destination);
        result
    }
}
