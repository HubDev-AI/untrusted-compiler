#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_script="${root_dir}/scripts/build-m26-priority-matrix.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_runtime="${tmp_dir}/kickoff-runtime.json"
cat > "${kickoff_runtime}" <<'JSON'
{
  "version": "0.1",
  "m25Overall": "PASS",
  "primaryFocus": "runtime",
  "pendingGates": []
}
JSON

runtime_json="$(${matrix_script} --kickoff-json "${kickoff_runtime}" --output-json "${tmp_dir}/runtime-matrix.json" --output-markdown "${tmp_dir}/runtime-matrix.md" --format json)"
if ! printf '%s\n' "${runtime_json}" | jq -e '
  .version == "0.1"
  and .primaryFocus == "runtime"
  and .m25Overall == "PASS"
  and .pendingGateCount == 0
  and (.tracks | length) == 3
  and .tracks[0].track == "runtime"
' >/dev/null; then
  echo "expected deterministic runtime-focused M26 priority matrix" >&2
  exit 1
fi

if ! rg -Fq '| 1 | runtime |' "${tmp_dir}/runtime-matrix.md"; then
  echo "expected markdown M26 matrix to include runtime as first priority for runtime focus" >&2
  exit 1
fi

kickoff_stabilization="${tmp_dir}/kickoff-stabilization.json"
cat > "${kickoff_stabilization}" <<'JSON'
{
  "version": "0.1",
  "m25Overall": "PENDING",
  "primaryFocus": "stabilization",
  "pendingGates": ["M25-E"]
}
JSON

stabilization_json="$(${matrix_script} --kickoff-json "${kickoff_stabilization}" --output-json "${tmp_dir}/stabilization-matrix.json" --output-markdown "${tmp_dir}/stabilization-matrix.md" --format json)"
if ! printf '%s\n' "${stabilization_json}" | jq -e '
  .primaryFocus == "stabilization"
  and .m25Overall == "PENDING"
  and .pendingGateCount == 1
  and .tracks[0].track == "runtime"
  and .tracks[0].score > .tracks[1].score
' >/dev/null; then
  echo "expected stabilization-focused M26 priority matrix to force runtime top priority" >&2
  exit 1
fi

if ${matrix_script} --kickoff-json "${tmp_dir}/missing-kickoff.json" --format json >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected M26 priority matrix builder to fail when kickoff json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M26 kickoff brief json:' "${tmp_dir}/missing.log"; then
  echo "expected missing-kickoff diagnostic from M26 priority matrix builder" >&2
  exit 1
fi

echo "m26 priority matrix test passed"
