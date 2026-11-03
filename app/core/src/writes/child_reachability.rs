//! Past reviewed server evidence plus local ancestry; never a new network grant.
use super::push_preview::{complete_history, remaining};
use super::*;
use crate::remote::push::*;
fn stale() -> Error {
    fail(
        ErrorKind::StaleReview,
        "Child evidence changed or is too old; explicitly observe and inspect again",
    )
}
impl ControlledWrites {
    pub(crate) fn child_reachability(
        &mut self,
        workspace: &WorkspaceId,
        parent: &crate::workspace::StoredWorktree,
        child: &crate::workspace::StoredWorktree,
        selection: ChildReachabilitySelection,
        options: &ReadOptions,
    ) -> Result<ChildReachability> {
        let started = Instant::now();
        if parent.id == child.id {
            return Err(unsupported("Select a distinct child worktree explicitly"));
        }
        let preview = self.push_preview(
            workspace,
            parent,
            &selection.parent_observation,
            &remaining(options, started)?,
        )?;
        if preview.source_oid != selection.source_oid {
            return Err(stale());
        }
        let requirement = preview
            .outgoing_gitlinks
            .iter()
            .find(|r| r.path == selection.path && r.oid == selection.child_oid)
            .ok_or_else(stale)?;
        let mapping = requirement
            .mappings
            .iter()
            .find(|m| m.gitmodules_blob.as_ref() == Some(&selection.gitmodules_blob))
            .ok_or_else(stale)?
            .clone();
        if mapping.issue.is_some() {
            return Err(unsupported(
                "Resolve unsupported historical child mapping before requesting evidence",
            ));
        }
        let destination = mapping.destination.as_ref().ok_or_else(stale)?;
        let observation =
            self.completed_observation(&selection.child_observation, workspace, &child.id)?;
        let observed_at = observation.observed_at_ms.ok_or_else(stale)?;
        let fresh = || {
            now_ms()
                .checked_sub(observed_at)
                .is_some_and(|age| age <= 300_000)
        };
        if !fresh() || destination.display != observation.plan.destination {
            return Err(stale());
        }
        let old = &observation.plan;
        let current = self.prepare_transport_plan(
            workspace,
            child,
            &old.remote,
            &old.reference,
            &remaining(options, started)?,
        )?;
        if current.config_digest != old.config_digest
            || current.destination != old.destination
            || current.transport != old.transport
            || current.authentication != old.authentication
            || current.authentication_profile != old.authentication_profile
            || current.tools != old.tools
            || current.local_binding != old.local_binding
            || current.metadata.root_binding != old.metadata.root_binding
            || current.metadata.git_binding != old.metadata.git_binding
            || current.metadata.common_binding != old.metadata.common_binding
        {
            return Err(stale());
        }
        let observed = observation.oid.ok_or_else(stale)?;
        let reachable = inspection::read(
            &child.metadata.root.to_path()?,
            &remaining(options, started)?,
            |s| {
                if s.generation != current.generation
                    || s.format != selection.child_oid.format
                    || s.format != observed.format
                {
                    return Err(stale());
                }
                complete_history(s)?;
                // Available objects are mandatory; neither a tracking ref nor a missing
                // object authorizes fetching or establishes server reachability.
                crate::history::commit_in(s, &selection.child_oid)?;
                crate::history::commit_in(s, &observed)?;
                let args = [
                    "merge-base",
                    "--is-ancestor",
                    &selection.child_oid.hex,
                    &observed.hex,
                ];
                let output = s.output(
                    &args
                        .iter()
                        .map(std::ffi::OsString::from)
                        .collect::<Vec<_>>(),
                )?;
                complete_history(s)?;
                match output.status.code() {
                    Some(0) => Ok(true),
                    Some(1) => Ok(false),
                    _ => Err(fail(
                        ErrorKind::InspectionFailed,
                        "Cannot establish child commit ancestry",
                    )),
                }
            },
        )?;
        self.revalidate_transport(&current, &remaining(options, started)?)?;
        let after = self.push_preview(
            workspace,
            parent,
            &selection.parent_observation,
            &remaining(options, started)?,
        )?;
        if after.source_oid != preview.source_oid
            || after.generation != preview.generation
            || after.config_digest != preview.config_digest
            || !fresh()
        {
            return Err(stale());
        }
        Ok(ChildReachability {
            parent: preview,
            selection,
            mapping,
            child_remote: current.remote,
            child_destination: current.destination,
            child_reference: current.reference,
            observed_child_oid: observed,
            child_observed_at_ms: observed_at,
            child_generation: current.generation,
            child_config_digest: current.config_digest,
            checked_at_ms: now_ms(),
            reachable,
        })
    }
}
