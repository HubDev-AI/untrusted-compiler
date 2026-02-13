#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

good_matrix="${repo_root}/benchmark-suite/scripts/testdata/sample-cross-impl-compare-matrix.json"
warn_matrix="${repo_root}/benchmark-suite/scripts/testdata/sample-trend-compare-matrix-wrk.json"

good_out="$("${repo_root}/scripts/check-benchmark-evidence-quality.sh" --matrix "${good_matrix}")"
if ! grep -q 'overall: PASS' <<<"${good_out}"; then
  echo "expected PASS quality status for baseline sample matrix" >&2
  exit 1
fi

warn_out="$("${repo_root}/scripts/check-benchmark-evidence-quality.sh" --matrix "${warn_matrix}")"
if ! grep -q 'overall: WARN' <<<"${warn_out}"; then
  echo "expected WARN quality status for non-constant-rate matrix" >&2
  exit 1
fi
if ! grep -q 'leader non-constant-rate run' <<<"${warn_out}"; then
  echo "expected non-constant-rate warning in quality output" >&2
  exit 1
fi

if "${repo_root}/scripts/check-benchmark-evidence-quality.sh" --matrix "${warn_matrix}" --fail-on-warning >/dev/null 2>&1; then
  echo "expected --fail-on-warning to fail for warn matrix" >&2
  exit 1
fi

echo "check-benchmark-evidence-quality test passed"
