# Unconfirmed batch publication display — November 8, 2026

A real native G05 two-repository batch exposed misleading presentation: alpha's ref
had advanced, but a post-update test-fault guard error left a durable uncertain outcome
with published=false. Bulk displayed Not published and audit showed Published: false.
That field is lack of recorded confirmation, not proof the ref stayed unchanged.

Added one shared publication-status formatter for bulk rows and individual/batch
audit outcomes. Published records remain Published; uncertain/publication-boundary
records say Publication unconfirmed — inspect guarded recovery; known pre-publication
failures/cancellation remain Not published. Missing outcome gets No publication result
recorded. Raw recorded phase-event flags remain available in the historical details.
No authority, journal, retry, recovery or execution behavior changed.

Focused status regression covers uncertain/reference-updated false flags, confirmed
publication needing reconciliation, cancellation and missing result. Parameterized
bulk integration covers blocked and uncertain outcomes; hostile reason text remains
escaped. Full desktop gate passed 91 renderer, 13 default-host, 16 all-feature host,
plus type/lint/contracts/format/builds and strict Clippy. Native visual verification
of the new label remains pending the corrected host-crash fixture.

The attempted crash did NOT kill the host: fixture preparation supplied the native
host marker but omitted Core's distinct repository marker/layout and temp policy.
Core refused test-fault injection after its ref update; batch stopped before beta.
This was a preparation error, not a host-crash pass. Native guarded recovery inspected
exact ref evidence, stated Commit is published, and after separate native approval
promoted only the owned index. Result complete/published/reconciled, one alpha commit,
no hook or commit replay. Beta/control retained exact bytes/modes, exactly the two
specifically authorized trusts, integrity and writer-lock reopen passed; app exit 0.
An initial evidence check used index_reconciled instead of stored reconciled; corrected
read-only oracle without changing operation data. Actual crash/restart acceptance
remains open and needs a correctly marked new fixture. No guard was relaxed.

[Evidence](../validation/g05-native-batch-crash-2026-11-08/) retains failed-attempt
fixture/binary/features, native events, exact before/after and recovered records,
validation and proof. First publication: cb58233b3e42eddd49f4cdd85422a6210e54a525.
Batch: 1f3fa2a3-7e4e-4a79-ae7e-c7d23c2e971e. Recovery operation:
a2375ade-76d4-4bea-83ef-e8ddcd35b04f. No push or product write-default change.
