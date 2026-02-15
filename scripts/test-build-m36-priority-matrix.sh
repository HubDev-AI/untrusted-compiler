#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_script="${root_dir}/scripts/build-m36-priority-matrix.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_runtime="${tmp_dir}/kickoff-runtime.json"
cat > "${kickoff_runtime}" <<'JSON'
{
  "version": "0.1",
  "kickoffGate": "M36-A",
  "m35Overall": "PASS",
  "convergenceOverall": "PASS",
  "primaryFocus": "runtime",
  "selectedTrack": "runtime",
  "runtimeStatus": "DRY_RUN",
  "pendingGates": [],
  "pendingGateCount": 0
}
JSON

runtime_json="$(${matrix_script} --kickoff-json "${kickoff_runtime}" --output-json "${tmp_dir}/runtime-matrix.json" --output-markdown "${tmp_dir}/runtime-matrix.md" --format json)"
if ! printf '%s\n' "${runtime_json}" | jq -e '
  .version == "0.1"
  and .closureMarker == "M36-B"
  and .kickoffGate == "M36-A"
  and .primaryFocus == "runtime"
  and (.tracks | length) == 3
  and [.tracks[].track] == ["runtime", "release", "editor"]
  and [.tracks[].priority] == [1, 2, 3]
  and .tracks[0].score > .tracks[1].score
' >/dev/null; then
  echo "expected deterministic runtime-focused M36 priority matrix json contract" >&2
  exit 1
fi

if [ ! -f "${tmp_dir}/runtime-matrix.md" ]; then
  echo "expected M36 priority matrix markdown output file" >&2
  exit 1
fi
if ! rg -Fq '| 1 | runtime |' "${tmp_dir}/runtime-matrix.md"; then
  echo "expected markdown M36 matrix to include runtime as first priority row" >&2
  exit 1
fi
if ! rg -Fq '| 2 | release |' "${tmp_dir}/runtime-matrix.md"; then
  echo "expected markdown M36 matrix to include release as second priority row" >&2
  exit 1
fi
if ! rg -Fq '| 3 | editor |' "${tmp_dir}/runtime-matrix.md"; then
  echo "expected markdown M36 matrix to include editor as third priority row" >&2
  exit 1
fi

if ${matrix_script} --kickoff-json "${tmp_dir}/missing-kickoff.json" --format json >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected M36 priority matrix builder to fail when kickoff json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M36 kickoff brief json:' "${tmp_dir}/missing.log"; then
  echo "expected missing-kickoff diagnostic from M36 priority matrix builder" >&2
  exit 1
fi

invalid_kickoff="${tmp_dir}/invalid-kickoff.json"
cat > "${invalid_kickoff}" <<'JSON'
{
  "version": "0.1",
  "primaryFocus": "runtime"
}
JSON

if ${matrix_script} --kickoff-json "${invalid_kickoff}" --format json >"${tmp_dir}/invalid.log" 2>&1; then
  echo "expected M36 priority matrix builder to fail for invalid kickoff contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M36 kickoff brief contract:' "${tmp_dir}/invalid.log"; then
  echo "expected invalid-kickoff contract diagnostic from M36 priority matrix builder" >&2
  exit 1
fi

echo "m36 priority matrix test passed"
