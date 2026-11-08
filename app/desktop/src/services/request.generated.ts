/* Generated from Rust Core JSON Schema. Run npm run contracts. */

export type Request =
  | {
      increment: Increment;
      kind: "preview_version_file";
      path: NativePath;
      prerelease: PrereleasePolicy;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "authentication_settings";
    }
  | {
      kind: "authentication_choices";
    }
  | {
      change: Change;
      kind: "review_authentication";
    }
  | {
      cursor?: Cursor | null;
      kind: "graph";
      scope: HistoryScope;
      workspace: string;
      worktree: string;
    }
  | {
      author?: string | null;
      kind: "activity";
      reference?: string | null;
      since: number;
      until: number;
      workspace: string;
      worktree?: string | null;
    }
  | {
      kind: "operation_audit";
      query: OperationAuditQuery;
      workspace: string;
    }
  | {
      kind: "audit";
      query: AuditQuery;
      workspace: string;
    }
  | {
      kind: "review_batch";
      selections: BatchSelection[];
      workspace: string;
    }
  | {
      kind: "batches";
      offset: number;
      workspace: string;
    }
  | {
      kind: "load_bulk_composer";
      workspace: string;
    }
  | {
      kind: "save_bulk_composer";
      state: BulkComposerState;
      workspace: string;
    }
  | {
      kind: "remotes";
      workspace: string;
      worktree: string;
    }
  | {
      kind: "review_remote_observation";
      reference: string;
      remote: string;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "remote_observations";
      offset: number;
      workspace: string;
      worktree?: string | null;
    }
  | {
      kind: "review_upstream_setup";
      observation: string;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "review_upstream_recovery";
      operation: string;
      workspace: string;
    }
  | {
      kind: "push_preview";
      observation: string;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "child_reachability";
      selection: ChildReachabilitySelection;
      workspace: string;
      worktree: string;
    }
  | {
      hooks: HookPolicy;
      kind: "review_push_check";
      selection: PushPlanningSelection;
      workspace: string;
    }
  | {
      kind: "create_push_batch";
      selections: PushBatchSelection[];
      workspace: string;
    }
  | {
      kind: "push_batches";
      offset: number;
      workspace: string;
    }
  | {
      id: string;
      kind: "push_batch";
      workspace: string;
    }
  | {
      batch: string;
      kind: "review_push_batch_item";
      selection: PushPlanningSelection;
      workspace: string;
    }
  | {
      hooks: HookPolicy;
      kind: "review_push_publication";
      selection: PushPlanningSelection;
      workspace: string;
    }
  | {
      kind: "review_push_publication_recovery";
      observation: string;
      operation: string;
      workspace: string;
    }
  | {
      kind: "push_dependency_plan";
      selections: PushPlanningSelection[];
      workspace: string;
    }
  | {
      kind: "review_fetch_capture";
      observation: string;
      workspace: string;
      worktree: string;
    }
  | {
      capture: string;
      kind: "review_fetch_import";
      workspace: string;
      worktree: string;
    }
  | {
      kind: "fetch_imports";
      offset: number;
      workspace: string;
    }
  | {
      kind: "review_fetch_import_recovery";
      operation: string;
      workspace: string;
    }
  | {
      kind: "fetch_captures";
      offset: number;
      workspace: string;
      worktree?: string | null;
    }
  | {
      kind: "workspaces";
    }
  | {
      kind: "create";
      name: string;
      picker: string;
    }
  | {
      exclusions?: DiscoveryOptions | null;
      kind: "update";
      name?: string | null;
      picker?: string | null;
      workspace: string;
    }
  | {
      kind: "remove";
      workspace: string;
    }
  | {
      kind: "activate";
      workspace: string;
    }
  | {
      kind: "inventory";
      workspace: string;
    }
  | {
      kind: "scan";
      workspace: string;
    }
  | {
      kind: "refresh";
      workspace: string;
      worktree: string;
    }
  | {
      cursor?: Cursor2 | null;
      kind: "history";
      workspace: string;
      worktree: string;
    }
  | {
      kind: "commit";
      oid: string;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "diff";
      mode: DiffMode;
      path?: NativePath | null;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "preview";
      path: NativePath;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "load_state";
      workspace: string;
    }
  | {
      kind: "save_state";
      state: DesktopState;
      workspace: string;
    }
  | {
      kind: "trust_status";
      workspace: string;
    }
  | {
      kind: "review_trust";
      workspace: string;
      worktrees: string[];
    }
  | {
      kind: "revoke";
      workspace: string;
      worktree: string;
    }
  | {
      kind: "hook_policy";
      policy: HookPolicy;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "propose";
      request: WriteRequest;
      workspace: string;
      worktree: string;
    }
  | {
      kind: "operations";
      offset: number;
      workspace: string;
    }
  | {
      kind: "review_recovery";
      operation: string;
      workspace: string;
    };
export type Increment = "major" | "minor" | "patch";
/**
 * `display` is never used to reconstruct a path. Byte/wide encodings round-trip.
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
 * Prerelease handling is explicit, never inferred from a model or silently finalized.
 */
export type PrereleasePolicy = "reject" | "discard_and_increment";
export type Change =
  | {
      id?: string | null;
      kind: "save";
      name: string;
      policy: ProfilePolicy;
    }
  | {
      id: string;
      kind: "delete";
    }
  | {
      kind: "assign";
      profile?: string | null;
      remote: string;
      workspace: string;
      worktree: string;
    };
export type ProfilePolicy =
  | {
      account: string;
      ca_file?: string | null;
      helper: string;
      kind: "https";
    }
  | {
      agent_socket: string;
      kind: "ssh";
      known_hosts: string;
      public_identity: string;
    };
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
export type ObjectFormat = "sha1" | "sha256";
export type OperationAuditKind =
  | "stage"
  | "unstage"
  | "commit"
  | "batch_commit"
  | "remote_observation"
  | "fetch_capture"
  | "fetch_import"
  | "upstream_setup"
  | "push_check"
  | "push_publication"
  | "push_batch";
export type OperationAuditResult =
  | "pending"
  | "complete"
  | "blocked"
  | "cancelled"
  | "interrupted"
  | "needs_attention";
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
export type HookPolicy = "execute" | "bypass_all";
export type BatchMode = "review_all" | "index_only";
export type SigningPreference = "configured" | "unsigned";
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
export type ContentSource = "index" | "working_tree";
export type WriteRequest =
  | {
      hooks: HookPolicy;
      kind: "commit";
      message: string;
      selected: SelectedPath[];
      signing: SigningPreference;
      [k: string]: unknown;
    }
  | {
      hooks: HookPolicy;
      kind: "stage";
      paths: NativePath[];
      [k: string]: unknown;
    }
  | {
      hooks: HookPolicy;
      kind: "unstage";
      paths: NativePath[];
      [k: string]: unknown;
    };

export interface Cursor {
  history: Cursor2;
  lanes: ObjectId[];
}
export interface Cursor2 {
  generation: string;
  offset: number;
  scope: HistoryScope;
  [k: string]: unknown;
}
export interface ObjectId {
  format: ObjectFormat;
  hex: string;
  [k: string]: unknown;
}
export interface OperationAuditQuery {
  offset: number;
  operation_kind?: OperationAuditKind | null;
  result?: OperationAuditResult | null;
  since_ms?: number | null;
  until_ms?: number | null;
  worktree?: string | null;
}
export interface AuditQuery {
  offset: number;
  operation_kind?: string | null;
  phase?: Phase | null;
  since_ms?: number | null;
  until_ms?: number | null;
  worktree?: string | null;
}
export interface BatchSelection {
  excluded: NativePath[];
  hooks: HookPolicy;
  message: string;
  mode: BatchMode;
  signing: SigningPreference;
  worktree: string;
}
export interface BulkComposerState {
  configurations: {
    [k: string]: BatchConfiguration;
  };
  message: string;
  version: number;
}
export interface BatchConfiguration {
  excluded: NativePath[];
  hooks: HookPolicy;
  mode: BatchMode;
  signing: SigningPreference;
}
export interface ChildReachabilitySelection {
  child_observation: string;
  child_oid: ObjectId;
  child_worktree: string;
  gitmodules_blob: ObjectId;
  parent_observation: string;
  path: number[];
  source_oid: ObjectId;
  [k: string]: unknown;
}
/**
 * Explicit planning selection. Nothing in this request authorizes publication.
 */
export interface PushPlanningSelection {
  children: ChildReachabilitySelection[];
  observation: string;
  source_oid: ObjectId;
  worktree: string;
}
export interface PushBatchSelection {
  hooks: HookPolicy;
  selection: PushPlanningSelection;
}
export interface DiscoveryOptions {
  excluded_names: string[];
  excluded_patterns: string[];
  [k: string]: unknown;
}
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
export interface SelectedPath {
  path: NativePath;
  source: ContentSource;
  [k: string]: unknown;
}
