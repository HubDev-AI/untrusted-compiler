#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/build-m36-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m35_closure_pass="${tmp_dir}/m35-closure-pass.json"
cat > "${m35_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "closureGate": "M35-G",
  "overall": "PASS",
  "m35Gates": [
    {"gate":"M35-A","status":"PASS"},
    {"gate":"M35-B","status":"PASS"},
    {"gate":"M35-C","status":"PASS"},
    {"gate":"M35-D","status":"PASS"},
    {"gate":"M35-E","status":"PASS"},
    {"gate":"M35-F","status":"PASS"}
  ],
  "packetSummary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M35-S4-runtime-validator-destub",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

m35_packet_pass="${tmp_dir}/m35-packet-pass.json"
cat > "${m35_packet_pass}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M35-S4-runtime-validator-destub",
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
  "closureGate": "M35-F"
}
JSON

pass_output_json="${tmp_dir}/m36-kickoff-pass.json"
pass_json="$(${brief_script} \
  --m35-closure-json "${m35_closure_pass}" \
  --m35-packet-json "${m35_packet_pass}" \
  --output "${pass_output_json}" \
  --format json)"

if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .kickoffGate == "M36-A"
  and .m35Overall == "PASS"
  and .m35ClosureGate == "M35-G"
  and .m35PacketGate == "M35-F"
  and .selectedTrack == "runtime"
  and .selectedSliceId == "M35-S4-runtime-validator-destub"
  and .runtimeStatus == "DRY_RUN"
  and .convergenceOverall == "PASS"
  and .pendingGateCount == 0
  and (.pendingGates | length) == 0
  and .primaryFocus == "runtime"
  and (.m36KickoffIntent | contains("M36"))
  and (.m36KickoffIntent | contains("M35-S4-runtime-validator-destub"))
  and (.recommendations | length) == 3
' >/dev/null; then
  echo "expected deterministic PASS M36 kickoff brief json contract" >&2
  exit 1
fi

if [ ! -f "${pass_output_json}" ]; then
  echo "expected M36 kickoff brief builder to write output json file" >&2
  exit 1
fi
if ! jq -e '.kickoffGate == "M36-A"' "${pass_output_json}" >/dev/null; then
  echo "expected output json file to include M36 kickoff gate marker" >&2
  exit 1
fi

text_output="$(${brief_script} \
  --m35-closure-json "${m35_closure_pass}" \
  --m35-packet-json "${m35_packet_pass}" \
  --output "${tmp_dir}/m36-kickoff-pass-text.json" \
  --format text)"
if ! printf '%s\n' "${text_output}" | rg -Fq 'M36 Kickoff Brief'; then
  echo "expected text output heading from M36 kickoff brief builder" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'primaryFocus: runtime'; then
  echo "expected text output to include runtime primary focus" >&2
  exit 1
fi

m35_closure_pending="${tmp_dir}/m35-closure-pending.json"
cat > "${m35_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "closureGate": "M35-G",
  "overall": "PENDING",
  "m35Gates": [
    {"gate":"M35-A","status":"PASS"},
    {"gate":"M35-B","status":"PASS"},
    {"gate":"M35-C","status":"PASS"},
    {"gate":"M35-D","status":"PASS"},
    {"gate":"M35-E","status":"PENDING"},
    {"gate":"M35-F","status":"PASS"}
  ],
  "packetSummary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M35-S4-runtime-validator-destub",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

m35_packet_pending="${tmp_dir}/m35-packet-pending.json"
cat > "${m35_packet_pending}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M35-S4-runtime-validator-destub",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  },
  "artifacts": {
    "kickoff": "kickoff.json",
    "priorityMatrix": "priority-matrix.json",
    "runtimePlan": "runtime-plan.json",
    "runtimeExecution": "runtime-execution.json",
    "convergence": "convergence.json"
  },
  "closureGate": "M35-F"
}
JSON

pending_json="$(${brief_script} \
  --m35-closure-json "${m35_closure_pending}" \
  --m35-packet-json "${m35_packet_pending}" \
  --output "${tmp_dir}/m36-kickoff-pending.json" \
  --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m35Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .runtimeStatus == "FAIL"
  and .primaryFocus == "stabilization"
  and .pendingGateCount == 1
  and (.pendingGates | index("M35-E")) != null
  and (.m36KickoffIntent | contains("Resolve remaining M35 closure/convergence issues"))
  and (.recommendations[] | contains("M35-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M36 kickoff brief" >&2
  exit 1
fi

if ${brief_script} \
  --m35-closure-json "${tmp_dir}/missing-closure.json" \
  --m35-packet-json "${m35_packet_pass}" \
  --format json >"${tmp_dir}/missing-closure.log" 2>&1; then
  echo "expected M36 kickoff brief builder to fail when closure report is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M35 closure report json:' "${tmp_dir}/missing-closure.log"; then
  echo "expected missing-closure diagnostic from M36 kickoff brief builder" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"planSelectedTrack":"runtime"}}
JSON

if ${brief_script} \
  --m35-closure-json "${m35_closure_pass}" \
  --m35-packet-json "${invalid_packet}" \
  --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M36 kickoff brief builder to fail for invalid packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M35 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid packet diagnostic from M36 kickoff brief builder" >&2
  exit 1
fi

mismatch_packet="${tmp_dir}/mismatch-packet.json"
cat > "${mismatch_packet}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M35-S4-runtime-db-destub",
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
  "closureGate": "M35-F"
}
JSON

if ${brief_script} \
  --m35-closure-json "${m35_closure_pass}" \
  --m35-packet-json "${mismatch_packet}" \
  --format json >"${tmp_dir}/mismatch.log" 2>&1; then
  echo "expected M36 kickoff brief builder to fail for closure/packet selected-slice mismatch" >&2
  exit 1
fi
if ! rg -Fq 'M35 closure report packetSummary.planSelectedSliceId does not match transition packet summary.planSelectedSliceId' "${tmp_dir}/mismatch.log"; then
  echo "expected selected-slice mismatch diagnostic from M36 kickoff brief builder" >&2
  exit 1
fi

echo "m36 kickoff brief test passed"
