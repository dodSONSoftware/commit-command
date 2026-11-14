use super::*;

pub(crate) const LOCAL: &str = "json_extract(payload,'$.phase') NOT IN ('complete','blocked','published_with_post_processing_failure','cancelled_before_publication','proposed','approved','needs_reauthorization')";
pub(crate) const FETCH: &str = "key LIKE 'fetch-import:%' AND json_extract(value,'$.value.phase') IN ('importing_objects','publication_pending','reference_prepared','reference_updated','uncertain')";
pub(crate) const UPSTREAM: &str = "key LIKE 'upstream:%' AND json_extract(value,'$.value.phase') IN ('lock_pending','lock_created','lock_ready','publication_pending','config_published','uncertain')";
pub(crate) const VERSION: &str = "key LIKE 'version-action:%' AND json_extract(value,'$.value.phase') IN ('preparing','replacing','unconfirmed')";
pub(crate) const PUBLICATION: &str = "key LIKE 'push-publication:%' AND (json_extract(value,'$.value.phase') IN ('checking','preparing','publishing','unconfirmed') OR (json_extract(value,'$.value.phase')='proposed' AND json_extract(value,'$.session')<>?1))";
pub(crate) const BATCH: &str = "key LIKE 'push-batch:%' AND (json_extract(value,'$.value.phase')='executing' OR (json_extract(value,'$.value.phase')='ready' AND json_extract(value,'$.session')<>?1))";

impl WorkspaceService {
    pub(super) fn require_resolved(&self, id: &WorkspaceId) -> Result<()> {
        // The original workspace-only schema predates the write journal.
        let version: u32 = self
            .database
            .pragma_query_value(None, "user_version", |r| r.get(0))?;
        let local = if version >= 2 {
            format!("EXISTS(SELECT 1 FROM write_operations WHERE json_extract(payload,'$.proposal.workspace_id')=?2 AND {LOCAL})")
        } else {
            "0".into()
        };
        let sql = format!("SELECT {local} OR
            EXISTS(SELECT 1 FROM state WHERE {FETCH} AND json_extract(value,'$.value.plan.workspace')=?2) OR
            EXISTS(SELECT 1 FROM state WHERE {UPSTREAM} AND json_extract(value,'$.value.plan.workspace')=?2) OR
            EXISTS(SELECT 1 FROM state WHERE {VERSION} AND json_extract(value,'$.value.plan.workspace')=?2) OR
            EXISTS(SELECT 1 FROM state WHERE {PUBLICATION} AND json_extract(value,'$.value.plan.check.workspace')=?2) OR
            EXISTS(SELECT 1 FROM state WHERE {BATCH} AND json_extract(value,'$.value.plan.workspace')=?2)");
        let pending: bool = self
            .database
            .query_row(&sql, params![self.session, id.0], |r| r.get(0))?;
        if pending {
            return Err(Error::with_kind(
                ErrorKind::Busy,
                "Resolve interrupted operations before removing or changing workspace scope",
            ));
        }
        Ok(())
    }
}
