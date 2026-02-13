#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
matrix="${root_dir}/testdata/sample-trend-compare-matrix.json"
fallback_matrix="${root_dir}/testdata/sample-trend-compare-matrix-wrk.json"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

out="${tmp}/trend-note.md"
"$root_dir/render_trend_note_entry.sh" \
  "$matrix" \
  --date 2026-02-13 \
  --baseline-dir "${root_dir}/../baselines" \
  --out "$out" >/dev/null

if ! grep -q '^## Trend Entry (2026-02-13)$' "$out"; then
  echo "missing trend entry heading" >&2
  exit 1
fi
if ! grep -q '^- Run mode: constant-rate$' "$out"; then
  echo "missing constant-rate run mode line" >&2
  exit 1
fi
if ! grep -q '^- Generators: wrk2$' "$out"; then
  echo "missing default wrk2 generator summary line" >&2
  exit 1
fi

if ! grep -q '| ping | node | 18.20 | 90.00 | pass | pass |' "$out"; then
  echo "missing ping pass row" >&2
  exit 1
fi

if ! grep -q '| decode | node | 92.00 | 55.00 | fail | fail |' "$out"; then
  echo "missing decode fail row" >&2
  exit 1
fi

if ! grep -q 'Overall absolute guard status: fail' "$out"; then
  echo "missing absolute guard summary" >&2
  exit 1
fi

if ! grep -q 'Overall baseline guard status: fail' "$out"; then
  echo "missing baseline guard summary" >&2
  exit 1
fi

fallback_out="${tmp}/trend-note-wrk.md"
"$root_dir/render_trend_note_entry.sh" \
  "$fallback_matrix" \
  --date 2026-02-13 \
  --baseline-dir "${root_dir}/../baselines" \
  --out "$fallback_out" >/dev/null

if ! grep -q '| ping | go | 1.75 | n/a | n/a | n/a |' "$fallback_out"; then
  echo "missing non-constant-rate n/a row" >&2
  exit 1
fi
if ! grep -q '^- Run mode: non-constant-rate$' "$fallback_out"; then
  echo "missing non-constant run mode line" >&2
  exit 1
fi
if ! grep -q '^- Generators: wrk$' "$fallback_out"; then
  echo "missing wrk generator summary line" >&2
  exit 1
fi

echo "render_trend_note_entry test passed"
