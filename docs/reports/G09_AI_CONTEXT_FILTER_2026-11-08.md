# G09 bounded AI context filtering — November 8, 2026

Disposition: pure Core foundation implemented and automated boundaries passed;
repository capture, exact native transfer review and assisted generation remain next.
This checkpoint does not close AI-010–013 or G09.

`ai/context.rs` prepares one deterministic, ephemeral JSON user payload from bounded
source fragments. It has no provider, filesystem, Git execution or authorization
capability. Payload budget counts actual serialized UTF-8 bytes, including JSON
escaping and provenance framing. Whole-line chunks preserve fragment IDs, origin,
kind and line ranges; local accounting records input/included/redacted/omitted counts
and explicit reasons. Excluded path names and content never enter the sent payload.
Input is capped at 1,024 fragments/16 MiB; payload at 1 KiB–1 MiB and 128 lines per
attributable chunk. Budget accounting does linear input work rather than repeatedly
serializing the growing whole payload.

Sensitive credential paths, configured globs, binary/non-UTF-8 fragments are omitted.
Recognizable credential assignments, bearer headers, private-key blocks, common token
prefixes and suspicious long mixed tokens are conservatively redacted before payload
serialization. Unsafe control/bidirectional text is redacted. Secret screening is
explicitly defense in depth, with false positives and unrecognized secrets possible;
exact payload inspection is required before source-transfer approval. Raw source is
not persisted. Omission/redaction marks coverage incomplete and cannot justify an
automatic commit. Source text, including prompt injection, remains JSON data and
cannot grant Git, Action, endpoint or command capabilities.

Validation: seven focused tests passed: sensitive/binary exclusions, multi-line
private keys and credential removal, injection as data, exact byte/UTF-8/escape bounds,
determinism, line-chunk provenance, invalid input and excluded empty-file coverage.
Strict all-feature/all-target Core Clippy, formatting and diff checks passed.
The initial test-style Clippy failure is retained with the corrected final pass.
Evidence: `docs/validation/g09-ai-context-filter-2026-11-08/`.
No native acceptance, repository-source transmission or provider run occurred here.
