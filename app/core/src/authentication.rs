//! Reviewed nonsecret authentication configuration. Saving never contacts a server/helper.
use crate::{
    domain::*,
    remote::{self, observation::ToolBinding, TransportOptions},
    Error, ErrorKind, Result,
};
use rusqlite::{Connection, OptionalExtension};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProfilePolicy {
    Https {
        helper: String,
        account: String,
        ca_file: Option<String>,
    },
    Ssh {
        known_hosts: String,
        public_identity: String,
        agent_socket: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub policy: ProfilePolicy,
    pub tools: Vec<ToolBinding>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub workspace: WorkspaceId,
    pub worktree: WorktreeId,
    pub remote: String,
    pub destination: String,
    pub profile: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub revision: String,
    pub profiles: Vec<Profile>,
    pub assignments: Vec<Assignment>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileBinding {
    pub revision: String,
    pub id: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Change {
    Save {
        id: Option<String>,
        name: String,
        policy: ProfilePolicy,
    },
    Delete {
        id: String,
    },
    Assign {
        workspace: String,
        worktree: String,
        remote: String,
        profile: Option<String>,
    },
}
pub(crate) fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}
fn path(value: &str) -> Result<PathBuf> {
    if value.len() > 4096
        || value.chars().any(char::is_control)
        || !std::path::Path::new(value).is_absolute()
    {
        return Err(invalid(
            "Select an absolute, control-free authentication path",
        ));
    }
    Ok(value.into())
}
impl ProfilePolicy {
    pub(crate) fn options(
        &self,
        base: &TransportOptions,
    ) -> Result<(TransportOptions, remote::Transport)> {
        let mut options = base.clone();
        options.ssh = None;
        options.https_credentials = None;
        options.ca_file = None;
        let transport = match self {
            Self::Https {
                helper,
                account,
                ca_file,
            } => {
                crate::credentials::valid_account(account)?;
                options.https_credentials = Some(remote::HttpsCredentialPolicy {
                    home: base
                        .credential_home
                        .clone()
                        .ok_or_else(|| invalid("Native credential home is unavailable"))?,
                    helper: path(helper)?,
                    account: account.clone(),
                    bridge: base
                        .credential_bridge
                        .clone()
                        .ok_or_else(|| invalid("Native credential bridge is unavailable"))?,
                });
                options.ca_file = ca_file.as_deref().map(path).transpose()?;
                remote::Transport::Https
            }
            Self::Ssh {
                known_hosts,
                public_identity,
                agent_socket,
            } => {
                options.ssh = Some(remote::SshPolicy {
                    known_hosts: path(known_hosts)?,
                    public_identity: path(public_identity)?,
                    agent_socket: path(agent_socket)?,
                });
                remote::Transport::Ssh
            }
        };
        Ok((options, transport))
    }
}
pub(crate) fn load(db: &Connection) -> Result<Settings> {
    let raw: Option<String> = db
        .query_row(
            "SELECT value FROM state WHERE key='authentication_settings_v1'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    match raw {
        Some(s) if s.len() <= 1024 * 1024 => Ok(serde_json::from_str(&s)?),
        Some(_) => Err(invalid("Authentication settings exceed limit")),
        None => Ok(Settings::default()),
    }
}
pub(crate) fn save(db: &Connection, settings: &Settings) -> Result<()> {
    let raw = serde_json::to_string(settings)?;
    if raw.len() > 1024 * 1024 || settings.profiles.len() > 64 || settings.assignments.len() > 1024
    {
        return Err(invalid("Authentication settings exceed limit"));
    }
    db.execute("INSERT INTO state(key,value) VALUES('authentication_settings_v1',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[raw])?;
    Ok(())
}
pub(crate) fn effective(
    db: &Connection,
    base: &TransportOptions,
    workspace: &WorkspaceId,
    worktree: &WorktreeId,
    remote: &str,
    url: &str,
    transport: &remote::Transport,
) -> Result<(TransportOptions, Option<ProfileBinding>)> {
    let settings = load(db)?;
    let Some(a) = settings
        .assignments
        .iter()
        .find(|a| a.workspace == *workspace && a.worktree == *worktree && a.remote == remote)
    else {
        return Ok((base.clone(), None));
    };
    if a.destination != url {
        return Err(invalid(
            "Assigned authentication destination changed; inspect and review its assignment again",
        ));
    }
    let p = settings
        .profiles
        .iter()
        .find(|p| p.id == a.profile)
        .ok_or_else(|| invalid("Assigned authentication profile is missing"))?;
    let (options, kind) = p.policy.options(base)?;
    if &kind != transport {
        return Err(invalid(
            "Assigned authentication transport does not match destination",
        ));
    }
    let actual = crate::writes::authentication_tools(&options, &kind)?;
    if actual != p.tools {
        return Err(Error::with_kind(
            ErrorKind::StaleReview,
            "Authentication tools or agent changed; review and save the profile again",
        ));
    }
    Ok((
        options,
        Some(ProfileBinding {
            revision: settings.revision,
            id: p.id.clone(),
            name: p.name.clone(),
        }),
    ))
}

/// Explicit local discovery returns suggestions, never selections or authority.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Choices {
    pub helpers: Vec<HelperChoice>,
    pub public_identities: Vec<String>,
    pub known_hosts: Option<String>,
    pub agent_socket: Option<String>,
    pub notes: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HelperChoice {
    pub label: String,
    pub path: String,
}
pub(crate) fn choices(options: &crate::inspection::ReadOptions) -> Result<Choices> {
    let helpers = crate::writes::authentication_helper_choices(options)?;
    let mut result=Choices{helpers,public_identities:vec![],known_hosts:None,agent_socket:None,notes:vec!["Discovery reads only local tool paths, public-key filenames and socket metadata. It does not run a credential helper, query an agent, load keys or contact a server.".into()]};
    if let Some(home) = std::env::var_os("HOME") {
        let ssh = PathBuf::from(home).join(".ssh");
        if std::fs::symlink_metadata(&ssh).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
        {
            for entry in std::fs::read_dir(&ssh)?.take(128) {
                let entry = entry?;
                let p = entry.path();
                if !std::fs::symlink_metadata(&p)?.is_file() {
                    continue;
                }
                if entry.file_name() == "known_hosts" {
                    result.known_hosts = p.to_str().map(str::to_owned);
                } else if p.extension().is_some_and(|e| e == "pub")
                    && result.public_identities.len() < 32
                {
                    if let Some(p) = p.to_str() {
                        result.public_identities.push(p.into());
                    }
                }
            }
            result.public_identities.sort();
        }
    }
    if let Some(socket) = std::env::var_os("SSH_AUTH_SOCK").map(PathBuf::from) {
        if crate::writes::authentication_agent_available(&socket) {
            result.agent_socket = socket.to_str().map(str::to_owned);
        }
    }
    result.notes.push("Choose a nonsecret Git account explicitly. For HTTPS use an existing matching Keychain credential or create it with your approved external Git client; GitFleet never asks for a password or token.".into());
    result.notes.push("Only selected public Ed25519 agent identities are supported. Verify unknown or changed host keys externally; no automatic known_hosts edits.".into());
    Ok(result)
}
