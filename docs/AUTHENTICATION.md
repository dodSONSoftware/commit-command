# GitFleet authentication on macOS

Authentication is explicit and per remote. GitFleet uses existing credential helpers and SSH agents; it does not collect passwords/tokens, manage a credential store, load private keys or authenticate from background monitoring. Ordinary development builds remain read-only until the remaining write acceptance is completed. The steps below describe the enabled acceptance build and intended first-release behavior.

## Configure a profile

1. Open a repository, expand **Remotes and upstream**, and choose **Inspect remote destinations**. Check the actual URL and remote name.
2. Open **Authentication profiles and assignments**. **Find local authentication tools** suggests the installed Git macOS Keychain helper, public-key filenames, known-hosts file and a usable advertised agent socket. Discovery does not query credentials/agent keys or select an account/profile automatically.
3. Choose a method and a descriptive profile name. Use the suggestion selectors or native **Browse** buttons. Paths may also be entered explicitly; text correction is disabled so literal characters survive.
4. For HTTPS, select the credential helper executable and enter the nonsecret Git account. An optional CA file selects private-server trust without changing the system trust store. Never enter a password or PAT into the account field.
5. For SSH, select a verified known-hosts file, one public Ed25519 identity (`.pub`) and the agent socket. Load/unlock the matching private identity with your external agent's normal tools. GitFleet does not load it or infer a default identity.
6. **Review profile settings**, inspect the exact stored change/tools, and approve the independent native dialog. Saving contacts no server, performs no credential lookup and grants no repository trust.
7. Select the remote and saved profile, then **Review remote assignment**. Check the resolved URL/worktree in the independent review. Assignment is scoped to that worktree and remote; it is not inherited by other repositories/remotes.

Each observation/download has its own review and native approval, showing destination, account/profile, helper/agent bindings and verification policy. Downloading and offline tracking-reference import have separate approvals. A past observation is not another lookup grant.

## HTTPS credentials and native prompts

The validated built-in discovery choice is Git's `git-credential-osxkeychain` on macOS. Provision the appropriate existing credential with your approved external Git client or Keychain tools. GitFleet sends protocol, host/port, repository path and selected account to **get** only. Matching a credential stored by another client depends on that helper's path/account matching rules. If none is found, check the exact account, path, scheme and port externally and prepare a fresh review.

The helper runs unsandboxed as the user and uses the native host's protected HOME, which is bound into the review. Git's network repository/HOME/configuration remain isolated. This allows the existing Keychain helper to find its native store without re-enabling user Git configuration, proxies, arbitrary helper chains or SSH commands. Native unlock/access prompts can occur during approved lookup. Decline the prompt or cancel the operation to stop; rejection never forwards Git store/erase to the helper. GitFleet does not create or replace the server credential for you.

Gitea 1.25.0 HTTPS password and repository-read PAT authentication have been exercised against a disposable real server with the actual macOS helper, followed by separate download/import. This does not establish compatibility with every Gitea version, authentication policy or private deployment. The user's private server was not contacted.

## SSH verification

Only the selected public Ed25519 identity through the selected agent is supported in the current envelope. Host keys are strictly checked against the selected protected known-hosts snapshot. Unknown/changed keys stop execution: verify the fingerprint externally with the server administrator, update the selected file externally, then review/save the profile again. GitFleet never accepts a host key automatically.

Password/keyboard-interactive fallback, private-key loading, implicit certificates, forwarding and arbitrary SSH configuration/proxy commands remain unsupported. Other agent/helper products are not automatically discovered as validated integrations. Explicit custom get-only helpers remain trusted programs; unsupported responses/configurations fail closed. OAuth/browser/terminal-login and other auth mechanisms are not claimed as validated compatibility.

## Changes, failures and cancellation

- Profile, destination, helper/bridge/HOME, CA, public identity, known-hosts or agent-binding changes can require a fresh profile/assignment and operation review. Application upgrades change the bridge binding; edit/review/save affected profiles before further network operations. Existing history remains readable.
- An assigned profile cannot be deleted until its assignments are explicitly removed. Selecting a remote shows its saved assignment; choosing another profile changes only the draft until native approval.
- Locked/denied/missing credentials or authentication rejection stop the read without an alternate helper/account or automatic erase. Resolve the issue externally and prepare a fresh operation.
- Cancellation and Quit terminate the owned Git/bridge/helper process group. Remote reads do not roll back already observed server contact, and cancellation of later publication must be assessed from its durable result/recovery state.
- HTTPS verifies certificates/hostnames, rejects redirects and uses Basic username/password-or-PAT authentication in this supported envelope. Secrets stay out of renderer IPC, arguments/environment, profiles and application journals; private helper state is discarded.
