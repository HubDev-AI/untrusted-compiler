#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_script="${root_dir}/scripts/build-m18-priority-matrix.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

pass_report="${tmp_dir}/pass-report.json"
cat > "${pass_report}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "failedStep": "none",
  "steps": [{"id":"playbook"}],
  "friction": []
}
JSON

pass_json_output="${tmp_dir}/pass-matrix.json"
pass_md_output="${tmp_dir}/pass-matrix.md"
pass_json="$(${matrix_script} --report "${pass_report}" --output-json "${pass_json_output}" --output-markdown "${pass_md_output}" --format json)"

if [ ! -f "${pass_json_output}" ]; then
  echo "expected priority matrix script to write json output" >&2
  exit 1
fi
if [ ! -f "${pass_md_output}" ]; then
  echo "expected priority matrix script to write markdown output" >&2
  exit 1
fi

if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and .frictionCount == 0
  and (.tracks | length) == 3
  and .tracks[0].track == "editor"
  and .tracks[1].track == "release"
  and .tracks[2].track == "runtime"
' >/dev/null; then
  echo "expected deterministic PASS priority matrix ordering" >&2
  exit 1
fi

if ! rg -Fq '| 1 | editor |' "${pass_md_output}"; then
  echo "expected markdown matrix to include editor as first priority for PASS report" >&2
  exit 1
fi

fail_report="${tmp_dir}/fail-report.json"
cat > "${fail_report}" <<'JSON'
{
  "version": "0.1",
  "overall": "FAIL",
  "failedStep": "quickstart",
  "steps": [{"id":"playbook"}],
  "friction": [{"stepId":"quickstart","message":"forced failure"}]
}
JSON

fail_json="$(${matrix_script} --report "${fail_report}" --output-json "${tmp_dir}/fail-matrix.json" --output-markdown "${tmp_dir}/fail-matrix.md" --format json)"
if ! printf '%s\n' "${fail_json}" | jq -e '
  .overall == "FAIL"
  and .frictionCount == 1
  and .tracks[0].track == "runtime"
  and .tracks[1].track == "release"
  and .tracks[2].track == "editor"
  and (.tracks[0].score > .tracks[1].score)
  and (.tracks[1].score > .tracks[2].score)
' >/dev/null; then
  echo "expected deterministic FAIL priority matrix ordering" >&2
  exit 1
fi

if ${matrix_script} --report "${tmp_dir}/missing.json" --format json >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected priority matrix script to fail when report is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing rehearsal report:' "${tmp_dir}/missing.log"; then
  echo "expected missing-report diagnostic from priority matrix script" >&2
  exit 1
fi

echo "m18 priority matrix builder test passed"
