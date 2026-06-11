#!/usr/bin/env bash
# Remove Rust build output and common local temp files.
# Run from repo root:  ./scripts/clean.sh

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

echo "Cleaning Smart-Road (root: $root)"

if command -v cargo >/dev/null 2>&1; then
  echo "-> cargo clean"
  cargo clean
else
  echo "warning: cargo not found; removing target/ manually." >&2
  rm -rf target
fi

echo "-> removing __pycache__, .pytest_cache, *.pyc, .DS_Store, Thumbs.db"
find "$root" -type d -name '__pycache__' -prune -exec rm -rf {} + 2>/dev/null || true
find "$root" -type d -name '.pytest_cache' -prune -exec rm -rf {} + 2>/dev/null || true
find "$root" -type f -name '*.pyc' -delete 2>/dev/null || true
find "$root" -type f -name '.DS_Store' -delete 2>/dev/null || true
find "$root" -type f -name 'Thumbs.db' -delete 2>/dev/null || true

echo "Done."
