#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
packet_script="${root_dir}/scripts/build-m35-transition-handoff-packet.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_json="${tmp_dir}/kickoff.json"
cat > "${kickoff_json}" <<'JSON'
{
  "version": "0.1",
  "m34Overall": "PASS",
  "convergenceOverall": "PASS",
  "primaryFocus": "runtime",
  "pendingGates": []
}
JSON

matrix_json="${tmp_dir}/matrix.json"
cat > "${matrix_json}" <<'JSON'
{
  "version": "0.1",
  "tracks": [
    {"priority":1,"track":"runtime","score":82},
    {"priority":2,"track":"release","score":63}
  ]
}
JSON

plan_json="${tmp_dir}/plan.json"
cat > "${plan_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "closureGate": "M35-C",
  "plan": [
    {"order":1,"id":"M35-S4-runtime-validator-destub","title":"Validator de-stub","domain":"validators"}
  ]
}
JSON

runtime_execution_json="${tmp_dir}/runtime-execution.json"
cat > "${runtime_execution_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSlice": {
    "id": "M35-S4-runtime-validator-destub",
    "domain": "validators"
  },
  "executionStatus": "DRY_RUN",
  "closureGate": "M35-D"
}
JSON

packet_json="$(${packet_script} \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --output-dir "${tmp_dir}/packet" \
  --format json)"

if ! printf '%s\n' "${packet_json}" | jq -e '
  .version == "0.1"
  and .summary.kickoffPrimaryFocus == "runtime"
  and .summary.planSelectedTrack == "runtime"
  and .summary.planSelectedSliceId == "M35-S4-runtime-validator-destub"
  and .summary.runtimeStatus == "DRY_RUN"
  and .summary.convergenceOverall == "PASS"
  and .artifacts.kickoff == "kickoff.json"
  and .artifacts.priorityMatrix == "priority-matrix.json"
  and .artifacts.runtimePlan == "runtime-plan.json"
  and .artifacts.runtimeExecution == "runtime-execution.json"
  and .artifacts.convergence == "convergence.json"
  and .closureGate == "M35-F"
' >/dev/null; then
  echo "expected deterministic M35 transition packet json contract" >&2
  exit 1
fi

for required in \
  "${tmp_dir}/packet/handoff-packet.json" \
  "${tmp_dir}/packet/kickoff.json" \
  "${tmp_dir}/packet/priority-matrix.json" \
  "${tmp_dir}/packet/runtime-plan.json" \
  "${tmp_dir}/packet/runtime-execution.json" \
  "${tmp_dir}/packet/convergence.json"; do
  if [ ! -f "${required}" ]; then
    echo "expected transition packet artifact file: ${required}" >&2
    exit 1
  fi
done

runtime_mismatch_json="${tmp_dir}/runtime-mismatch.json"
cat > "${runtime_mismatch_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSlice": {
    "id": "M35-S4-runtime-net-destub",
    "domain": "net"
  },
  "executionStatus": "PASS",
  "closureGate": "M35-D"
}
JSON

if ${packet_script} \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_mismatch_json}" \
  --output-dir "${tmp_dir}/packet-mismatch" \
  --format json > "${tmp_dir}/mismatch.log" 2>&1; then
  echo "expected M35 transition packet script to fail for runtime/plan selected-slice mismatch" >&2
  exit 1
fi
if ! rg -Fq 'runtime execution selectedSlice.id does not match plan slice id' "${tmp_dir}/mismatch.log"; then
  echo "expected selected-slice mismatch diagnostic from M35 transition packet script" >&2
  exit 1
fi

if ${packet_script} \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${tmp_dir}/missing-runtime.json" \
  --output-dir "${tmp_dir}/packet-missing-runtime" \
  --format json > "${tmp_dir}/missing-runtime.log" 2>&1; then
  echo "expected M35 transition packet script to fail when runtime execution json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing runtime execution json:' "${tmp_dir}/missing-runtime.log"; then
  echo "expected missing-runtime diagnostic from M35 transition packet script" >&2
  exit 1
fi

echo "m35 transition handoff packet test passed"
