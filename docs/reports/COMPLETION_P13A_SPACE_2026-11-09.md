# Completion P13A destination free-space preflight

Private archive publication now validates the manifest and checks free space through fstatvfs on its held destination descriptor before creating the exclusive temporary file. It requires declared stored content size plus32 MiB for bounded container metadata and16 MiB reserved for live-state journal/cleanup. The archive reader and publisher share the format-overhead constant. Arithmetic uses128-bit available/required values so filesystem counters cannot wrap into a false success. Invalid allocation units and failed filesystem queries refuse publication.

Insufficient space gives an actionable free-space/destination message. This is an advisory preflight, not a reservation against other writers or a guarantee that future writes/fsync succeed. Existing no-clobber publication, cancellation, exact hashing, file/directory sync and partial-file cleanup remain authoritative; there is no rotation or success on failure.

Seventeen focused backup tests passed, including independent exact threshold/one-byte-below/zero/large-counter oracles and existing real private-directory publication/preservation tests. Strict all-feature/all-target Core Clippy passed. Evidence:docs/validation/completion-p13a-space-2026-11-09/.

P13A/BACKUP05 remains partial: no actual full filesystem/quota/fsync failure or process-crash facility was exercised. Coherent owner capture, recovery-artifact inventory, migration adoption, extraction/restore controller and general backup UI remain unfinished. No new durable schema, cleanup policy or release acceptance.
