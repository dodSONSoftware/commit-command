//! No network or managed writes. A past observation is data, never execution approval.
use super::*;
use crate::domain::ObjectId;
use crate::remote::{push::*, Transport};
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Push preview context changed; inspect and observe again",
    )
}
pub(super) fn remaining(options: &ReadOptions, start: Instant) -> Result<ReadOptions> {
    options.check(start)?;
    let mut next = options.clone();
    next.timeout = next.timeout.saturating_sub(start.elapsed());
    Ok(next)
}
pub(super) fn complete_history(s: &inspection::Session<'_>) -> Result<()> {
    for path in [
        s.common_dir.join("shallow"),
        s.common_dir.join("info/grafts"),
    ] {
        match fs::symlink_metadata(path) {
            Ok(_) => {
                return Err(unsupported(
                    "Shallow or grafted ancestry cannot establish a push comparison",
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn source_gitlinks(s: &inspection::Session<'_>, oid: &ObjectId) -> Result<Vec<SourceGitlink>> {
    // Read the committed tree, never .gitmodules URLs or child worktrees. Git does
    // not descend into gitlinks, including uninitialized or absent children.
    let tree = s.git(&["ls-tree", "-r", "-z", "--full-tree", &oid.hex, "--"])?;
    let mut links = Vec::new();
    for record in tree.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let tab = record
            .iter()
            .position(|b| *b == b'\t')
            .ok_or_else(|| fail(ErrorKind::InvalidData, "Invalid committed tree entry"))?;
        let header = std::str::from_utf8(&record[..tab])
            .map_err(|_| fail(ErrorKind::InvalidData, "Invalid committed tree header"))?;
        let fields = header.split(' ').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(fail(
                ErrorKind::InvalidData,
                "Invalid committed tree header",
            ));
        }
        if fields[0] != "160000" {
            continue;
        }
        if fields[1] != "commit" || record.len() == tab + 1 {
            return Err(fail(ErrorKind::InvalidData, "Invalid committed gitlink"));
        }
        if links.len() == 1024 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Source exceeds 1,024 gitlinks",
            ));
        }
        links.push(SourceGitlink {
            path: record[tab + 1..].to_vec(),
            oid: ObjectId::parse(fields[2], &s.format)?,
        });
    }
    Ok(links)
}
fn outgoing_gitlinks(
    s: &inspection::Session<'_>,
    range: &str,
    expected_count: u32,
    base: &str,
) -> Result<Vec<OutgoingGitlink>> {
    use std::collections::{BTreeMap, BTreeSet};
    // This list is independent of the 100-commit presentation limit and includes
    // all merge parents reachable from source but not the observed destination.
    let list = inspection::output_text(s.git(&[
        "rev-list",
        "--topo-order",
        "--no-commit-header",
        "--format=%H %T",
        "--max-count=10001",
        range,
        "--",
    ])?)?;
    let mut trees = BTreeSet::new();
    let mut requirements = BTreeMap::new();
    let mut count = 0;
    let mut mapping_count = 0;
    let mut detail_bytes = 0;
    for line in list.lines() {
        let (commit, tree) = line
            .split_once(' ')
            .ok_or_else(|| fail(ErrorKind::InvalidData, "Invalid outgoing tree listing"))?;
        let commit = ObjectId::parse(commit, &s.format)?;
        let tree = ObjectId::parse(tree, &s.format)?;
        count += 1;
        if !trees.insert(tree.hex.clone()) {
            continue;
        }
        if trees.len() > 256 {
            return Err(fail(
                ErrorKind::OutputLimit,
                "Outgoing history exceeds 256 distinct trees",
            ));
        }
        let links = source_gitlinks(s, &tree)?;
        if links.is_empty() {
            continue;
        }
        let mappings = super::push_mappings::MappingFile::read(s, &tree)?;
        for link in links {
            let key = (link.path.clone(), link.oid.hex.clone());
            let requirement_count = requirements.len();
            if !requirements.contains_key(&key) {
                detail_bytes += link.path.len();
                if detail_bytes > 2 * 1024 * 1024 {
                    return Err(fail(
                        ErrorKind::OutputLimit,
                        "Outgoing dependency details exceed 2 MiB",
                    ));
                }
            }
            if requirement_count == 1024 && !requirements.contains_key(&key) {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Outgoing history exceeds 1,024 child requirements",
                ));
            }
            let mapping = mappings.mapping(&link.path, &commit, base);
            let entry = requirements.entry(key).or_insert_with(|| OutgoingGitlink {
                path: link.path,
                oid: link.oid,
                example_commit: commit.clone(),
                mappings: Vec::new(),
            });
            if !entry
                .mappings
                .iter()
                .any(|m| m.gitmodules_blob == mapping.gitmodules_blob)
            {
                mapping_count += 1;
                if mapping_count > 4096 {
                    return Err(fail(
                        ErrorKind::OutputLimit,
                        "Outgoing history exceeds 4,096 mapping contexts",
                    ));
                }
                detail_bytes += mapping.declared_url.as_ref().map_or(0, String::len)
                    + mapping.resolution_base.as_ref().map_or(0, String::len)
                    + mapping.destination.as_ref().map_or(0, |d| d.display.len());
                if detail_bytes > 2 * 1024 * 1024 {
                    return Err(fail(
                        ErrorKind::OutputLimit,
                        "Outgoing dependency details exceed 2 MiB",
                    ));
                }
                entry.mappings.push(mapping);
            }
        }
    }
    if count != expected_count {
        return Err(stale());
    }
    Ok(requirements.into_values().collect())
}
impl ControlledWrites {
    pub(crate) fn push_preview(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        observation: &str,
        options: &ReadOptions,
    ) -> Result<PushPreview> {
        let previous = std::mem::replace(&mut self.options.process, options.clone());
        let result = self.push_preview_inner(workspace, entry, observation, options);
        self.options.process = previous;
        result
    }
    fn push_preview_inner(
        &mut self,
        workspace: &WorkspaceId,
        entry: &crate::workspace::StoredWorktree,
        id: &str,
        options: &ReadOptions,
    ) -> Result<PushPreview> {
        let started = Instant::now();
        let observation = self.completed_observation(id, workspace, &entry.id)?;
        let observed = observation.oid.ok_or_else(stale)?;
        let old = &observation.plan;
        let current = self.prepare_transport_plan(
            workspace,
            entry,
            &old.remote,
            &old.reference,
            &remaining(options, started)?,
        )?;
        // Import can legitimately change local refs after observation. Bind current
        // local state separately; retain the original observed OID/time explicitly.
        if old.config_digest != current.config_digest
            || old.destination != current.destination
            || old.metadata.root_binding != current.metadata.root_binding
            || old.metadata.git_binding != current.metadata.git_binding
            || old.metadata.common_binding != current.metadata.common_binding
        {
            return Err(stale());
        }
        let report = crate::remote::inspect(
            &entry.metadata.root.to_path()?,
            self.options.global_config.as_deref(),
            &remaining(options, started)?,
        )?;
        if report.config_digest != current.config_digest || report.generation != current.generation
        {
            return Err(stale());
        }
        let remote = report
            .remotes
            .iter()
            .find(|r| r.name == current.remote)
            .ok_or_else(stale)?;
        if remote.push.len() != 1
            || remote.push_refspec_count != 0
            || remote.push[0].display != current.destination
            || remote.push[0].credentials_hidden
            || !remote.push[0].restrictions.is_empty()
            || !matches!(
                remote.push[0].transport,
                Transport::Https | Transport::Local | Transport::Ssh
            )
        {
            return Err(unsupported("Local push preview requires one identical fetch/push destination and no configured push refspecs"));
        }
        let source_ref = report
            .branch
            .ok_or_else(|| unsupported("Push preview requires a checked-out local branch"))?;
        let mut read = remaining(options, started)?;
        read.max_output = read.max_output.min(2 * 1024 * 1024);
        let preview = inspection::read(&entry.metadata.root.to_path()?, &read, |s| {
            if s.generation != current.generation {
                return Err(stale());
            }
            complete_history(s)?;
            let source_oid = s
                .head()?
                .ok_or_else(|| unsupported("Push preview requires an existing local commit"))?;
            if observed.format != s.format {
                return Err(stale());
            }
            // Missing objects never trigger lazy fetching; explicit fetch is separate.
            crate::history::commit_in(s, &observed)?;
            let range = format!("{}...{}", observed.hex, source_oid.hex);
            let counts = inspection::output_text(s.git(&[
                "rev-list",
                "--left-right",
                "--count",
                "--max-count=10001",
                &range,
                "--",
            ])?)?;
            let fields = counts.split_whitespace().collect::<Vec<_>>();
            if fields.len() != 2 {
                return Err(fail(ErrorKind::InvalidData, "Invalid ancestry comparison"));
            }
            let incoming: u32 = fields[0]
                .parse()
                .map_err(|_| fail(ErrorKind::InvalidData, "Invalid ancestry count"))?;
            let outgoing: u32 = fields[1]
                .parse()
                .map_err(|_| fail(ErrorKind::InvalidData, "Invalid ancestry count"))?;
            if u64::from(incoming) + u64::from(outgoing) > 10_000 {
                return Err(fail(
                    ErrorKind::OutputLimit,
                    "Push comparison exceeds 10,000 commits",
                ));
            }
            let relation = match (incoming, outgoing) {
                (0, 0) => PushRelation::UpToDate,
                (0, _) => PushRelation::FastForward,
                (_, 0) => PushRelation::Behind,
                _ => PushRelation::Diverged,
            };
            let range = format!("{}..{}", observed.hex, source_oid.hex);
            let list = inspection::output_text(s.git(&[
                "rev-list",
                "--topo-order",
                "--max-count=100",
                &range,
                "--",
            ])?)?;
            let mut commits = Vec::new();
            for line in list.lines().filter(|line| !line.is_empty()) {
                let oid = crate::domain::ObjectId::parse(line, &s.format)?;
                let commit = crate::history::commit_in(s, &oid)?;
                let subject = commit
                    .message
                    .split(|b| *b == b'\n')
                    .next()
                    .unwrap_or_default();
                commits.push(OutgoingCommit {
                    oid,
                    subject: subject[..subject.len().min(4096)].to_vec(),
                    subject_truncated: subject.len() > 4096,
                });
            }
            complete_history(s)?;
            if commits.len() != outgoing.min(100) as usize {
                return Err(stale());
            }
            let source_gitlinks = source_gitlinks(s, &source_oid)?;
            let outgoing_gitlinks = outgoing_gitlinks(s, &range, outgoing, &current.destination)?;
            Ok(PushPreview {
                source_ref: source_ref.clone(),
                source_oid,
                remote: current.remote.clone(),
                destination: current.destination.clone(),
                destination_ref: current.reference.clone(),
                observed_oid: observed.clone(),
                observation_id: id.into(),
                observed_at_ms: observation.observed_at_ms.ok_or_else(stale)?,
                generated_at_ms: now_ms(),
                generation: current.generation.clone(),
                config_digest: current.config_digest.clone(),
                relation,
                outgoing_count: outgoing,
                incoming_count: incoming,
                outgoing_truncated: outgoing > commits.len() as u32,
                outgoing: commits,
                source_gitlinks,
                outgoing_gitlinks,
                upstream_matches: report.upstream_remote.as_deref()
                    == Some(current.remote.as_str())
                    && report.upstream_ref.as_deref() == Some(current.reference.as_str()),
            })
        })?;
        self.revalidate_transport(&current, &remaining(options, started)?)?;
        Ok(preview)
    }
}
