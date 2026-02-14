#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
selector_script="${root_dir}/scripts/select-m18-next-slice.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

kickoff_pass="${tmp_dir}/kickoff-pass.json"
cat > "${kickoff_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "failedStep": "none",
  "frictionCount": 0,
  "recommendations": []
}
JSON

matrix_editor_top="${tmp_dir}/matrix-editor.json"
cat > "${matrix_editor_top}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "frictionCount": 0,
  "tracks": [
    {"priority":1,"track":"editor","score":65},
    {"priority":2,"track":"release","score":62},
    {"priority":3,"track":"runtime","score":60}
  ]
}
JSON

selector_json="$(${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${matrix_editor_top}" --output "${tmp_dir}/selector-pass.json" --format json)"

if ! printf '%s\n' "${selector_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and .selectedTrack == "editor"
  and .recommendation.id == "M18-S4-editor-contract-expansion"
  and .recommendation.closureGate == "M18-C"
' >/dev/null; then
  echo "expected selector to recommend editor expansion slice for PASS editor-top matrix" >&2
  exit 1
fi

matrix_release_pass="${tmp_dir}/matrix-release-pass.json"
cat > "${matrix_release_pass}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "frictionCount": 0,
  "tracks": [
    {"priority":1,"track":"release","score":70},
    {"priority":2,"track":"editor","score":68}
  ]
}
JSON

release_selector_json="$(${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${matrix_release_pass}" --output "${tmp_dir}/selector-release-pass.json" --format json)"
if ! printf '%s\n' "${release_selector_json}" | jq -e '
  .overall == "PASS"
  and .selectedTrack == "release"
  and .recommendation.id == "M18-S5-release-publish-integrity-contract-expansion"
' >/dev/null; then
  echo "expected selector to recommend release publish integrity slice when release is top priority" >&2
  exit 1
fi

kickoff_fail="${tmp_dir}/kickoff-fail.json"
cat > "${kickoff_fail}" <<'JSON'
{
  "version": "0.1",
  "overall": "FAIL",
  "failedStep": "quickstart",
  "frictionCount": 2,
  "recommendations": []
}
JSON

matrix_release_top="${tmp_dir}/matrix-release.json"
cat > "${matrix_release_top}" <<'JSON'
{
  "version": "0.1",
  "overall": "FAIL",
  "frictionCount": 2,
  "tracks": [
    {"priority":1,"track":"release","score":80},
    {"priority":2,"track":"runtime","score":79}
  ]
}
JSON

fail_json="$(${selector_script} --kickoff-json "${kickoff_fail}" --matrix-json "${matrix_release_top}" --output "${tmp_dir}/selector-fail.json" --format json)"
if ! printf '%s\n' "${fail_json}" | jq -e '
  .overall == "FAIL"
  and .selectedTrack == "runtime"
  and .recommendation.id == "M18-S6-runtime-remediation-first"
' >/dev/null; then
  echo "expected selector to force runtime remediation when kickoff is failing" >&2
  exit 1
fi

if ${selector_script} --kickoff-json "${tmp_dir}/missing.json" --matrix-json "${matrix_editor_top}" --format json >"${tmp_dir}/missing-kickoff.log" 2>&1; then
  echo "expected selector to fail when kickoff json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing kickoff summary json:' "${tmp_dir}/missing-kickoff.log"; then
  echo "expected missing-kickoff diagnostic from selector" >&2
  exit 1
fi

invalid_matrix="${tmp_dir}/invalid-matrix.json"
cat > "${invalid_matrix}" <<'JSON'
{"version":"0.1","tracks":[]}
JSON

if ${selector_script} --kickoff-json "${kickoff_pass}" --matrix-json "${invalid_matrix}" --format json >"${tmp_dir}/invalid-matrix.log" 2>&1; then
  echo "expected selector to fail for invalid matrix contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid priority matrix contract:' "${tmp_dir}/invalid-matrix.log"; then
  echo "expected invalid-matrix diagnostic from selector" >&2
  exit 1
fi

echo "m18 next-slice selector test passed"
