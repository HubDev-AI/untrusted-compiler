#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_script="${root_dir}/scripts/print-m17-operator-troubleshooting-matrix.sh"

text_output="$("${matrix_script}")"
if ! printf '%s\n' "${text_output}" | rg -Fq 'M17 Operator Troubleshooting Matrix'; then
  echo "expected troubleshooting matrix text heading" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'artifact_missing'; then
  echo "expected artifact_missing entry in troubleshooting matrix text output" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'closure_pending'; then
  echo "expected closure_pending entry in troubleshooting matrix text output" >&2
  exit 1
fi

json_output="$("${matrix_script}" --format json)"
if ! printf '%s\n' "${json_output}" | jq -e '
  .version == "0.1"
  and [.entries[].id] == [
    "artifact_missing",
    "runflags_shape_drift",
    "trace_header_missing",
    "trace_mismatch",
    "handoff_chapter_token_missing",
    "bundle_branch_missing",
    "closure_pending"
  ]
' >/dev/null; then
  echo "expected deterministic troubleshooting matrix json contract output" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT
if "${matrix_script}" --format yaml >"${tmp_dir}/invalid.log" 2>&1; then
  echo "expected troubleshooting matrix script to fail on unsupported format" >&2
  exit 1
fi
if ! rg -Fq 'unknown format: yaml' "${tmp_dir}/invalid.log"; then
  echo "expected invalid-format diagnostic from troubleshooting matrix script" >&2
  exit 1
fi

echo "m17 operator troubleshooting matrix test passed"
