#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

matrix_src="${root_dir}/benchmark-suite/scripts/testdata/sample-cross-impl-compare-matrix.json"
matrix_target="${tmp}/compare-matrix.json"
entry="${tmp}/trend-note-entry.md"
trend_note="${tmp}/trend-note.md"

cat >"${entry}" <<'EOF'
## Trend Entry (2026-02-13)

| Endpoint | Leader | p99 (ms) | Coverage (%) | Absolute Guard | Baseline Guard |
| --- | --- | ---: | ---: | --- | --- |
| ping | sec4 | 4.20 | 98.00 | pass | pass |
EOF

cat >"${trend_note}" <<'EOF'
# Trend Note
EOF

dry_out="$("${root_dir}/scripts/refresh-closure-evidence-from-ci.sh" \
  --dry-run \
  --repo HubDev-AI/untrusted-compiler \
  --matrix "${matrix_src}" \
  --entry "${entry}" \
  --target-matrix "${matrix_target}" \
  --trend-note "${trend_note}")"

if ! grep -q 'update_cross_impl_matrix_from_ci.sh --repo HubDev-AI/untrusted-compiler --target' <<<"${dry_out}"; then
  echo "dry-run missing M10 update command" >&2
  exit 1
fi

if ! grep -q 'update_trend_note_from_ci.sh --repo HubDev-AI/untrusted-compiler --chapter' <<<"${dry_out}"; then
  echo "dry-run missing M13 update command" >&2
  exit 1
fi

if ! grep -q 'check-milestone-closure.sh --matrix' <<<"${dry_out}"; then
  echo "dry-run missing closure check command" >&2
  exit 1
fi

"${root_dir}/scripts/refresh-closure-evidence-from-ci.sh" \
  --repo HubDev-AI/untrusted-compiler \
  --matrix "${matrix_src}" \
  --entry "${entry}" \
  --target-matrix "${matrix_target}" \
  --trend-note "${trend_note}" >/dev/null

if [ ! -f "${matrix_target}" ]; then
  echo "matrix target not generated" >&2
  exit 1
fi

if ! jq -e '.endpoints[0].compared[] | select(.impl == "rust")' "${matrix_target}" >/dev/null; then
  echo "matrix target missing rust impl" >&2
  exit 1
fi

if ! grep -q '^## Trend Entry (2026-02-13)$' "${trend_note}"; then
  echo "trend note missing imported trend entry" >&2
  exit 1
fi

echo "refresh-closure-evidence-from-ci test passed"
