#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m20-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

m19_closure_pass="${tmp_dir}/m19-closure-pass.json"
cat > "${m19_closure_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "m19Gates": [
    {"gate":"M19-A","status":"PASS"},
    {"gate":"M19-B","status":"PASS"},
    {"gate":"M19-C","status":"PASS"},
    {"gate":"M19-D","status":"PASS"},
    {"gate":"M19-E","status":"PASS"},
    {"gate":"M19-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PASS"
  }
}
JSON

m19_packet_runtime="${tmp_dir}/m19-packet-runtime.json"
cat > "${m19_packet_runtime}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "runtime",
    "selectorTrack": "runtime",
    "selectorRecommendationId": "M19-S4-runtime-hardening",
    "runtimeStatus": "DRY_RUN",
    "convergenceOverall": "PASS"
  }
}
JSON

pass_json="$(${brief_script} --m19-closure-json "${m19_closure_pass}" --m19-packet-json "${m19_packet_runtime}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .m19Overall == "PASS"
  and .convergenceOverall == "PASS"
  and .selectorTrack == "runtime"
  and .primaryFocus == "runtime"
  and (.pendingGates | length) == 0
  and (.recommendations | length) == 2
  and (.recommendations[] | contains("M19-S4-runtime-hardening"))
' >/dev/null; then
  echo "expected deterministic PASS M20 kickoff brief json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/m20-kickoff-brief.md"
${brief_script} --m19-closure-json "${m19_closure_pass}" --m19-packet-json "${m19_packet_runtime}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M20 kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${markdown_output}"; then
  echo "expected M20 kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`runtime`' "${markdown_output}"; then
  echo "expected M20 kickoff brief markdown to include runtime primary focus" >&2
  exit 1
fi

m19_closure_pending="${tmp_dir}/m19-closure-pending.json"
cat > "${m19_closure_pending}" <<'JSON'
{
  "version": "0.1",
  "overall": "PENDING",
  "m19Gates": [
    {"gate":"M19-A","status":"PASS"},
    {"gate":"M19-B","status":"PASS"},
    {"gate":"M19-C","status":"PASS"},
    {"gate":"M19-D","status":"PASS"},
    {"gate":"M19-E","status":"PENDING"},
    {"gate":"M19-F","status":"PASS"}
  ],
  "packetSummary": {
    "convergenceOverall": "PENDING"
  }
}
JSON

m19_packet_editor="${tmp_dir}/m19-packet-editor.json"
cat > "${m19_packet_editor}" <<'JSON'
{
  "version": "0.1",
  "summary": {
    "kickoffPrimaryFocus": "stabilization",
    "selectorTrack": "editor",
    "selectorRecommendationId": "M19-S4-editor-expansion",
    "runtimeStatus": "FAIL",
    "convergenceOverall": "PENDING"
  }
}
JSON

pending_json="$(${brief_script} --m19-closure-json "${m19_closure_pending}" --m19-packet-json "${m19_packet_editor}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .m19Overall == "PENDING"
  and .convergenceOverall == "PENDING"
  and .primaryFocus == "stabilization"
  and (.pendingGates | index("M19-E")) != null
  and (.recommendations[] | contains("M19-E"))
' >/dev/null; then
  echo "expected deterministic stabilization fallback in M20 kickoff brief" >&2
  exit 1
fi

generated_closure_path="${tmp_dir}/generated-closure.json"
generated_json="$(${brief_script} --m19-closure-json "${generated_closure_path}" --m19-packet-json "${m19_packet_runtime}" --format json)"
if [ ! -f "${generated_closure_path}" ]; then
  echo "expected M20 kickoff brief script to auto-generate missing M19 closure report json" >&2
  exit 1
fi
if ! printf '%s\n' "${generated_json}" | jq -e '.version == "0.1"' >/dev/null; then
  echo "expected deterministic json output after auto-generating M19 closure report" >&2
  exit 1
fi

invalid_packet="${tmp_dir}/invalid-packet.json"
cat > "${invalid_packet}" <<'JSON'
{"version":"0.1","summary":{"selectorTrack":"runtime"}}
JSON

if ${brief_script} --m19-closure-json "${m19_closure_pass}" --m19-packet-json "${invalid_packet}" --format json >"${tmp_dir}/invalid-packet.log" 2>&1; then
  echo "expected M20 kickoff brief script to fail for invalid M19 packet contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M19 transition packet json contract:' "${tmp_dir}/invalid-packet.log"; then
  echo "expected invalid M19 packet diagnostic from M20 kickoff brief script" >&2
  exit 1
fi

echo "m20 kickoff brief test passed"
