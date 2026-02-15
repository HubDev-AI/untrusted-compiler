#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
planner_script="${root_dir}/scripts/plan-m33-runtime-destub.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_runtime="${tmp_dir}/kickoff-runtime.json"
cat > "${kickoff_runtime}" <<'JSON'
{
  "version": "0.1",
  "m32Overall": "PASS",
  "convergenceOverall": "PASS",
  "primaryFocus": "runtime",
  "pendingGates": []
}
JSON

matrix_runtime="${tmp_dir}/matrix-runtime.json"
cat > "${matrix_runtime}" <<'JSON'
{
  "version": "0.1",
  "tracks": [
    {"priority":1,"track":"runtime","score":81},
    {"priority":2,"track":"release","score":66},
    {"priority":3,"track":"editor","score":64}
  ]
}
JSON

runtime_json="$(${planner_script} --kickoff-json "${kickoff_runtime}" --matrix-json "${matrix_runtime}" --output "${tmp_dir}/runtime-plan.json" --format json)"
if ! printf '%s\n' "${runtime_json}" | jq -e '
  .version == "0.1"
  and .m32Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .primaryFocus == "runtime"
  and .selectedTrack == "runtime"
  and .stabilizationMode == false
  and .closureGate == "M33-C"
  and (.plan | length) == 5
  and .plan[0].id == "M33-S4-runtime-db-destub"
  and .plan[0].order == 1
' >/dev/null; then
  echo "expected deterministic runtime-first M33 de-stub plan contract" >&2
  exit 1
fi

kickoff_stabilization="${tmp_dir}/kickoff-stabilization.json"
cat > "${kickoff_stabilization}" <<'JSON'
{
  "version": "0.1",
  "m32Overall": "PENDING",
  "convergenceOverall": "PENDING",
  "primaryFocus": "stabilization",
  "pendingGates": ["M32-G"]
}
JSON

matrix_editor="${tmp_dir}/matrix-editor.json"
cat > "${matrix_editor}" <<'JSON'
{
  "version": "0.1",
  "tracks": [
    {"priority":1,"track":"editor","score":78},
    {"priority":2,"track":"runtime","score":70}
  ]
}
JSON

stabilization_json="$(${planner_script} --kickoff-json "${kickoff_stabilization}" --matrix-json "${matrix_editor}" --output "${tmp_dir}/stabilization-plan.json" --format json)"
if ! printf '%s\n' "${stabilization_json}" | jq -e '
  .m32Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and .selectedTrack == "runtime"
  and .stabilizationMode == true
  and .plan[0].id == "M33-S4-runtime-validator-destub"
  and (.plan[] | select(.domain == "secrets")) != null
' >/dev/null; then
  echo "expected stabilization-mode M33 de-stub plan to force validator-first runtime ordering" >&2
  exit 1
fi

if ${planner_script} --kickoff-json "${kickoff_runtime}" --matrix-json "${tmp_dir}/missing-matrix.json" --format json >"${tmp_dir}/missing-matrix.log" 2>&1; then
  echo "expected M33 de-stub planner to fail when matrix json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M33 priority matrix json:' "${tmp_dir}/missing-matrix.log"; then
  echo "expected missing-matrix diagnostic from M33 de-stub planner" >&2
  exit 1
fi

invalid_kickoff="${tmp_dir}/invalid-kickoff.json"
cat > "${invalid_kickoff}" <<'JSON'
{"version":"0.1","primaryFocus":"runtime","pendingGates":[]}
JSON

if ${planner_script} --kickoff-json "${invalid_kickoff}" --matrix-json "${matrix_runtime}" --format json >"${tmp_dir}/invalid-kickoff.log" 2>&1; then
  echo "expected M33 de-stub planner to fail for invalid kickoff contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M33 kickoff brief json contract:' "${tmp_dir}/invalid-kickoff.log"; then
  echo "expected invalid-kickoff diagnostic from M33 de-stub planner" >&2
  exit 1
fi

echo "m33 runtime de-stub planner test passed"
