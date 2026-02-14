#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
selector_script="${root_dir}/scripts/select-m27-next-slice.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_pass="${tmp_dir}/kickoff-pass.json"
cat > "${kickoff_pass}" <<'JSON'
{
  "version": "0.1",
  "m26Overall": "PASS",
  "primaryFocus": "release",
  "pendingGates": []
}
JSON

matrix_release_top="${tmp_dir}/matrix-release.json"
cat > "${matrix_release_top}" <<'JSON'
{
  "version": "0.1",
  "tracks": [
    {"priority":1,"track":"release","score":75},
    {"priority":2,"track":"runtime","score":66},
    {"priority":3,"track":"editor","score":64}
  ]
}
JSON

pass_json="$(${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${matrix_release_top}" --output "${tmp_dir}/selector-pass.json" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m26Overall == "PASS"
  and .primaryFocus == "release"
  and .pendingGateCount == 0
  and .selectedTrack == "release"
  and .recommendation.id == "M27-S4-release-hardening"
  and .recommendation.closureGate == "M27-C"
' >/dev/null; then
  echo "expected selector to recommend release slice for PASS release-focused M27 inputs" >&2
  exit 1
fi

kickoff_stabilization="${tmp_dir}/kickoff-stabilization.json"
cat > "${kickoff_stabilization}" <<'JSON'
{
  "version": "0.1",
  "m26Overall": "PENDING",
  "primaryFocus": "stabilization",
  "pendingGates": ["M26-E"]
}
JSON

matrix_editor_top="${tmp_dir}/matrix-editor.json"
cat > "${matrix_editor_top}" <<'JSON'
{
  "version": "0.1",
  "tracks": [
    {"priority":1,"track":"editor","score":75},
    {"priority":2,"track":"release","score":64}
  ]
}
JSON

stabilization_json="$(${selector_script} --kickoff-json "${kickoff_stabilization}" --matrix-json "${matrix_editor_top}" --output "${tmp_dir}/selector-stabilization.json" --format json)"
if ! printf '%s\n' "${stabilization_json}" | jq -e '
  .m26Overall == "PENDING"
  and .primaryFocus == "stabilization"
  and .pendingGateCount == 1
  and .selectedTrack == "runtime"
  and .recommendation.id == "M27-S4-stabilization-remediation"
' >/dev/null; then
  echo "expected selector to force stabilization remediation when kickoff is not fully clean" >&2
  exit 1
fi

if ${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${tmp_dir}/missing-matrix.json" --format json >"${tmp_dir}/missing-matrix.log" 2>&1; then
  echo "expected selector to fail when matrix json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M27 priority matrix json:' "${tmp_dir}/missing-matrix.log"; then
  echo "expected missing-matrix diagnostic from M27 selector" >&2
  exit 1
fi

invalid_matrix="${tmp_dir}/invalid-matrix.json"
cat > "${invalid_matrix}" <<'JSON'
{"version":"0.1","tracks":[]}
JSON

if ${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${invalid_matrix}" --format json >"${tmp_dir}/invalid-matrix.log" 2>&1; then
  echo "expected selector to fail for invalid M27 matrix contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M27 priority matrix contract:' "${tmp_dir}/invalid-matrix.log"; then
  echo "expected invalid-matrix diagnostic from M27 selector" >&2
  exit 1
fi

echo "m27 next-slice selector test passed"
