# G11 CI prerequisite provisioning — November 8, 2026

The workflow pinned Rust but inherited Git and had no macOS GPG provisioning, despite
mandatory Git 2.56 and production OpenPGP positives. Both jobs now build official
Git 2.56.0 into a new runner-temporary prefix using a pinned SHA256 verified against
the [official checksum list](https://www.kernel.org/pub/software/scm/git/sha256sums.asc).
Linux installs declared build headers on its disposable runner. macOS Core explicitly
installs GnuPG with Homebrew and supplies GITFLEET_TEST_GPG and PATH. Short process-local
TMPDIR avoids the diagnosed agent socket limit. Job deadlines are 30 minutes.
No local global package install, tool switch, credentials or hosted action was used.

Bootstrap refuses existing prefixes, verifies digest before extraction/execution, and
allows only regular files/directories plus the three exact repository-internal links
in the pinned source. An initial over-restrictive archive check safely refused those
upstream links; corrected to explicit allowlist. Four tests cover traversal, wrong
roots, unapproved links, special files, duplicate paths and entries below links.

Actual isolated local source build/install passed. Git reports 2.56.0; SHA256 fixture
commit/blob, installed HTTPS helper and declared desktop prerequisites passed.
Workflow YAML parses and all run blocks pass bash -n. Initial shell-check helper
used Ruby filter_map unavailable on this host; map/compact retry passed. That helper
is not shipped in the workflow. Raw compiler log retained under temporary prefix;
committed provenance records source/binary hashes and a raw-log digest.

Hosted CI, Linux build and macOS brew provisioning are not yet executed evidence.
No workflow dispatch/push, signing/notarization/installer or release publication.
G11 remains partial; tool delivery/version policy, exact release platform/identifier,
licenses, install/upgrade/migration/uninstall and update/release invariants remain open.

[Validation evidence](../validation/g11-ci-prerequisites-2026-11-08/).
