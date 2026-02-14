#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m23-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m22_closure_pass="${tmp_dir}/m22-closure-pass.json"
cat > "${m22_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m22Gates": [
    {"gate":"M22-A","status":"PASS"},
    {"gate":"M22-B","status":"PASS"},
    {"gate":"M22-C","status":"PASS"},
    {"gate":"M22-D","status":"PASS"},
    {"gate":"M22-E","status":"PASS"},
    {"gate":"M22-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m22_packet_runtime="${tmp_dir}/m22-packet-runtime.json"
cat > "${m22_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M22-S4-runtime-hardening",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m22-closure-json "${m22_closure_pass}" --m22-packet-json "${m22_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m22Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M22-S4-runtime-hardening"))
' >/dev/null; then
  echo "expected deterministic PASS M23 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m23-kickoff-brief.md"
${brief_script} --m22-closure-json "${m22_closure_pass}" --m22-packet-json "${m22_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M23 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M23 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M23 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m22_closure_pending="${tmp_dir}/m22-closure-pending.json"
cat > "${m22_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m22Gates": [
    {"gate":"M22-A","status":"PASS"},
    {"gate":"M22-B","status":"PASS"},
    {"gate":"M22-C","status":"PASS"},
    {"gate":"M22-D","status":"PASS"},
    {"gate":"M22-E","status":"PENDING"},
    {"gate":"M22-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m22_packet_editor="${tmp_dir}/m22-packet-editor.json"
cat > "${m22_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "selectorTrack": "editor",
    "selectorRecommendationId": "M22-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m22-closure-json "${m22_closure_pending}" --m22-packet-json "${m22_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m22Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M22-E")) != null
  and (.recommendations[] | contains("M22-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M23 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m22-closure-json "${generated_closure_path}" --m22-packet-json "${m22_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M23 kickoff brief script to auto-generate missing M22 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M22 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${brief_script} --m22-closure-json "${m22_closure_pass}" --m22-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M23 kickoff brief script to fail for invalid M22 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M22 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M22 packet diagnostic from M23 kickoff brief script" >&2
  exit 1
fi

echo "m23 kickoff brief test passed"
