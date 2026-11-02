/* Generated from Rust Core JSON Schema. Run npm run contracts. */

/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Data".
 */
export type Data =
  | {
      kind: "graph";
      value: Page;
      [k: string]: unknown;
    }
  | {
      kind: "activity";
      value: ActivityReport;
      [k: string]: unknown;
    }
  | {
      kind: "operation_audit";
      value: OperationAuditPage;
      [k: string]: unknown;
    }
  | {
      kind: "audit";
      value: AuditPage;
      [k: string]: unknown;
    }
  | {
      kind: "batches";
      value: BatchPage;
      [k: string]: unknown;
    }
  | {
      kind: "bulk_composer";
      value: BulkComposerState;
      [k: string]: unknown;
    }
  | {
      kind: "remotes";
      value: Inspection;
      [k: string]: unknown;
    }
  | {
      kind: "remote_observations";
      value: ObservationPage;
      [k: string]: unknown;
    }
  | {
      kind: "workspaces";
      value: WorkspaceList;
      [k: string]: unknown;
    }
  | {
      kind: "inventory";
      value: InventoryView;
      [k: string]: unknown;
    }
  | {
      kind: "refreshed";
      value: StoredWorktree;
      [k: string]: unknown;
    }
  | {
      kind: "history";
      value: HistoryPage;
      [k: string]: unknown;
    }
  | {
      kind: "commit";
      value: Commit;
      [k: string]: unknown;
    }
  | {
      kind: "diff";
      value: Diff;
      [k: string]: unknown;
    }
  | {
      kind: "preview";
      value: Preview;
      [k: string]: unknown;
    }
  | {
      kind: "state";
      value: DesktopState;
      [k: string]: unknown;
    }
  | {
      kind: "trust";
      value: TrustState[];
      [k: string]: unknown;
    }
  | {
      kind: "review";
      value: ReviewDisplay;
      [k: string]: unknown;
    }
  | {
      kind: "operations";
      value: OperationPage;
      [k: string]: unknown;
    }
  | {
      kind: "done";
      [k: string]: unknown;
    };
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "HistoryScope".
 */
export type HistoryScope =
  | {
      kind: "local_branches";
      [k: string]: unknown;
    }
  | {
      kind: "reference";
      reference: string;
      [k: string]: unknown;
    };
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ObjectFormat".
 */
export type ObjectFormat = "sha1" | "sha256";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ErrorKind".
 */
export type ErrorKind =
  | "invalid_path"
  | "unsupported_configuration"
  | "inspection_failed"
  | "hook_rejected"
  | "io"
  | "invalid_data"
  | "experimental"
  | "invalid_request"
  | "unsupported_git_version"
  | "unavailable"
  | "concurrent_change"
  | "stale_cursor"
  | "missing_object"
  | "timeout"
  | "cancelled"
  | "output_limit"
  | "busy"
  | "storage"
  | "newer_schema"
  | "untrusted_repository"
  | "unsupported_repository_state"
  | "stale_review"
  | "signing_failed"
  | "needs_manual_reconciliation"
  | "uncertain_outcome";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationAuditKind".
 */
export type OperationAuditKind =
  "stage" | "unstage" | "commit" | "batch_commit" | "remote_observation";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationAuditRecord".
 */
export type OperationAuditRecord =
  | {
      kind: "local";
      value: AuditEntry;
      [k: string]: unknown;
    }
  | {
      kind: "batch";
      value: BatchView;
      [k: string]: unknown;
    }
  | {
      kind: "remote_observation";
      value: Observation;
      [k: string]: unknown;
    };
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Phase".
 */
export type Phase =
  | "proposed"
  | "approved"
  | "preparing"
  | "hooks_running"
  | "revalidated"
  | "object_ready"
  | "index_lock_written"
  | "publication_pending"
  | "reference_updated"
  | "index_renamed"
  | "head_log_written"
  | "published"
  | "index_reconciled"
  | "post_processing"
  | "complete"
  | "blocked"
  | "needs_reauthorization"
  | "cancelled_before_publication"
  | "published_needs_reconciliation"
  | "published_with_post_processing_failure"
  | "needs_manual_reconciliation"
  | "uncertain_outcome";
/**
 * `display` is never used to reconstruct a path. Byte/wide encodings round-trip.
 *
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "NativePath".
 */
export type NativePath =
  | {
      encoding: "utf8";
      value: string;
      [k: string]: unknown;
    }
  | {
      encoding: "unix_bytes";
      value: number[];
      [k: string]: unknown;
    }
  | {
      encoding: "windows_wide";
      value: number[];
      [k: string]: unknown;
    };
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchPhase".
 */
export type BatchPhase =
  | "proposed"
  | "executing"
  | "complete"
  | "blocked"
  | "partial_failure"
  | "cancelled"
  | "needs_reauthorization"
  | "interrupted";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ObservationPhase".
 */
export type ObservationPhase =
  "proposed" | "running" | "complete" | "blocked" | "cancelled" | "interrupted";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Transport".
 */
export type Transport =
  "local" | "https" | "ssh" | "git" | "remote_helper" | "unsupported";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationAuditResult".
 */
export type OperationAuditResult =
  | "pending"
  | "complete"
  | "blocked"
  | "cancelled"
  | "interrupted"
  | "needs_attention";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "HookPolicy".
 */
export type HookPolicy = "execute" | "bypass_all";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchMode".
 */
export type BatchMode = "review_all" | "index_only";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "SigningPreference".
 */
export type SigningPreference = "configured" | "unsigned";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Availability".
 */
export type Availability = "available" | "missing" | "unavailable";
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "DiffMode".
 */
export type DiffMode =
  | {
      kind: "staged";
      [k: string]: unknown;
    }
  | {
      kind: "unstaged";
      [k: string]: unknown;
    }
  | {
      commit: ObjectId;
      kind: "historical";
      parent?: ObjectId | null;
      [k: string]: unknown;
    };
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ContentSource".
 */
export type ContentSource = "index" | "working_tree";

export interface Response {
  data: Data;
  epoch: number;
  version: number;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Page".
 */
export interface Page {
  generation: string;
  next?: Cursor | null;
  references: Reference[];
  rows: Row[];
  shallow_boundaries: ObjectId[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Cursor".
 */
export interface Cursor {
  history: Cursor2;
  lanes: ObjectId[];
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Cursor2".
 */
export interface Cursor2 {
  generation: string;
  offset: number;
  scope: HistoryScope;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ObjectId".
 */
export interface ObjectId {
  format: ObjectFormat;
  hex: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Reference".
 */
export interface Reference {
  name: string;
  object_type: string;
  oid: ObjectId;
  peeled?: ObjectId | null;
  upstream?: string | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Row".
 */
export interface Row {
  after: ObjectId[];
  before: ObjectId[];
  commit: Commit;
  lane: number;
  tip: boolean;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Commit".
 */
export interface Commit {
  author: number[];
  author_time: number;
  committer: number[];
  committer_time: number;
  encoding?: string | null;
  message: number[];
  oid: ObjectId;
  parents: ObjectId[];
  tree: ObjectId;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ActivityReport".
 */
export interface ActivityReport {
  complete: boolean;
  issues: Issue[];
  items: ActivityEntry[];
  observed_at_ms: number;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Issue".
 */
export interface Issue {
  kind: ErrorKind;
  message: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ActivityEntry".
 */
export interface ActivityEntry {
  commit: Commit2;
  repository: string;
  worktree: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Commit2".
 */
export interface Commit2 {
  author: number[];
  committer_time: number;
  oid: ObjectId;
  subject: number[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationAuditPage".
 */
export interface OperationAuditPage {
  items: OperationAuditEntry[];
  next?: number | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationAuditEntry".
 */
export interface OperationAuditEntry {
  created_at_ms: number;
  id: string;
  operation_kind: OperationAuditKind;
  record: OperationAuditRecord;
  result: OperationAuditResult;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "AuditEntry".
 */
export interface AuditEntry {
  created_at_ms: number;
  events: AuditPhase[];
  events_complete: boolean;
  kind: string;
  outcome: Outcome;
  subject: string;
  worktree: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "AuditPhase".
 */
export interface AuditPhase {
  at_ms: number;
  new_oid?: string | null;
  phase: Phase;
  published: boolean;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Outcome".
 */
export interface Outcome {
  index_reconciled: boolean;
  new_head?: string | null;
  operation_id: string;
  phase: Phase;
  post_processing: string[];
  published: boolean;
  reason?: string | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchView".
 */
export interface BatchView {
  outcomes: Outcome[];
  record: BatchRecord;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchRecord".
 */
export interface BatchRecord {
  created_at_ms: number;
  id: string;
  issue?: Issue | null;
  items: BatchItem[];
  phase: BatchPhase;
  version: number;
  workspace: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchItem".
 */
export interface BatchItem {
  dependencies: string[];
  issue?: Issue | null;
  operation?: string | null;
  path: NativePath;
  worktree: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Observation".
 */
export interface Observation {
  issue?: Issue | null;
  observed_at_ms?: number | null;
  oid?: ObjectId | null;
  phase: ObservationPhase;
  plan: ObservationPlan;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ObservationPlan".
 */
export interface ObservationPlan {
  config_digest: string;
  created_at_ms: number;
  destination: string;
  generation: string;
  id: string;
  local_binding?: string | null;
  local_target?: NativePath | null;
  metadata: Metadata;
  reference: string;
  remote: string;
  tools: ToolBinding[];
  transport: Transport;
  workspace: string;
  worktree: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Metadata".
 */
export interface Metadata {
  common_binding: string;
  common_dir: NativePath;
  git_binding: string;
  git_dir: NativePath;
  object_format: ObjectFormat;
  root: NativePath;
  root_binding: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ToolBinding".
 */
export interface ToolBinding {
  fingerprint: string;
  path: NativePath;
  role: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "AuditPage".
 */
export interface AuditPage {
  items: AuditEntry[];
  next?: number | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchPage".
 */
export interface BatchPage {
  items: BatchView[];
  next?: number | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BulkComposerState".
 */
export interface BulkComposerState {
  configurations: {
    [k: string]: BatchConfiguration;
  };
  message: string;
  version: number;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "BatchConfiguration".
 */
export interface BatchConfiguration {
  excluded: NativePath[];
  hooks: HookPolicy;
  mode: BatchMode;
  signing: SigningPreference;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Inspection".
 */
export interface Inspection {
  branch?: string | null;
  config_digest: string;
  credential_helper_configured: boolean;
  generation: string;
  issues: string[];
  observed_at_ms: number;
  proxy_configured: boolean;
  remotes: Remote[];
  ssh_command_configured: boolean;
  suggested_push_remote?: string | null;
  tls_verification_disabled: boolean;
  upstream_ref?: string | null;
  upstream_remote?: string | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Remote".
 */
export interface Remote {
  fetch: Destination[];
  fetch_refspec_count: number;
  issues: string[];
  name: string;
  origins: ConfigOrigin[];
  push: Destination[];
  push_refspec_count: number;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Destination".
 */
export interface Destination {
  credentials_hidden: boolean;
  display: string;
  restrictions: string[];
  transport: Transport;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ConfigOrigin".
 */
export interface ConfigOrigin {
  file?: NativePath | null;
  scope: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ObservationPage".
 */
export interface ObservationPage {
  items: Observation[];
  next?: number | null;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "WorkspaceList".
 */
export interface WorkspaceList {
  active?: string | null;
  recovery: RecoveryNotice[];
  workspaces: Workspace[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "RecoveryNotice".
 */
export interface RecoveryNotice {
  operation_id: string;
  phase: Phase;
  published: boolean;
  reason?: string | null;
  workspace_id: string;
  worktree_id: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Workspace".
 */
export interface Workspace {
  discovery: DiscoveryOptions;
  display_root: NativePath;
  id: string;
  name: string;
  root: NativePath;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "DiscoveryOptions".
 */
export interface DiscoveryOptions {
  excluded_names: string[];
  excluded_patterns: string[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "InventoryView".
 */
export interface InventoryView {
  entries: StoredWorktree[];
  monitor: MonitorStatus;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "StoredWorktree".
 */
export interface StoredWorktree {
  availability: Availability;
  display_path: NativePath;
  id: string;
  issue?: Issue | null;
  metadata: Metadata;
  repository_id: string;
  status?: Snapshot | null;
  untrusted: boolean;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Snapshot".
 */
export interface Snapshot {
  branch?: string | null;
  changes: Change[];
  counts: Counts;
  detached: boolean;
  generation: string;
  head?: string | null;
  head_oid?: ObjectId | null;
  last_commit?: Commit | null;
  observed_at_ms: number;
  operations: string[];
  /**
   * Compatibility diagnostic only. Structured consumers use changes/counts.
   */
  status: string;
  tracking?: Tracking | null;
  unborn: boolean;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Change".
 */
export interface Change {
  conflict: boolean;
  head_mode: string;
  index: string;
  index_mode: string;
  original_path?: NativePath | null;
  path: NativePath;
  submodule: string;
  worktree: string;
  worktree_mode: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Counts".
 */
export interface Counts {
  conflicted: number;
  staged: number;
  unstaged: number;
  untracked: number;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Tracking".
 */
export interface Tracking {
  ahead?: number | null;
  behind?: number | null;
  comparison: string;
  fetch_freshness?: number | null;
  oid?: ObjectId | null;
  reference: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "MonitorStatus".
 */
export interface MonitorStatus {
  degraded?: string | null;
  epoch: number;
  running: boolean;
  watched: number;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "HistoryPage".
 */
export interface HistoryPage {
  commits: Commit[];
  generation: string;
  next?: Cursor2 | null;
  observed_at_ms: number;
  references: Reference[];
  shallow: boolean;
  shallow_boundaries: ObjectId[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Diff".
 */
export interface Diff {
  complete: boolean;
  files: FileChange[];
  generation: string;
  mode: DiffMode;
  observed_at_ms: number;
  patch: number[];
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "FileChange".
 */
export interface FileChange {
  change: string;
  new_mode: string;
  new_oid?: ObjectId | null;
  old_mode: string;
  old_oid?: ObjectId | null;
  original_path?: NativePath | null;
  path: NativePath;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "Preview".
 */
export interface Preview {
  binary: boolean;
  bytes: number[];
  complete: boolean;
  fingerprint: string;
  kind: string;
  path: NativePath;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "DesktopState".
 */
export interface DesktopState {
  active_tab?: string | null;
  drafts: {
    [k: string]: string;
  };
  filter: string;
  inspector_width: number;
  layout: string;
  selected: string[];
  sidebar_width: number;
  sort: string;
  tabs: RepositoryTab[];
  theme: string;
  version: number;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "RepositoryTab".
 */
export interface RepositoryTab {
  commit?: string | null;
  filter: string;
  parent?: string | null;
  path?: NativePath | null;
  scroll: number;
  source: ContentSource;
  view: string;
  worktree: string;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "TrustState".
 */
export interface TrustState {
  eligible: boolean;
  grant: string;
  policy?: HookPolicy | null;
  reason?: string | null;
  validity: string;
  worktree: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "ReviewDisplay".
 */
export interface ReviewDisplay {
  content: string;
  epoch: number;
  expires_at_ms: number;
  id: string;
  summary: string;
  title: string;
  [k: string]: unknown;
}
/**
 * This interface was referenced by `Response`'s JSON-Schema
 * via the `definition` "OperationPage".
 */
export interface OperationPage {
  items: Outcome[];
  next?: number | null;
  [k: string]: unknown;
}
