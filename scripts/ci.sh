#!/usr/bin/env bash
# Run the checks of the repository in Docker: format, clippy, tests and the conformance runs.
# Use: scripts/ci.sh
# Each cargo clippy and cargo test command has --locked: Cargo.lock gives the versions, and cargo never changes it.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "== cargo fmt"
"$here/cargo.sh" fmt --all -- --check
echo "== cargo clippy"
"$here/cargo.sh" clippy --locked --workspace --all-targets -- -D warnings
echo "== cargo test"
"$here/cargo.sh" test --locked --workspace
echo "== cargo test without the feature mcp"
"$here/cargo.sh" test --locked -p procurauth-pep -p reference-service --no-default-features
echo "== cargo clippy and cargo test with the feature tls"
"$here/cargo.sh" clippy --locked -p procurauth-pep --all-targets --features tls -- -D warnings
"$here/cargo.sh" test --locked -p procurauth-pep --features tls
echo "== conformance runs: series U, D and E"
# The script has its own time limits inside its container. This limit covers the build too.
timeout 1800 "$here/conformance.sh" all
echo "== all checks passed"
