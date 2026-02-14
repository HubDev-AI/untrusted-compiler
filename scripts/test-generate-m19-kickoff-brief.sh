#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m19-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m18_closure_pass="${tmp_dir}/m18-closure-pass.json"
cat > "${m18_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m18Gates": [
    {"gate":"M18-A","status":"PASS"},
    {"gate":"M18-B","status":"PASS"},
    {"gate":"M18-C","status":"PASS"},
    {"gate":"M18-D","status":"PASS"},
    {"gate":"M18-E","status":"PASS"},
    {"gate":"M18-F","status":"PASS"},
    {"gate":"M18-G","status":"PASS"},
    {"gate":"M18-H","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m18_packet_runtime="${tmp_dir}/m18-packet-runtime.json"
cat > "${m18_packet_runtime}" <<'JSON'
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

pass_json="$(${brief_script} --m18-closure-json "${m18_closure_pass}" --m18-packet-json "${m18_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m18Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M18-S6-runtime-confidence-hardening"))
' >/dev/null; then
  echo "expected deterministic PASS M19 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m19-kickoff-brief.md"
${brief_script} --m18-closure-json "${m18_closure_pass}" --m18-packet-json "${m18_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M19 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M19 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M19 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m18_closure_pending="${tmp_dir}/m18-closure-pending.json"
cat > "${m18_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m18Gates": [
    {"gate":"M18-A","status":"PASS"},
    {"gate":"M18-B","status":"PASS"},
    {"gate":"M18-C","status":"PASS"},
    {"gate":"M18-D","status":"PASS"},
    {"gate":"M18-E","status":"PENDING"},
    {"gate":"M18-F","status":"PASS"},
    {"gate":"M18-G","status":"PASS"},
    {"gate":"M18-H","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m18_packet_editor="${tmp_dir}/m18-packet-editor.json"
cat > "${m18_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffOverall": "PASS",
    "selectorTrack": "editor",
    "selectorRecommendationId": "M18-S4-editor-contract-expansion",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m18-closure-json "${m18_closure_pending}" --m18-packet-json "${m18_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m18Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M18-E")) != null
  and (.recommendations[] | contains("M18-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M19 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m18-closure-json "${generated_closure_path}" --m18-packet-json "${m18_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M19 kickoff brief script to auto-generate missing M18 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M18 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${brief_script} --m18-closure-json "${m18_closure_pass}" --m18-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M19 kickoff brief script to fail for invalid M18 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M18 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M18 packet diagnostic from M19 kickoff brief script" >&2
  exit 1
fi

echo "m19 kickoff brief test passed"
