//! Restricted, read-only source capture. No trust grant, provider request or write proposal.
use super::context::{self, Bundle, Fragment, Kind, Policy};
use crate::{
    domain::{digest, NativePath, ObjectId},
    inspection::{self, ReadOptions, Session},
    writes::{ContentSource, SelectedPath},
    Error, ErrorKind, Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn invalid() -> Error {
    Error::with_kind(
        ErrorKind::InvalidRequest,
        "Invalid, duplicate or unsupported AI context selection",
    )
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "AiCaptureSpec")]
pub struct Spec {
    pub selected: Vec<SelectedPath>,
    /// Explicit related tracked regular source files, not a recursive source scan.
    pub nearby_source: Vec<NativePath>,
    pub documentation: Vec<NativePath>,
    pub branch_commits: usize,
    pub file_commits: usize,
    pub policy: Policy,
}
impl Spec {
    pub fn validate(&self) -> Result<()> {
        self.policy.validate()?;
        if self.selected.is_empty()
            || self.selected.len() > 10_000
            || self.nearby_source.len() > 8
            || self.documentation.len() > 8
            || self.branch_commits > 20
            || self.file_commits > 10
        {
            return Err(invalid());
        }
        let mut paths = BTreeSet::new();
        for path in self
            .selected
            .iter()
            .map(|s| &s.path)
            .chain(&self.nearby_source)
            .chain(&self.documentation)
        {
            let relative = path.relative()?;
            let label = relative.to_str().ok_or_else(invalid)?;
            if label.len() > 1024 || label.chars().any(char::is_control) || !paths.insert(relative)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// Captured content is ephemeral; the hash covers even excluded/redacted source.
/// Deserializing this display value cannot create a provider/write capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AiCapturedContext")]
pub struct Captured {
    pub bundle: Bundle,
    pub snapshot_fingerprint: String,
    pub generation: String,
}
fn index_blob(s: &Session<'_>, path: &NativePath) -> Result<Option<(String, ObjectId)>> {
    let bytes = s.git_os(&[
        "ls-files".into(),
        "--stage".into(),
        "-z".into(),
        "--".into(),
        path.relative()?.into_os_string(),
    ])?;
    let records = bytes
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
        .collect::<Vec<_>>();
    if records.is_empty() {
        return Ok(None);
    }
    if records.len() != 1 {
        return Err(invalid());
    }
    let (metadata, actual) = records[0].split_at(
        records[0]
            .iter()
            .position(|b| *b == b'\t')
            .ok_or_else(invalid)?,
    );
    if NativePath::from_git(&actual[1..])?.relative()? != path.relative()? {
        return Err(invalid());
    }
    let fields = std::str::from_utf8(metadata)
        .map_err(|_| invalid())?
        .split(' ')
        .collect::<Vec<_>>();
    if fields.len() != 3 || fields[2] != "0" || !["100644", "100755"].contains(&fields[0]) {
        return Err(invalid());
    }
    Ok(Some((
        fields[0].into(),
        ObjectId::parse(fields[1], &s.format)?,
    )))
}
fn push(
    fragments: &mut Vec<Fragment>,
    kind: Kind,
    path: &NativePath,
    bytes: Vec<u8>,
) -> Result<()> {
    if fragments.iter().map(|f| f.bytes.len()).sum::<usize>() + bytes.len()
        > context::MAX_CAPTURE_BYTES
    {
        return Err(Error::with_kind(
            ErrorKind::OutputLimit,
            "AI capture exceeds aggregate source budget",
        ));
    }
    fragments.push(Fragment {
        kind,
        origin: path.relative()?.to_str().ok_or_else(invalid)?.into(),
        bytes,
    });
    Ok(())
}
fn history(
    s: &Session<'_>,
    head: &ObjectId,
    path: Option<&NativePath>,
    count: usize,
) -> Result<Vec<u8>> {
    if count == 0 {
        return Ok(vec![]);
    }
    let mut args = vec![
        "rev-list".into(),
        format!("--max-count={count}").into(),
        head.hex.clone().into(),
        "--".into(),
    ];
    if let Some(path) = path {
        args.push(path.relative()?.into_os_string());
    }
    let ids = s.git_os(&args)?;
    let mut result = Vec::new();
    for id in ids.split(|b| *b == b'\n').filter(|id| !id.is_empty()) {
        let id = ObjectId::parse(std::str::from_utf8(id).map_err(|_| invalid())?, &s.format)?;
        let commit = crate::history::commit_in(s, &id)?;
        if result.len() + commit.message.len() > 1024 * 1024 {
            return Err(Error::with_kind(
                ErrorKind::OutputLimit,
                "AI history capture exceeds byte bound",
            ));
        }
        result.extend_from_slice(format!("Commit {}\n", id.hex).as_bytes());
        result.extend_from_slice(&commit.message);
        result.push(b'\n');
    }
    Ok(result)
}
fn head_blob(
    s: &Session<'_>,
    head: Option<&ObjectId>,
    path: &NativePath,
) -> Result<Option<Vec<u8>>> {
    let Some(head) = head else { return Ok(None) };
    let records = s.git_os(&[
        "ls-tree".into(),
        "-z".into(),
        head.hex.clone().into(),
        "--".into(),
        path.relative()?.into_os_string(),
    ])?;
    if records.is_empty() {
        return Ok(None);
    }
    let tab = records
        .iter()
        .position(|b| *b == b'\t')
        .ok_or_else(invalid)?;
    let fields = std::str::from_utf8(&records[..tab])
        .map_err(|_| invalid())?
        .split(' ')
        .collect::<Vec<_>>();
    if fields.len() != 3 || fields[1] != "blob" {
        return Err(invalid());
    }
    s.blob(&ObjectId::parse(fields[2], &s.format)?).map(Some)
}
#[derive(Default)]
struct Comparison {
    patch: Vec<u8>,
    private_patch_lines: Vec<bool>,
    private_content_lines: Vec<bool>,
}

fn whole_file_diff(old: &[u8], new: &[u8]) -> Comparison {
    if old == new {
        return Comparison::default();
    }
    let binary = || Comparison {
        patch: b"Binary comparison omitted\0".to_vec(),
        ..Default::default()
    };
    let (Ok(old), Ok(new)) = (std::str::from_utf8(old), std::str::from_utf8(new)) else {
        return binary();
    };
    if old.contains('\0') || new.contains('\0') {
        return binary();
    }
    let old = old.split_inclusive('\n').collect::<Vec<_>>();
    let new = new.split_inclusive('\n').collect::<Vec<_>>();
    let classify = |lines: &[&str]| {
        let mut inside = false;
        lines
            .iter()
            .map(|line| context::private_key_line(line, &mut inside))
            .collect::<Vec<_>>()
    };
    let mut old_private = classify(&old);
    let mut new_private = classify(&new);
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    // A delimiter edit can expose a previously protected line in the opposite
    // version, even when that line has moved outside the common context.
    let protected: std::collections::HashSet<_> = old
        .iter()
        .zip(&old_private)
        .chain(new.iter().zip(&new_private))
        .filter(|(_, private)| **private)
        .map(|(line, _)| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    if !protected.is_empty() {
        for (line, flag) in old
            .iter()
            .zip(&mut old_private)
            .chain(new.iter().zip(&mut new_private))
        {
            *flag |= protected.contains(line.trim());
        }
    }
    let start = prefix.saturating_sub(3);
    let old_end = (old.len() - suffix + 3).min(old.len());
    let new_end = (new.len() - suffix + 3).min(new.len());
    let mut result = format!(
        "Comparison: HEAD -> chosen content\n@@ -{},{} +{},{} @@\n",
        start + 1,
        old_end - start,
        start + 1,
        new_end - start
    );
    let mut private_patch_lines = vec![false; 2]; // comparison and hunk headers
    for (prefix, lines, private) in [
        (" ", &new[start..prefix], &new_private[start..prefix]),
        (
            "-",
            &old[prefix..old.len() - suffix],
            &old_private[prefix..old.len() - suffix],
        ),
        (
            "+",
            &new[prefix..new.len() - suffix],
            &new_private[prefix..new.len() - suffix],
        ),
        (
            " ",
            &new[new.len() - suffix..new_end],
            &new_private[new.len() - suffix..new_end],
        ),
    ] {
        for (line, &protected) in lines.iter().zip(private) {
            private_patch_lines.push(protected);
            result.push_str(prefix);
            result.push_str(line);
            if !line.ends_with('\n') {
                result.push_str("\n\\ No newline at end of file\n");
                private_patch_lines.push(false);
            }
        }
    }
    Comparison {
        patch: result.into_bytes(),
        private_patch_lines,
        private_content_lines: new_private,
    }
}

pub fn capture(repo: &Path, spec: &Spec, options: &ReadOptions) -> Result<Captured> {
    spec.validate()?;
    let start = std::time::Instant::now();
    let mut bounded = options.clone();
    bounded.max_output = bounded.max_output.min(4 * 1024 * 1024);
    inspection::read(repo, &bounded, |s| {
        let mut fragments = Vec::new();
        let mut private_lines = BTreeMap::new();
        let mut bindings = Vec::new();
        let mut watched = Vec::new();
        let head = s.head()?;
        for selection in &spec.selected {
            let path = &selection.path;
            let index = index_blob(s, path)?;
            let old = head_blob(s, head.as_ref(), path)?;
            let (bytes, binding) = match selection.source {
                ContentSource::Index => match index.as_ref() {
                    Some((_, oid)) => (s.blob(oid)?, oid.hex.clone()),
                    None if old.is_some() => (Vec::new(), "absent:index".into()),
                    None => return Err(invalid()),
                },
                ContentSource::WorkingTree => {
                    if index.is_none() {
                        let untracked = s.git_os(&[
                            "ls-files".into(),
                            "--others".into(),
                            "--exclude-standard".into(),
                            "-z".into(),
                            "--".into(),
                            path.relative()?.into_os_string(),
                        ])?;
                        let mut expected = path.relative()?.as_os_str().as_encoded_bytes().to_vec();
                        expected.push(0);
                        if untracked != expected {
                            return Err(invalid());
                        }
                    }
                    let value = super::capture_file::read(&s.root, path, 4 * 1024 * 1024, true)?;
                    watched.push((path.clone(), value.1.clone()));
                    value
                }
            };
            push(
                &mut fragments,
                Kind::SelectedMetadata,
                path,
                serde_json::to_vec(
                    &serde_json::json!({"source":selection.source,"exists":!binding.starts_with("absent:"),"index_mode":index.as_ref().map(|(mode,_)|mode)}),
                )?,
            )?;
            bindings.push(binding);
            // Build comparisons from the already captured bytes and immutable HEAD
            // blobs. A second Git worktree read could race a swapped ancestor.
            let comparison = whole_file_diff(old.as_deref().unwrap_or_default(), &bytes);
            if !comparison.patch.is_empty() {
                if comparison.private_patch_lines.contains(&true) {
                    private_lines.insert(fragments.len(), comparison.private_patch_lines);
                }
                push(&mut fragments, Kind::SelectedDiff, path, comparison.patch)?;
            }
            if comparison.private_content_lines.contains(&true) {
                private_lines.insert(fragments.len(), comparison.private_content_lines);
            }
            push(&mut fragments, Kind::SelectedContent, path, bytes)?;
        }
        for (kind, paths) in [
            (Kind::NearbySource, &spec.nearby_source),
            (Kind::Documentation, &spec.documentation),
        ] {
            for path in paths {
                index_blob(s, path)?.ok_or_else(invalid)?;
                let (bytes, binding) =
                    super::capture_file::read(&s.root, path, 4 * 1024 * 1024, false)?;
                bindings.push(binding.clone());
                watched.push((path.clone(), binding));
                push(&mut fragments, kind, path, bytes)?;
            }
        }
        if let Some(head) = head {
            if spec.branch_commits > 0 {
                push(
                    &mut fragments,
                    Kind::BranchHistory,
                    &NativePath::Utf8("current-branch-history".into()),
                    history(s, &head, None, spec.branch_commits)?,
                )?;
            }
            for selected in &spec.selected {
                if spec.file_commits > 0 {
                    push(
                        &mut fragments,
                        Kind::FileHistory,
                        &selected.path,
                        history(s, &head, Some(&selected.path), spec.file_commits)?,
                    )?;
                }
            }
        }
        for (path, binding) in watched {
            if super::capture_file::read(&s.root, &path, 4 * 1024 * 1024, true)?.1 != binding {
                return Err(Error::with_kind(
                    ErrorKind::ConcurrentChange,
                    "AI context file changed during capture",
                ));
            }
        }
        let snapshot_fingerprint = digest(&serde_json::to_vec(&(&s.generation, spec, bindings))?);
        let bundle = context::prepare_with_private_lines(fragments, &spec.policy, &private_lines)?;
        options.check(start)?;
        Ok(Captured {
            bundle,
            snapshot_fingerprint,
            generation: s.generation.clone(),
        })
    })
}
/// Re-capture raw selection, metadata and explicit related context; filtered equality
/// alone is insufficient because a secret/redacted file may also have changed.
pub fn revalidate(
    repo: &Path,
    spec: &Spec,
    expected: &Captured,
    options: &ReadOptions,
) -> Result<()> {
    let current = capture(repo, spec, options)?;
    if current.snapshot_fingerprint != expected.snapshot_fingerprint
        || current.bundle.fingerprint != expected.bundle.fingerprint
    {
        return Err(Error::with_kind(
            ErrorKind::StaleReview,
            "AI source snapshot changed; capture and approve a fresh request",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod comparison_tests {
    use super::*;
    #[test]
    fn removing_delimiters_keeps_known_key_material_protected() {
        let comparison = whole_file_diff(
            b"-----BEGIN PRIVATE KEY-----\nSyntheticBody\n-----END PRIVATE KEY-----\n",
            b"ordinary change\nSyntheticBody",
        );
        assert_eq!(comparison.private_content_lines, [false, true]);
        let patch = std::str::from_utf8(&comparison.patch).unwrap();
        assert_eq!(patch.lines().count(), comparison.private_patch_lines.len());
        for (line, protected) in patch.lines().zip(comparison.private_patch_lines) {
            assert_eq!(
                protected,
                line.contains("SyntheticBody") || line.contains("PRIVATE KEY")
            );
        }
    }
    #[test]
    fn omits_unchanged_prefix_and_suffix_from_selected_comparison() {
        let prefix = (0..100)
            .map(|i| format!("unchanged before {i}\n"))
            .collect::<String>();
        let suffix = (0..100)
            .map(|i| format!("unchanged after {i}\n"))
            .collect::<String>();
        let patch = String::from_utf8(
            whole_file_diff(
                format!("{prefix}old value\n{suffix}").as_bytes(),
                format!("{prefix}new value\n{suffix}").as_bytes(),
            )
            .patch,
        )
        .unwrap();
        assert!(patch.contains("-old value\n+new value\n"));
        assert!(!patch.contains("unchanged before 0\n"));
        assert!(!patch.contains("unchanged after 99\n"));
        assert!(patch.contains(" unchanged before 99\n"));
        assert!(patch.len() < 512);
    }
}
