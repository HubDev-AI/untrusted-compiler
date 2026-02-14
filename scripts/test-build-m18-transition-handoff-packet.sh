#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
packet_script="${root_dir}/scripts/build-m18-transition-handoff-packet.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_json="${tmp_dir}/kickoff.json"
cat > "${kickoff_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "failedStep": "none",
  "frictionCount": 0
}
JSON

matrix_json="${tmp_dir}/matrix.json"
cat > "${matrix_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "tracks": [
    {"priority":1,"track":"editor","score":65}
  ]
}
JSON

selector_json="${tmp_dir}/selector.json"
cat > "${selector_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendation": {
    "id": "M18-S6-runtime-confidence-hardening",
    "closureGate": "M18-C"
  }
}
JSON

convergence_json="${tmp_dir}/convergence.json"
cat > "${convergence_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "tracks": [
    {"track":"editor","gate":"M18-D","status":"PASS"},
    {"track":"release","gate":"M18-E","status":"PASS"},
    {"track":"runtime","gate":"M18-F","status":"PASS"}
  ]
}
JSON

output_dir="${tmp_dir}/packet"
packet_json="$(${packet_script} \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --selector-json "${selector_json}" \
  --convergence-json "${convergence_json}" \
  --output-dir "${output_dir}" \
  --format json)"

if ! printf '%s\n' "${packet_json}" | jq -e '
  .version == "0.1"
  and .summary.kickoffOverall == "PASS"
  and .summary.selectorTrack == "runtime"
  and .summary.selectorRecommendationId == "M18-S6-runtime-confidence-hardening"
  and .summary.convergenceOverall == "PASS"
  and .artifacts.kickoff == "kickoff.json"
  and .artifacts.priorityMatrix == "priority-matrix.json"
  and .artifacts.selector == "selector.json"
  and .artifacts.convergence == "convergence.json"
' >/dev/null; then
  echo "expected deterministic transition handoff packet json contract" >&2
  exit 1
fi

for required in kickoff.json priority-matrix.json selector.json convergence.json handoff-packet.json; do
  if [ ! -f "${output_dir}/${required}" ]; then
    echo "expected transition packet artifact ${required}" >&2
    exit 1
  fi
done

if ${packet_script} \
  --kickoff-json "${tmp_dir}/missing-kickoff.json" \
  --matrix-json "${matrix_json}" \
  --selector-json "${selector_json}" \
  --convergence-json "${convergence_json}" \
  --output-dir "${tmp_dir}/missing" \
  --format json >"${tmp_dir}/missing-kickoff.log" 2>&1; then
  echo "expected packet builder to fail when kickoff json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing kickoff json:' "${tmp_dir}/missing-kickoff.log"; then
  echo "expected missing-kickoff diagnostic from transition packet builder" >&2
  exit 1
fi

invalid_convergence_json="${tmp_dir}/convergence-invalid.json"
cat > "${invalid_convergence_json}" <<'JSON'
{"version":"0.1","overall":"PASS","tracks":[{"track":"editor"}]}
JSON

if ${packet_script} \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --selector-json "${selector_json}" \
  --convergence-json "${invalid_convergence_json}" \
  --output-dir "${tmp_dir}/invalid-convergence" \
  --format json >"${tmp_dir}/invalid-convergence.log" 2>&1; then
  echo "expected packet builder to fail for invalid convergence contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid convergence json contract:' "${tmp_dir}/invalid-convergence.log"; then
  echo "expected invalid-convergence diagnostic from transition packet builder" >&2
  exit 1
fi

echo "m18 transition handoff packet test passed"
