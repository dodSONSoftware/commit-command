# G09 real Ollama provider acceptance — November 8, 2026

Disposition: selected diagnostics and generated-fixture suggestion checks passed.
This does not close G09 or establish semantic accuracy. No project source was sent,
no repository writes occurred, and no real credentials were used.

The native fixture connected to the user-supplied Ollama endpoint at
`http://10.10.10.72:11434/` and selected `qwen3.5:9b-q4_K_M`. Discovery found the
requested installed model. Configuration was saved only in the disposable acceptance
fixture; the application's default assignment remained on its offline fixture profile.

## Observations

- A fixed synthetic request with the original default generation budgets failed. A
  separate direct protocol probe of the same request completed with `done_reason` `stop`;
  it was not native application acceptance and did not reproduce the failure.
- Repeating the native request confirmed an output token/context limit. Partial output
  was discarded. The provider adapter now reports a bounded actionable error that
  recommends increasing the profile budget or reducing input, then retrying explicitly.
  Other unfinished or unsupported finish reasons are discarded without echoing provider
  values.
- After explicitly raising disposable profile budgets to 8,192 output tokens and 16,384
  context tokens, synthetic inference completed in about 32 seconds. A separate request
  with an unavailable model failed clearly without fallback.
- Native cancellation was observed while the owned curl client was running. The job
  entered the cancelled phase, its result was discarded, and the owned client was reaped.
  Server-side inference cancellation was not independently observed.
- A source-suggestion request used only generated content in a disposable acceptance
  repository, not project source. It completed in about 27 seconds and returned three
  template-valid suggestions. Their claims overinterpreted plain fixture text and
  referenced only two of five reported chunks. Structural validation does not prove
  semantic correctness or complete coverage; human review remains necessary. No
  suggestion was selected or applied.

## Validation and limits

For the completion-limit parser change, the recorded final Core run passed 63 library
and 41 application integration tests. Strict all-feature/all-target Core Clippy passed.
The complete desktop runner exited zero with 123 renderer tests, 20 ordinary host tests
plus one ignored worker, and 23 all-feature host tests plus one ignored worker; its
recorded type, contract, lint, format, build, Clippy and release checks passed. Logs are
in `../validation/g09-ollama-real-provider-2026-11-08/`.

The complete five-configuration Core regression was run earlier against committed AI
source-integration checkpoint `d9d6954` and is recorded separately; it is not claimed
as rerun for this two-file provider parsing change. The final native application's
Quit/exit after the generated suggestions was interrupted, so no final clean post-suggestion
shutdown/restart claim is made. Earlier provider settings, source review, cancellation
and preservation evidence remains in the validation directory.

G09 remains open for hierarchical multi-request analysis, bounded automatic commit
authority, broader compatible-provider/TLS behavior and usability. G02–G11 remain
incomplete overall; this report is a checkpoint, not a release decision.
