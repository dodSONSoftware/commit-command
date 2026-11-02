//! Read-only projections of existing journals; history never restores execution authority.
use super::*;
use crate::remote::observation::{Observation, ObservationPhase};
use rusqlite::params;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationAuditKind {
    Stage,
    Unstage,
    Commit,
    BatchCommit,
    RemoteObservation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperationAuditResult {
    Pending,
    Complete,
    Blocked,
    Cancelled,
    Interrupted,
    NeedsAttention,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationAuditQuery {
    pub offset: usize,
    pub worktree: Option<String>,
    pub operation_kind: Option<OperationAuditKind>,
    pub result: Option<OperationAuditResult>,
    pub since_ms: Option<u64>,
    pub until_ms: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum OperationAuditRecord {
    Local(AuditEntry),
    Batch(BatchView),
    RemoteObservation(Box<Observation>),
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OperationAuditEntry {
    pub id: String,
    pub created_at_ms: u64,
    pub operation_kind: OperationAuditKind,
    pub result: OperationAuditResult,
    pub record: OperationAuditRecord,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OperationAuditPage {
    pub items: Vec<OperationAuditEntry>,
    pub next: Option<usize>,
}

// Filtering and ordering happen before paging across sources. No journal is rewritten.
const QUERY: &str = r#"
WITH sources AS (
  SELECT 'local' AS source, id, json_extract(payload,'$.proposal.created_at_ms') AS created,
         json_extract(payload,'$.proposal.request.kind') AS kind,
         json_extract(payload,'$.phase') AS phase, payload,
         json_extract(payload,'$.proposal.worktree_id') AS worktree
    FROM write_operations WHERE json_extract(payload,'$.proposal.workspace_id')=?1
  UNION ALL
  SELECT 'batch', substr(key,7), json_extract(value,'$.created_at_ms'),
         'batch_commit', json_extract(value,'$.phase'), value, NULL
    FROM state WHERE key LIKE 'batch:%' AND json_extract(value,'$.workspace')=?1
  UNION ALL
  SELECT 'remote', substr(key,20), json_extract(value,'$.value.plan.created_at_ms'),
         'remote_observation', json_extract(value,'$.value.phase'), value,
         json_extract(value,'$.value.plan.worktree')
    FROM state WHERE key LIKE 'remote-observation:%' AND json_extract(value,'$.value.plan.workspace')=?1
), classified AS (
  SELECT *, CASE
    WHEN source='local' AND phase IN ('proposed','approved') AND id IS NOT ?8 THEN 'interrupted'
    WHEN source='batch' AND (phase='executing' OR (phase='proposed' AND id IS NOT ?9)) THEN 'interrupted'
    WHEN source='remote' AND (phase='running' OR (phase='proposed' AND id IS NOT ?10)) THEN 'interrupted'
    WHEN phase='complete' THEN 'complete'
    WHEN phase='blocked' THEN 'blocked'
    WHEN phase IN ('cancelled','cancelled_before_publication') THEN 'cancelled'
    WHEN phase='interrupted' THEN 'interrupted'
    WHEN phase IN ('proposed','approved') THEN 'pending'
    ELSE 'needs_attention' END AS result
  FROM sources
)
SELECT source,id,created,kind,result,
       CASE WHEN length(CAST(payload AS BLOB))<=4194304 THEN payload ELSE NULL END
FROM classified
WHERE (?2 IS NULL OR worktree=?2 OR (source='batch' AND EXISTS (
  SELECT 1 FROM json_each(payload,'$.items') WHERE json_extract(value,'$.worktree')=?2)))
  AND (?3 IS NULL OR kind=?3) AND (?4 IS NULL OR result=?4)
  AND (?5 IS NULL OR created>=?5) AND (?6 IS NULL OR created<=?6)
ORDER BY created DESC,source,id LIMIT 51 OFFSET ?7
"#;

fn name<T: Serialize>(value: Option<T>) -> Result<Option<String>> {
    value
        .map(|v| {
            let encoded = serde_json::to_value(v)?;
            encoded
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("Invalid audit filter"))
        })
        .transpose()
}
fn decoded<T: serde::de::DeserializeOwned>(value: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(value.into())).map_err(Error::from)
}
impl Application {
    pub(super) fn operation_audit(
        &mut self,
        workspace: &str,
        query: OperationAuditQuery,
    ) -> Result<OperationAuditPage> {
        self.active(workspace)?;
        if query.offset > 1_000_000
            || query.since_ms.is_some_and(|v| v > i64::MAX as u64)
            || query.until_ms.is_some_and(|v| v > i64::MAX as u64)
            || query
                .since_ms
                .zip(query.until_ms)
                .is_some_and(|(a, b)| a > b)
        {
            return Err(invalid("Invalid operation audit range or offset"));
        }
        if let Some(worktree) = &query.worktree {
            // Scope by stored IDs so removed/unavailable checkouts remain inspectable.
            Uuid::parse_str(worktree).map_err(|_| invalid("Invalid audit worktree ID"))?;
        }
        let (mut write, mut batch, mut remote) = (None, None, None);
        if let Some(pending) = self.pending.as_ref().filter(|p| self.review(&p.id).is_ok()) {
            match &pending.action {
                review::Action::Write(p) => write = Some(p.operation_id.clone()),
                review::Action::Batch(p) => batch = Some(p.id.clone()),
                review::Action::RemoteObservation(p) => remote = Some(p.id.clone()),
                _ => {}
            }
        }
        let kind = name(query.operation_kind)?;
        let result = name(query.result)?;
        let mut rows = Vec::new();
        let mut more = false;
        {
            let mut statement = self.writes.workspace.connection().prepare(QUERY)?;
            let records = statement.query_map(
                params![
                    workspace,
                    query.worktree,
                    kind,
                    result,
                    query.since_ms.map(|v| v as i64),
                    query.until_ms.map(|v| v as i64),
                    query.offset as i64,
                    write,
                    batch,
                    remote
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, Option<String>>(5)?,
                    ))
                },
            )?;
            let mut size = 0;
            for row in records {
                let row = row?;
                if rows.len() == 50 {
                    more = true;
                    break;
                }
                let length = row.5.as_ref().map(String::len).ok_or_else(|| {
                    Error::with_kind(ErrorKind::OutputLimit, "Stored audit record exceeds 4 MiB")
                })?;
                if size + length > 4 * 1024 * 1024 {
                    more = true;
                    break;
                }
                size += length;
                rows.push(row);
            }
        }
        let mut items = Vec::new();
        let mut size = 0;
        for (source, id, created, kind, result, payload) in rows {
            let payload = payload.ok_or_else(|| invalid("Missing audit record"))?;
            let created =
                u64::try_from(created).map_err(|_| invalid("Invalid stored audit time"))?;
            let operation_kind = decoded(&kind)?;
            let result = decoded(&result)?;
            let record = match source.as_str() {
                "local" => {
                    let detail = self.writes.application_audit_detail(&payload)?;
                    if detail.outcome.operation_id != id
                        || detail.created_at_ms != created
                        || !matches!(
                            operation_kind,
                            OperationAuditKind::Stage
                                | OperationAuditKind::Unstage
                                | OperationAuditKind::Commit
                        )
                    {
                        return Err(invalid("Inconsistent local audit record"));
                    }
                    OperationAuditRecord::Local(detail)
                }
                "batch" => {
                    let mut record: BatchRecord = serde_json::from_str(&payload)?;
                    if record.version != 1 {
                        return Err(Error::with_kind(
                            ErrorKind::NewerSchema,
                            "Unsupported batch record",
                        ));
                    }
                    if record.id != id
                        || record.workspace.0 != workspace
                        || record.created_at_ms != created
                    {
                        return Err(invalid("Inconsistent batch audit record"));
                    }
                    if result == OperationAuditResult::Interrupted {
                        record.phase = BatchPhase::Interrupted;
                    }
                    OperationAuditRecord::Batch(self.batch_view(record)?)
                }
                "remote" => {
                    let mut observation = self.writes.observation_audit_detail(&payload)?;
                    if observation.plan.id != id
                        || observation.plan.workspace.0 != workspace
                        || observation.plan.created_at_ms != created
                    {
                        return Err(invalid("Inconsistent observation audit record"));
                    }
                    if result == OperationAuditResult::Interrupted {
                        observation.phase = ObservationPhase::Interrupted;
                    }
                    OperationAuditRecord::RemoteObservation(Box::new(observation))
                }
                _ => return Err(invalid("Unknown audit source")),
            };
            let entry = OperationAuditEntry {
                id,
                created_at_ms: created,
                operation_kind,
                result,
                record,
            };
            let length = serde_json::to_vec(&entry)?.len();
            if size + length > 8 * 1024 * 1024 {
                if items.is_empty() {
                    return Err(Error::with_kind(
                        ErrorKind::OutputLimit,
                        "Audit detail exceeds 8 MiB",
                    ));
                }
                more = true;
                break;
            }
            size += length;
            items.push(entry);
        }
        let next = more.then_some(query.offset + items.len());
        Ok(OperationAuditPage { items, next })
    }
}
