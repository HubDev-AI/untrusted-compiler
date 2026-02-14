#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_script="${root_dir}/scripts/build-m24-closure-report.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

closure_pass_json="${tmp_dir}/closure-pass.json"
cat > "${closure_pass_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "gates": [
    {"gate":"M24-A","status":"PASS"},
    {"gate":"M24-B","status":"PASS"},
    {"gate":"M24-C","status":"PASS"},
    {"gate":"M24-D","status":"PASS"},
    {"gate":"M24-E","status":"PASS"},
    {"gate":"M24-F","status":"PASS"}
  ]
}
JSON

packet_json="${tmp_dir}/packet.json"
cat > "${packet_json}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M24-S4-runtime-hardening",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

report_json="$(${report_script} --closure-json "${closure_pass_json}" --packet-json "${packet_json}" --format json)"
if ! printf '%s\n' "${report_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and (.m24Gates | length) == 6
  and .packetSummary.selectorTrack == "runtime"
  and .packetSummary.runtimeStatus == "DRY_RUN"
  and .nextAction == "M24 is closed; start M25 kickoff."
' >/dev/null; then
  echo "expected deterministic PASS M24 closure report json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m24-closure-report.md"
${report_script} --closure-json "${closure_pass_json}" --packet-json "${packet_json}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M24 closure report markdown output file" >&2
  exit 1
fi
if ! rg -Fq '| M24-F | PASS |' "${markdown_output}"; then
  echo "expected M24 closure report markdown to include M24-F gate row" >&2
  exit 1
fi

closure_pending_json="${tmp_dir}/closure-pending.json"
cat > "${closure_pending_json}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "gates": [
    {"gate":"M24-A","status":"PASS"},
    {"gate":"M24-B","status":"PASS"},
    {"gate":"M24-C","status":"PASS"},
    {"gate":"M24-D","status":"PASS"},
    {"gate":"M24-E","status":"PENDING"},
    {"gate":"M24-F","status":"PASS"}
  ]
}
JSON

packet_pending_json="${tmp_dir}/packet-pending.json"
cat > "${packet_pending_json}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M24-S4-runtime-hardening",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${report_script} --closure-json "${closure_pending_json}" --packet-json "${packet_pending_json}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .overall == "PENDING"
  and (.m24Gates[] | select(.gate == "M24-E")).status == "PENDING"
  and .packetSummary.convergenceOverall == "PENDING"
' >/dev/null; then
  echo "expected deterministic PENDING M24 closure report json contract" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${report_script} --closure-json "${closure_pass_json}" --packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M24 closure report script to fail for invalid packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid packet diagnostic from M24 closure report script" >&2
  exit 1
fi

echo "m24 closure report test passed"
