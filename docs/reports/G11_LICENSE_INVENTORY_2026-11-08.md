# Locked dependency declaration inventory — November 8, 2026

Added `app/core/scripts/collect-license-inventory.py`, a read-only offline inventory
of both locked Rust dependency graphs (all features for the named target) and the
complete npm lock graph. It records package/version/source, declared license or
license-file presence, first-party identity, graph membership, npm dev/optional flags,
integrity and SHA-256 hashes of the three lockfiles. Linked npm packages and unexpected
lock formats fail rather than being silently omitted. No dependency/lockfile changes.

Run from the checkout:

    python3 app/core/scripts/collect-license-inventory.py --target aarch64-apple-darwin --output /private/tmp/license-inventory.json

Actual inventory has 580 records: 262 deduplicated Cargo packages across the two
graphs and 318 npm lock paths, including the first-party root. All third-party records
have a license or license-file declaration. The npm root gitfleet-desktop lacks a
package metadata declaration; this does not determine its legal status or override
repository LICENSE files. Product-wide licensing/distribution policy remains an open
G11 decision; no license metadata was guessed or changed.

Two actual offline runs produced byte-identical JSON. Verification checks counts,
missing declarations and output digest. Initial unfiltered all-platform Cargo metadata
could not run offline because the Linux inotify source was not cached; the supported
explicit target filter succeeded. Neither Linux nor Windows inventory is accepted.
The inventory scope includes test/development and optional npm platform packages;
it is intentionally broader than the final shipping closure. It contains metadata
only, not license texts, attribution notices or a license compatibility decision.
Final packaging must determine actual bundled components and include their required
texts/notices, including branding assets and any delivered external tools. Hosted
CI, installer, signing/notarization and distribution gates remain open.

[Evidence](../validation/g11-license-inventory-2026-11-08/) contains input provenance,
inventory, first failed attempt, successful run and repeat verification. No network,
credential access, installation, package publication or application execution used.
