# G09 native-reviewed optional provider diagnostics — November 8, 2026

Implemented Ollama and OpenAI-compatible local-provider discovery plus synthetic
inference, using current unsaved connection/profile/template editors. Discovery
populates searchable installed-model suggestions; manual IDs remain supported and
unreported IDs are preserved with an availability notice. Diagnostic output shows
reported timing/token metadata; it cannot select a manual message, stage, commit,
run an Action or authorize future requests. Model identifiers disable browser
capitalization/autocorrection after the preceding native test exposed an unwanted
case change. No saved settings are required to test an unsaved draft.

Each request freezes the exact route/body/model/options/fingerprint, then uses the
existing privileged Core/native confirmation capability. Native decline sends no
request. Core alone marks this provider read as permitted in an ordinary read-only
build; the flag defaults false, cannot be supplied through request JSON, and all
repository-write reviews remain gated. The host checks the trusted current display
after issuing the native capability. Saved AI settings drift invalidates the review.
There is no durable/reusable endpoint execution permission or automatic inference.

MacOS adapter uses fixed `/usr/bin/curl`, `-q` first, clean environment, no proxy,
fixed GET/POST paths, protocol restrictions, verified HTTPS defaults and no redirect
following. Approved payload and optional ephemeral auth are passed in bounded stdin
configuration, never source-bearing argv or temporary files. No shell runs. The
existing bounded process-group transport handles cancellation, output and total
deadlines, including execution-slot wait. Responses are capped at 1 MiB; inference
content at 64 KiB. Raw error bodies/stderr never reach errors. Unsupported parameters,
partial inference, changed model, tool/function calls, malformed/oversized JSON and
HTTP redirects/auth errors refuse without fallback. Empty tool lists are accepted
as zero tool invocations. Exact parser capabilities remain a bounded subset; no
thinking/keepalive/streaming capability is invented.

Provider work runs in independent ephemeral jobs, outside the serialized application/
storage owner. Global/per-destination request limits count every request and default
to one each. Lowered limits govern subsequent admission; active slots are held until
transport cleanup. Jobs/results are not serialized into workspace/app state. Epoch
reset and shutdown cancel all jobs/discard results; ordinary restart resumes none.
Shutdown waits for owned provider children with a bounded error instead of blocking
indefinitely. Cancellation affects only the chosen AI request. Model-discovery results
from another endpoint cannot populate the current connection's model suggestions.

Validation: final application-services library 48 passed, application integration
31 passed, strict all-feature/all-target Core Clippy passed. Final complete desktop
runner exited zero: 116 renderer, 14 ordinary host, 17 all-feature host plus types,
lint, contracts, format, builds and strict Clippy. Seven focused editor/diagnostic
renderer tests passed. Core actual loopback tests cover both routes/adapters, exact
JSON including hostile config-like text, synthetic bearer auth over stdin, bounded
results/errors, redirects with an uncontacted target, actual held cancellation/timeout,
expired authority, concurrency and reset cleanup. Application native-capability test
covers decline/no request, prompt return while network held, independent workspace
read, cancellation/shutdown and exact repository bytes. Final checked source boxes
only the large synthetic diagnostic input fields; earlier failed host-Clippy size
logs remain. No test or lint was disabled. Full five-configuration write/signing
matrix is prior dated evidence, not rerun for this provider-read checkpoint.

Native ordinary artifact against the owned loopback-only provider:

- Startup/settings opening and actual native decline: zero HTTP requests.
- Fresh exact native approval: one model discovery, showing `fixture-model`.
- Fresh approval: one synthetic inference using lowercase exact model ID and the
  unsaved `UNSAVED SYNTHETIC SYSTEM MARKER` system message. Result showed synthetic
  message JSON and the fixture-reported 42 input/12 output tokens. No message applied.
- Another request was proven received and held at the server before explicit Cancel;
  UI showed cancelled and the pre-existing manual message remained exact.
- A fourth total HTTP request was proven running in both server and owned curl PID.
  Main-window local refresh completed while it remained held. Cmd+Q exited zero;
  exact host/curl PIDs were gone, writer lock reopened. Server handlers were released
  only after host exit. Exact-artifact ordinary restart showed no restored AI jobs,
  preserved the manual draft/read-only controls, issued no HTTP replay and exited zero.
- Exactly four requests total: one discovery, three synthetic inferences. All POST
  bodies match the unsaved synthetic input; no repository paths/content or credentials.
  AI settings files are byte-exact: unsaved diagnostic fields were never saved.

[Evidence and repeatable repository oracle](../validation/g09-ai-provider-diagnostics-2026-11-08/):
source/artifact hashes, exact synthetic request log, held/cancel/Quit/PID proofs,
settings hashes and final proof. All managed bytes/modes/journals/trust and privately
retained version original exact; parent remains revoked; SQLite integrity ok; lock
reopened. No new trust, Git publication, external provider, real credentials, model
installation or ordinary repository-write enablement. Owned server stopped afterward.

Technical contracts verified against primary references on this date:
[Ollama introduction](https://docs.ollama.com/api/introduction),
[model discovery](https://docs.ollama.com/api/tags),
[chat](https://docs.ollama.com/api/chat),
[LM Studio compatibility](https://lmstudio.ai/docs/developer/openai-compat),
[chat completions](https://lmstudio.ai/docs/developer/openai-compat/chat-completions),
[curl options/configuration](https://curl.se/docs/manpage.html).

Remaining G09: native secure authentication, verified TLS/actual supported-provider
acceptance, screened exact repository context, hierarchical coverage/candidates,
separately bounded automatic authority and broader native/accessibility cases. This
synthetic transport checkpoint does not close those gates or imply semantic message
validation, secret-screening completeness or a release readiness claim.
