#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m29-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m28_closure_pass="${tmp_dir}/m28-closure-pass.json"
cat > "${m28_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m28Gates": [
    {"gate":"M28-A","status":"PASS"},
    {"gate":"M28-B","status":"PASS"},
    {"gate":"M28-C","status":"PASS"},
    {"gate":"M28-D","status":"PASS"},
    {"gate":"M28-E","status":"PASS"},
    {"gate":"M28-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m28_packet_runtime="${tmp_dir}/m28-packet-runtime.json"
cat > "${m28_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M28-S4-runtime-hardening",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m28-closure-json "${m28_closure_pass}" --m28-packet-json "${m28_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m28Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M28-S4-runtime-hardening"))
' >/dev/null; then
  echo "expected deterministic PASS M29 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m29-kickoff-brief.md"
${brief_script} --m28-closure-json "${m28_closure_pass}" --m28-packet-json "${m28_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M29 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M29 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M29 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m28_closure_pending="${tmp_dir}/m28-closure-pending.json"
cat > "${m28_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m28Gates": [
    {"gate":"M28-A","status":"PASS"},
    {"gate":"M28-B","status":"PASS"},
    {"gate":"M28-C","status":"PASS"},
    {"gate":"M28-D","status":"PASS"},
    {"gate":"M28-E","status":"PENDING"},
    {"gate":"M28-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m28_packet_editor="${tmp_dir}/m28-packet-editor.json"
cat > "${m28_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "selectorTrack": "editor",
    "selectorRecommendationId": "M28-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m28-closure-json "${m28_closure_pending}" --m28-packet-json "${m28_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m28Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M28-E")) != null
  and (.recommendations[] | contains("M28-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M29 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m28-closure-json "${generated_closure_path}" --m28-packet-json "${m28_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M29 kickoff brief script to auto-generate missing M28 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M28 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${brief_script} --m28-closure-json "${m28_closure_pass}" --m28-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M29 kickoff brief script to fail for invalid M28 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M28 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M28 packet diagnostic from M29 kickoff brief script" >&2
  exit 1
fi

echo "m29 kickoff brief test passed"
