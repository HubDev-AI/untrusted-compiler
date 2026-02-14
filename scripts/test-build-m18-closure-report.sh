#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_script="${root_dir}/scripts/build-m18-closure-report.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

closure_pass="${tmp_dir}/closure-pass.json"
cat > "${closure_pass}" <<'JSON'
{
  "repo": ".",
  "overall": "PASS",
  "pendingCount": 0,
  "gates": [
    {"gate":"M18-A","status":"PASS"},
    {"gate":"M18-B","status":"PASS"},
    {"gate":"M18-C","status":"PASS"},
    {"gate":"M18-D","status":"PASS"},
    {"gate":"M18-E","status":"PASS"},
    {"gate":"M18-F","status":"PASS"},
    {"gate":"M18-G","status":"PASS"},
    {"gate":"M18-H","status":"PASS"}
  ]
}
JSON

packet_pass="${tmp_dir}/packet-pass.json"
cat > "${packet_pass}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffOverall": "PASS",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M18-S6-runtime-confidence-hardening",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${report_script} --closure-json "${closure_pass}" --packet-json "${packet_pass}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and (.m18Gates | length) == 8
  and (.m18Gates[] | select(.gate == "M18-H")).status == "PASS"
  and .packetSummary.convergenceOverall == "PASS"
  and .nextAction == "M18 is closed; start M19 kickoff."
' >/dev/null; then
  echo "expected deterministic PASS M18 closure report json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m18-closure-report.md"
${report_script} --closure-json "${closure_pass}" --packet-json "${packet_pass}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M18 closure report markdown output file" >&2
  exit 1
fi
if ! rg -Fq '| M18-H | PASS |' "${markdown_output}"; then
  echo "expected markdown closure report to include M18-H gate row" >&2
  exit 1
fi

closure_pending="${tmp_dir}/closure-pending.json"
cat > "${closure_pending}" <<'JSON'
{
  "repo": ".",
  "overall": "PENDING",
  "pendingCount": 1,
  "gates": [
    {"gate":"M18-A","status":"PASS"},
    {"gate":"M18-B","status":"PASS"},
    {"gate":"M18-C","status":"PASS"},
    {"gate":"M18-D","status":"PASS"},
    {"gate":"M18-E","status":"PENDING"},
    {"gate":"M18-F","status":"PASS"},
    {"gate":"M18-G","status":"PASS"},
    {"gate":"M18-H","status":"PASS"}
  ]
}
JSON

pending_json="$(${report_script} --closure-json "${closure_pending}" --packet-json "${packet_pass}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .overall == "PENDING"
  and (.m18Gates[] | select(.gate == "M18-E")).status == "PENDING"
  and .nextAction == "Resolve pending M18 gates or convergence before starting M19."
' >/dev/null; then
  echo "expected deterministic PENDING M18 closure report json contract" >&2
  exit 1
fi

closure_missing="${tmp_dir}/closure-missing.json"
cat > "${closure_missing}" <<'JSON'
{
  "repo": ".",
  "overall": "PASS",
  "pendingCount": 0,
  "gates": [
    {"gate":"M18-A","status":"PASS"},
    {"gate":"M18-B","status":"PASS"}
  ]
}
JSON

if ${report_script} --closure-json "${closure_missing}" --packet-json "${packet_pass}" --format json >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected M18 closure report builder to fail when required M18 gate is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing required M18 gate in closure json: M18-C' "${tmp_dir}/missing.log"; then
  echo "expected missing-gate diagnostic from M18 closure report builder" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/packet-invalid.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"kickoffOverall":"PASS"}}
JSON

if ${report_script} --closure-json "${closure_pass}" --packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M18 closure report builder to fail for invalid transition packet" >&2
  exit 1
fi
if ! rg -Fq 'invalid transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid-packet diagnostic from M18 closure report builder" >&2
  exit 1
fi

echo "m18 closure report test passed"
