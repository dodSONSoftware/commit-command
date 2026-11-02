#!/bin/sh
# No OS UI automation: live native acceptance is separate and must be permitted.
set -eu
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
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo build --manifest-path src-tauri/Cargo.toml --release --bin gitfleet-desktop --locked
