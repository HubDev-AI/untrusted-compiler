#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_script="${root_dir}/scripts/build-m36-closure-report.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

closure_pass_json="${tmp_dir}/closure-pass.json"
cat > "${closure_pass_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "gates": [
    {"gate":"M36-A","status":"PASS"},
    {"gate":"M36-B","status":"PASS"},
    {"gate":"M36-C","status":"PASS"},
    {"gate":"M36-D","status":"PASS"},
    {"gate":"M36-E","status":"PASS"},
    {"gate":"M36-F","status":"PASS"}
  ]
}
JSON

kickoff_json="${tmp_dir}/kickoff.json"
cat > "${kickoff_json}" <<'JSON'
{
  "version": "0.1",
  "m35Overall": "PASS",
  "convergenceOverall": "PASS",
  "primaryFocus": "runtime",
  "pendingGates": []
}
JSON

matrix_json="${tmp_dir}/matrix.json"
cat > "${matrix_json}" <<'JSON'
{
  "version": "0.1",
  "primaryFocus": "runtime",
  "m35Overall": "PASS",
  "convergenceOverall": "PASS",
  "pendingGateCount": 0,
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
  "closureGate": "M36-C",
  "plan": [
    {"order":1,"id":"M36-S4-runtime-validator-destub","title":"Validator de-stub","domain":"validators"}
  ]
}
JSON

runtime_execution_json="${tmp_dir}/runtime-execution.json"
cat > "${runtime_execution_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSlice": {
    "id": "M36-S4-runtime-validator-destub",
    "domain": "validators"
  },
  "executionStatus": "DRY_RUN",
  "closureGate": "M36-D"
}
JSON

convergence_json="${tmp_dir}/convergence.json"
cat > "${convergence_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSliceId": "M36-S4-runtime-validator-destub",
  "planClosureGate": "M36-C",
  "runtimeClosureGate": "M36-D",
  "runtimeStatus": "DRY_RUN",
  "executionPass": true,
  "overall": "PASS",
  "nextAction": "Proceed to M36 transition handoff packet preparation.",
  "convergenceClosureGate": "M36-E"
}
JSON

packet_json="${tmp_dir}/packet/handoff-packet.json"
report_json="$(${report_script} \
  --closure-json "${closure_pass_json}" \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --convergence-json "${convergence_json}" \
  --packet-json "${packet_json}" \
  --format json)"

if ! printf '%s\n' "${report_json}" | jq -e '
  .version == "0.1"
  and .closureGate == "M36-G"
  and .overall == "PASS"
  and (.m36Gates | length) == 6
  and .packetSummary.kickoffPrimaryFocus == "runtime"
  and .packetSummary.planSelectedTrack == "runtime"
  and .packetSummary.planSelectedSliceId == "M36-S4-runtime-validator-destub"
  and .packetSummary.runtimeStatus == "DRY_RUN"
  and .packetSummary.convergenceOverall == "PASS"
  and .nextAction == "M36 is closed; start M37 kickoff."
' >/dev/null; then
  echo "expected deterministic PASS M36 closure report json contract" >&2
  exit 1
fi

if [ ! -f "${packet_json}" ]; then
  echo "expected M36 transition handoff packet to be generated when packet json is missing" >&2
  exit 1
fi
if ! jq -e '.closureGate == "M36-F"' "${packet_json}" >/dev/null; then
  echo "expected generated M36 transition handoff packet closure gate marker" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m36-closure-report.md"
${report_script} \
  --closure-json "${closure_pass_json}" \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --convergence-json "${convergence_json}" \
  --packet-json "${packet_json}" \
  --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M36 closure report markdown output file" >&2
  exit 1
fi
if ! rg -Fq '| M36-F | PASS |' "${markdown_output}"; then
  echo "expected M36 closure report markdown to include M36-F gate row" >&2
  exit 1
fi
if ! rg -Fq -- '- closureGate: `M36-G`' "${markdown_output}"; then
  echo "expected M36 closure report markdown to include M36-G closure marker" >&2
  exit 1
fi

if ${report_script} \
  --closure-json "${closure_pass_json}" \
  --kickoff-json "${tmp_dir}/missing-kickoff.json" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --convergence-json "${convergence_json}" \
  --packet-json "${packet_json}" \
  --format json >"${tmp_dir}/missing-kickoff.log" 2>&1; then
  echo "expected M36 closure report script to fail when kickoff json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M36 kickoff brief json:' "${tmp_dir}/missing-kickoff.log"; then
  echo "expected missing-kickoff diagnostic from M36 closure report script" >&2
  exit 1
fi

mismatch_packet_json="${tmp_dir}/mismatch-packet.json"
cat > "${mismatch_packet_json}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M36-S4-runtime-net-destub",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  },
  "artifacts": {
    "kickoff": "kickoff.json",
    "priorityMatrix": "priority-matrix.json",
    "runtimePlan": "runtime-plan.json",
    "runtimeExecution": "runtime-execution.json",
    "convergence": "convergence.json"
  },
  "closureGate": "M36-F"
}
JSON

if ${report_script} \
  --closure-json "${closure_pass_json}" \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --convergence-json "${convergence_json}" \
  --packet-json "${mismatch_packet_json}" \
  --format json >"${tmp_dir}/mismatch-packet.log" 2>&1; then
  echo "expected M36 closure report script to fail for packet/runtime plan mismatch" >&2
  exit 1
fi
if ! rg -Fq 'transition packet planSelectedSliceId does not match runtime plan slice id' "${tmp_dir}/mismatch-packet.log"; then
  echo "expected mismatch diagnostic from M36 closure report script" >&2
  exit 1
fi

invalid_packet_json="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet_json}" <<'JSON'
{"version":"0.1","artifacts":{"kickoff":"kickoff.json"}}
JSON

if ${report_script} \
  --closure-json "${closure_pass_json}" \
  --kickoff-json "${kickoff_json}" \
  --matrix-json "${matrix_json}" \
  --plan-json "${plan_json}" \
  --runtime-execution-json "${runtime_execution_json}" \
  --convergence-json "${convergence_json}" \
  --packet-json "${invalid_packet_json}" \
  --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M36 closure report script to fail for invalid packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M36 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid packet contract diagnostic from M36 closure report script" >&2
  exit 1
fi

echo "m36 closure report test passed"
