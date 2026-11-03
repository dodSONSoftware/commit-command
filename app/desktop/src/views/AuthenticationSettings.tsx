import { useEffect, useState } from "react";
import { value, type Bridge } from "../services/bridge";
import type { Change, ProfilePolicy } from "../services/request.generated";
import type { Profile, Settings } from "../services/response.generated";
import { bytes } from "../stores/model";
const text = (s: string) => bytes(Array.from(new TextEncoder().encode(s)));
export function AuthenticationSettings({
  bridge,
  workspace,
  worktree,
  remotes,
  busy,
  enabled,
  revision,
  act,
}: {
  bridge: Bridge;
  workspace: string;
  worktree: string;
  remotes: string[];
  busy: boolean;
  enabled: boolean;
  revision?: number;
  act: (action: () => Promise<void>) => Promise<void>;
}) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [kind, setKind] = useState<"https" | "ssh">("https");
  const [helper, setHelper] = useState("");
  const [account, setAccount] = useState("");
  const [ca, setCa] = useState("");
  const [known, setKnown] = useState("");
  const [identity, setIdentity] = useState("");
  const [agent, setAgent] = useState("");
  const [remote, setRemote] = useState("");
  const [profile, setProfile] = useState("");
  useEffect(() => {
    let live = true;
    void act(async () => {
      const s = value(
        await bridge.request({ kind: "authentication_settings" }),
        "authentication_settings",
      );
      if (live) setSettings(s);
    });
    return () => {
      live = false;
    };
  }, [bridge, workspace, worktree, revision, act]);
  const review = (change: Change) =>
    void act(async () => {
      await bridge.request({ kind: "review_authentication", change });
    });
  function edit(p: Profile) {
    setEditing(p.id);
    setName(p.name);
    setKind(p.policy.kind);
    if (p.policy.kind === "https") {
      setHelper(p.policy.helper);
      setAccount(p.policy.account);
      setCa(p.policy.ca_file ?? "");
    } else {
      setKnown(p.policy.known_hosts);
      setIdentity(p.policy.public_identity);
      setAgent(p.policy.agent_socket);
    }
  }
  const policy: ProfilePolicy =
    kind === "https"
      ? { kind, helper, account, ca_file: ca || null }
      : {
          kind,
          known_hosts: known,
          public_identity: identity,
          agent_socket: agent,
        };
  const valid = Boolean(
    name.trim() &&
    (kind === "https" ? helper && account : known && identity && agent),
  );
  return (
    <section aria-label="Authentication profiles and assignments">
      <h3>Authentication profiles</h3>
      <p>
        Profiles store nonsecret account and tool settings. Each remote
        assignment and profile change requires an independent native review.
        Saving does not contact a server or look up credentials.
      </p>
      <p>
        Selected helpers run as your user and may show unlock or sign-in prompts
        during an approved operation. No background authentication. Unknown or
        changed SSH host keys stop execution; verify and update known_hosts
        externally.
      </p>
      {!enabled && (
        <p>Settings changes are disabled in this read-only build.</p>
      )}
      {settings?.profiles.length === 0 && (
        <p>No saved authentication profiles.</p>
      )}
      {settings?.profiles.map((p) => (
        <article key={p.id}>
          <h4>{text(p.name)}</h4>
          <p>
            {p.policy.kind.toUpperCase()} ·{" "}
            {p.policy.kind === "https"
              ? text(p.policy.account)
              : "Selected public identity and agent"}
          </p>
          <button disabled={busy || !enabled} onClick={() => edit(p)}>
            Edit {text(p.name)}
          </button>
          <button
            disabled={
              busy ||
              !enabled ||
              settings.assignments.some((a) => a.profile === p.id)
            }
            onClick={() => review({ kind: "delete", id: p.id })}
          >
            Review delete {text(p.name)}
          </button>
        </article>
      ))}
      <form
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="none"
        spellCheck={false}
        onSubmit={(e) => {
          e.preventDefault();
          if (valid) review({ kind: "save", id: editing, name, policy });
        }}
      >
        <h4>{editing ? "Edit profile" : "New profile"}</h4>
        <label>
          Profile name
          <input
            value={name}
            maxLength={80}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <label>
          Authentication method
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value as "https" | "ssh")}
          >
            <option value="https">HTTPS credential helper</option>
            <option value="ssh">SSH agent</option>
          </select>
        </label>
        {kind === "https" ? (
          <>
            <label>
              Credential helper executable
              <input
                value={helper}
                placeholder="/absolute/path/to/git-credential-osxkeychain"
                onChange={(e) => setHelper(e.target.value)}
              />
            </label>
            <label>
              Nonsecret account
              <input
                value={account}
                autoComplete="off"
                onChange={(e) => setAccount(e.target.value)}
              />
            </label>
            <label>
              CA file (optional)
              <input
                value={ca}
                placeholder="System certificate trust when empty"
                onChange={(e) => setCa(e.target.value)}
              />
            </label>
          </>
        ) : (
          <>
            <label>
              Known-hosts file
              <input value={known} onChange={(e) => setKnown(e.target.value)} />
            </label>
            <label>
              Ed25519 public identity file
              <input
                value={identity}
                onChange={(e) => setIdentity(e.target.value)}
              />
            </label>
            <label>
              SSH agent socket
              <input value={agent} onChange={(e) => setAgent(e.target.value)} />
            </label>
          </>
        )}
        <button disabled={busy || !enabled || !valid} type="submit">
          Review profile settings
        </button>
        {editing && (
          <button
            type="button"
            disabled={busy}
            onClick={() => {
              setEditing(null);
              setName("");
            }}
          >
            Start a new profile
          </button>
        )}
      </form>
      <h3>Assign a profile to this remote</h3>
      <p>
        Inspect remote destinations first. Assignment binds the exact resolved
        fetch destination; destination changes require a new assignment review.
        Assignment does not authorize a network operation.
      </p>
      <label>
        Remote for authentication
        <select
          value={remote}
          onChange={(e) => {
            setRemote(e.target.value);
            setProfile("");
          }}
        >
          <option value="">Choose an inspected remote</option>
          {remotes.map((r) => (
            <option key={r} value={r}>
              {text(r)}
            </option>
          ))}
        </select>
      </label>
      <label>
        Authentication profile
        <select value={profile} onChange={(e) => setProfile(e.target.value)}>
          <option value="">No saved profile assignment</option>
          {settings?.profiles.map((p) => (
            <option key={p.id} value={p.id}>
              {text(p.name)}
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={busy || !enabled || !remote || !settings}
        onClick={() =>
          review({
            kind: "assign",
            workspace,
            worktree,
            remote,
            profile: profile || null,
          })
        }
      >
        Review remote assignment
      </button>
      {settings?.assignments
        .filter((a) => a.workspace === workspace && a.worktree === worktree)
        .map((a) => (
          <p key={a.remote}>
            Assigned {text(a.remote)} →{" "}
            {text(
              settings.profiles.find((p) => p.id === a.profile)?.name ??
                "Missing profile",
            )}{" "}
            · {text(a.destination)}
          </p>
        ))}
    </section>
  );
}
