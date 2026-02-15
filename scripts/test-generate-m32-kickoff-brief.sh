#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m32-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m31_closure_pass="${tmp_dir}/m31-closure-pass.json"
cat > "${m31_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m31Gates": [
    {"gate":"M31-A","status":"PASS"},
    {"gate":"M31-B","status":"PASS"},
    {"gate":"M31-C","status":"PASS"},
    {"gate":"M31-D","status":"PASS"},
    {"gate":"M31-E","status":"PASS"},
    {"gate":"M31-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m31_packet_runtime="${tmp_dir}/m31-packet-runtime.json"
cat > "${m31_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "planSelectedTrack": "runtime",
    "planSelectedSliceId": "M31-S4-runtime-destub-runner",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m31-closure-json "${m31_closure_pass}" --m31-packet-json "${m31_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m31Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .selectorRecommendationId == "M31-S4-runtime-destub-runner"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M31-S4-runtime-destub-runner"))
' >/dev/null; then
  echo "expected deterministic PASS M32 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m32-kickoff-brief.md"
${brief_script} --m31-closure-json "${m31_closure_pass}" --m31-packet-json "${m31_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M32 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M32 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M32 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m31_closure_pending="${tmp_dir}/m31-closure-pending.json"
cat > "${m31_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m31Gates": [
    {"gate":"M31-A","status":"PASS"},
    {"gate":"M31-B","status":"PASS"},
    {"gate":"M31-C","status":"PASS"},
    {"gate":"M31-D","status":"PASS"},
    {"gate":"M31-E","status":"PENDING"},
    {"gate":"M31-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m31_packet_editor="${tmp_dir}/m31-packet-editor.json"
cat > "${m31_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "planSelectedTrack": "editor",
    "planSelectedSliceId": "M31-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m31-closure-json "${m31_closure_pending}" --m31-packet-json "${m31_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m31Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M31-E")) != null
  and (.recommendations[] | contains("M31-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M32 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m31-closure-json "${generated_closure_path}" --m31-packet-json "${m31_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M32 kickoff brief script to auto-generate missing M31 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M31 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"planSelectedTrack":"runtime"}}
JSON

if ${brief_script} --m31-closure-json "${m31_closure_pass}" --m31-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M32 kickoff brief script to fail for invalid M31 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M31 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M31 packet diagnostic from M32 kickoff brief script" >&2
  exit 1
fi

echo "m32 kickoff brief test passed"
