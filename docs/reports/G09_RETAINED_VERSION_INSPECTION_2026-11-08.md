# G09 retained version-file inspection — November 8, 2026

Implemented a read-only inspection of the private displaced file belonging to a
validated version Action record. The renderer supplies an operation ID, never a
filesystem path. Core validates the record/workspace and pins every path component
from the app-state directory without following links. Inspection accepts an owned,
single-link regular file of at most 1,024 bytes; unsupported artifacts remain preserved
and the error identifies the fixed recovery location for manual inspection.

The ordinary read-only build exposes the bytes, observed fingerprint, timestamp,
location and comparison with the frozen original. Revoked trust or a moved repository
does not prevent reading its retained recovery artifact. Nothing restores, stages,
retries or deletes a file. Text wraps and paragraphs have scoped vertical spacing.

Validation: focused version suite 11 passed / one explicitly ignored crash worker;
strengthened path guards 2 passed; application suite 29 passed; Core strict Clippy
passed. Complete final desktop runner exited zero: 109 renderer tests, 14 ordinary
host tests, 17 all-feature host tests, types/lint/contracts/format/builds/strict Clippy.
The initial formatting failure remains in desktop-full.txt; successful final runs
are desktop-final.txt and desktop-spacing-final.txt. The prior Action checkpoint's
full five-configuration write matrix was not rerun for this read-only extension.

Native final ordinary artifact displayed the retained six bytes `2.0.0\n`, the exact
fixed operation location, timestamp, fingerprint and original-match notice. Stage
remained disabled. The paragraph spacing/wrapping was inspected in the narrow
composer; this is not full keyboard/VoiceOver/layout acceptance. Cmd+Q exited zero.
Initial native startup briefly showed the window shell before loaded content; it
then rendered without reload. This is not proof the earlier intermittent blank
startup is universally fixed.

Repeatable evidence/oracle:
[validation directory](../validation/g09-retained-version-inspection-2026-11-08/).
verify-native.py passed: all repository bytes/modes, execution journals, trust and
original artifact exact; parent still revoked; source/binary hashes verified;
SQLite integrity ok and exclusive writer lock reopened. First-build manifests are
retained separately from the final spacing build. No new trust or publication.

G09 remains open. Next: optional AI settings/profiles/templates, transport/context,
assisted candidates and separately bounded automatic authority. G10 must protect
retained recovery artifacts from cleanup and restore conflicts.
