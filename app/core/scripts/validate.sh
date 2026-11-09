#!/bin/sh
# All Git writes performed by these checks belong to new disposable fixtures.
set -eu
# Core deliberately permits four subprocesses. Bound the fixture runner too so
# unrelated test setup cannot exhaust a read deadline while waiting for a permit.
: "${RUST_TEST_THREADS:=4}"
export RUST_TEST_THREADS
cd "$(dirname "$0")/.."
evidence_dir=${1:-target/m3-evidence}
mkdir -p "$evidence_dir"
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check-prerequisites.py
python3 scripts/verify-scaffold.py
cargo fmt --all -- --check
cargo build --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --features experimental-writes --locked
cargo test --workspace --all-targets --features experimental-writes --locked
cargo clippy --workspace --all-targets --features experimental-writes --locked -- -D warnings
if [ "$(uname -s)" = Darwin ]; then
    # A missing signer fails the M2 gate; it never silently becomes a skip.
    GITFLEET_TEST_GPG=${GITFLEET_TEST_GPG:-$(command -v gpg || command -v gpg2 || true)}
    : "${GITFLEET_TEST_GPG:?M2 macOS validation requires an installed GPG executable}"
    export GITFLEET_TEST_GPG
    cargo build --workspace --all-targets --features controlled-writes --locked
    cargo test --workspace --all-targets --features controlled-writes --locked
    cargo clippy --workspace --all-targets --features controlled-writes --locked -- -D warnings
    cargo build --workspace --all-targets --features application-services --locked
    cargo test --workspace --all-targets --features application-services --locked
    cargo clippy --workspace --all-targets --features application-services --locked -- -D warnings
    cargo build --workspace --all-targets --all-features --locked
    cargo test --workspace --all-targets --all-features --locked
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
else
    echo 'UNSUPPORTED: M2 controlled writes and crash matrix require local macOS/APFS; not executed.'
fi
cargo run --locked --features experimental-writes --bin gitfleet-lab -- --output "$evidence_dir/lab-results.json"
python3 - "$evidence_dir" <<'ENVIRONMENT'
import json
import os
import platform
import subprocess
import sys
from pathlib import Path
metadata = {'platform': platform.platform(), 'architecture': platform.machine(), 'rust_test_threads': os.environ.get('RUST_TEST_THREADS')}
for command in ['git', 'rustc', 'cargo']:
    metadata[command] = subprocess.check_output([command, '--version'], text=True).strip()
if os.environ.get('GITFLEET_TEST_GPG'):
    metadata['m2_gpg'] = subprocess.check_output([os.environ['GITFLEET_TEST_GPG'], '--version'], text=True).splitlines()[:3]
(Path(sys.argv[1]) / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
ENVIRONMENT
