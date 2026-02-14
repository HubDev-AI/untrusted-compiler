#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m21-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m20_closure_pass="${tmp_dir}/m20-closure-pass.json"
cat > "${m20_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m20Gates": [
    {"gate":"M20-A","status":"PASS"},
    {"gate":"M20-B","status":"PASS"},
    {"gate":"M20-C","status":"PASS"},
    {"gate":"M20-D","status":"PASS"},
    {"gate":"M20-E","status":"PASS"},
    {"gate":"M20-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m20_packet_runtime="${tmp_dir}/m20-packet-runtime.json"
cat > "${m20_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M20-S4-runtime-hardening",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m20-closure-json "${m20_closure_pass}" --m20-packet-json "${m20_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m20Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M20-S4-runtime-hardening"))
' >/dev/null; then
  echo "expected deterministic PASS M21 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m21-kickoff-brief.md"
${brief_script} --m20-closure-json "${m20_closure_pass}" --m20-packet-json "${m20_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M21 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M21 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M21 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m20_closure_pending="${tmp_dir}/m20-closure-pending.json"
cat > "${m20_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m20Gates": [
    {"gate":"M20-A","status":"PASS"},
    {"gate":"M20-B","status":"PASS"},
    {"gate":"M20-C","status":"PASS"},
    {"gate":"M20-D","status":"PASS"},
    {"gate":"M20-E","status":"PENDING"},
    {"gate":"M20-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m20_packet_editor="${tmp_dir}/m20-packet-editor.json"
cat > "${m20_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "selectorTrack": "editor",
    "selectorRecommendationId": "M20-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m20-closure-json "${m20_closure_pending}" --m20-packet-json "${m20_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m20Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M20-E")) != null
  and (.recommendations[] | contains("M20-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M21 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m20-closure-json "${generated_closure_path}" --m20-packet-json "${m20_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M21 kickoff brief script to auto-generate missing M20 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M20 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${brief_script} --m20-closure-json "${m20_closure_pass}" --m20-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M21 kickoff brief script to fail for invalid M20 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M20 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M20 packet diagnostic from M21 kickoff brief script" >&2
  exit 1
fi

echo "m21 kickoff brief test passed"
