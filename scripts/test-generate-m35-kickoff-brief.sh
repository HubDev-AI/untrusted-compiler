#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m35-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m34_closure_pass="${tmp_dir}/m34-closure-pass.json"
cat > "${m34_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m34Gates": [
    {"gate":"M34-A","status":"PASS"},
    {"gate":"M34-B","status":"PASS"},
    {"gate":"M34-C","status":"PASS"},
    {"gate":"M34-D","status":"PASS"},
    {"gate":"M34-E","status":"PASS"},
    {"gate":"M34-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m34_packet_runtime="${tmp_dir}/m34-packet-runtime.json"
cat > "${m34_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M34-S4-runtime-destub-runner",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m34-closure-json "${m34_closure_pass}" --m34-packet-json "${m34_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m34Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .selectorRecommendationId == "M34-S4-runtime-destub-runner"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M34-S4-runtime-destub-runner"))
' >/dev/null; then
  echo "expected deterministic PASS M35 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m35-kickoff-brief.md"
${brief_script} --m34-closure-json "${m34_closure_pass}" --m34-packet-json "${m34_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M35 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M35 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M35 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m34_closure_pending="${tmp_dir}/m34-closure-pending.json"
cat > "${m34_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m34Gates": [
    {"gate":"M34-A","status":"PASS"},
    {"gate":"M34-B","status":"PASS"},
    {"gate":"M34-C","status":"PASS"},
    {"gate":"M34-D","status":"PASS"},
    {"gate":"M34-E","status":"PENDING"},
    {"gate":"M34-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m34_packet_editor="${tmp_dir}/m34-packet-editor.json"
cat > "${m34_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "planSelectedTrack": "editor",
    "planSelectedSliceId": "M34-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m34-closure-json "${m34_closure_pending}" --m34-packet-json "${m34_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m34Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M34-E")) != null
  and (.recommendations[] | contains("M34-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M35 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m34-closure-json "${generated_closure_path}" --m34-packet-json "${m34_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M35 kickoff brief script to auto-generate missing M34 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M34 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"planSelectedTrack":"runtime"}}
JSON

if ${brief_script} --m34-closure-json "${m34_closure_pass}" --m34-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M35 kickoff brief script to fail for invalid M34 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M34 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M34 packet diagnostic from M35 kickoff brief script" >&2
  exit 1
fi

echo "m35 kickoff brief test passed"
