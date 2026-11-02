/* Generated from Rust Core JSON Schema. Run npm run contracts. */

export type Request =
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
      kind: "audit";
      query: AuditQuery;
      workspace: string;
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
export type ContentSource = "index" | "working_tree";
export type HookPolicy = "execute" | "bypass_all";
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
export type SigningPreference = "configured" | "unsigned";

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
export interface AuditQuery {
  offset: number;
  operation_kind?: string | null;
  phase?: Phase | null;
  since_ms?: number | null;
  until_ms?: number | null;
  worktree?: string | null;
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
