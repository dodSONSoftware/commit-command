#!/bin/sh
# No OS UI automation: live native acceptance is separate and must be permitted.
set -eu
# Core bounds concurrent Git children; avoid fixture contention exhausting deadlines.
: "${RUST_TEST_THREADS:=4}"
export RUST_TEST_THREADS
cd "$(dirname "$0")/.."
python3 ../core/scripts/check-prerequisites.py --desktop
npm run typecheck
npm run lint
npm run contracts -- --check
npm run format:check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo build --manifest-path src-tauri/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --all-features --locked
# Bundled read-only host must remain buildable and enforce its write gate.
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --features custom-protocol --locked
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --features custom-protocol --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --no-default-features --features custom-protocol --locked -- -D warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo build --manifest-path src-tauri/Cargo.toml --release --bin gitfleet-desktop --locked
