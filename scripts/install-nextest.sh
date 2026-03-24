#!/usr/bin/env bash
# Default version matches taiki-e/install-action manifest "latest" for cargo-nextest.
# Override: CARGO_NEXTEST_VERSION=0.9.x ./scripts/install-nextest.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
VERSION="${CARGO_NEXTEST_VERSION:-0.9.132}"
exec cargo install cargo-nextest --locked --version "$VERSION"
