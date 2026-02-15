#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m31-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m30_closure_pass="${tmp_dir}/m30-closure-pass.json"
cat > "${m30_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m30Gates": [
    {"gate":"M30-A","status":"PASS"},
    {"gate":"M30-B","status":"PASS"},
    {"gate":"M30-C","status":"PASS"},
    {"gate":"M30-D","status":"PASS"},
    {"gate":"M30-E","status":"PASS"},
    {"gate":"M30-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m30_packet_runtime="${tmp_dir}/m30-packet-runtime.json"
cat > "${m30_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M30-S4-runtime-destub-runner",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m30-closure-json "${m30_closure_pass}" --m30-packet-json "${m30_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m30Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .selectorRecommendationId == "M30-S4-runtime-destub-runner"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M30-S4-runtime-destub-runner"))
' >/dev/null; then
  echo "expected deterministic PASS M31 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m31-kickoff-brief.md"
${brief_script} --m30-closure-json "${m30_closure_pass}" --m30-packet-json "${m30_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M31 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M31 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M31 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m30_closure_pending="${tmp_dir}/m30-closure-pending.json"
cat > "${m30_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m30Gates": [
    {"gate":"M30-A","status":"PASS"},
    {"gate":"M30-B","status":"PASS"},
    {"gate":"M30-C","status":"PASS"},
    {"gate":"M30-D","status":"PASS"},
    {"gate":"M30-E","status":"PENDING"},
    {"gate":"M30-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m30_packet_editor="${tmp_dir}/m30-packet-editor.json"
cat > "${m30_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "planSelectedTrack": "editor",
    "planSelectedSliceId": "M30-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m30-closure-json "${m30_closure_pending}" --m30-packet-json "${m30_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m30Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M30-E")) != null
  and (.recommendations[] | contains("M30-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M31 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m30-closure-json "${generated_closure_path}" --m30-packet-json "${m30_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M31 kickoff brief script to auto-generate missing M30 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M30 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"planSelectedTrack":"runtime"}}
JSON

if ${brief_script} --m30-closure-json "${m30_closure_pass}" --m30-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M31 kickoff brief script to fail for invalid M30 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M30 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M30 packet diagnostic from M31 kickoff brief script" >&2
  exit 1
fi

echo "m31 kickoff brief test passed"
