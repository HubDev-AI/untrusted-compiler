#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brief_script="${root_dir}/scripts/generate-m18-kickoff-brief.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

pass_report="${tmp_dir}/pass-report.json"
cat > "${pass_report}" <<'JSON'
{
  "version": "0.1",
  "overall": "PASS",
  "failedStep": "none",
  "steps": [
    {"id":"playbook"},
    {"id":"quickstart"},
    {"id":"ci-smoke"}
  ],
  "friction": []
}
JSON

pass_json="$(${brief_script} --report "${pass_report}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and .failedStep == "none"
  and .stepCount == 3
  and .frictionCount == 0
  and (.recommendations | length) == 2
' >/dev/null; then
  echo "expected deterministic PASS kickoff brief json contract" >&2
  exit 1
fi

pass_markdown="${tmp_dir}/kickoff-pass.md"
${brief_script} --report "${pass_report}" --output "${pass_markdown}" >/dev/null
if [ ! -f "${pass_markdown}" ]; then
  echo "expected kickoff brief markdown output file" >&2
  exit 1
fi
if ! rg -Fq '## Recommendations' "${pass_markdown}"; then
  echo "expected kickoff brief markdown to include recommendations heading" >&2
  exit 1
fi
if ! rg -Fq '`PASS`' "${pass_markdown}"; then
  echo "expected kickoff brief markdown to include PASS status" >&2
  exit 1
fi

fail_report="${tmp_dir}/fail-report.json"
cat > "${fail_report}" <<'JSON'
{
  "version": "0.1",
  "overall": "FAIL",
  "failedStep": "ci-smoke",
  "steps": [{"id":"playbook"}],
  "friction": [{"stepId":"ci-smoke","message":"forced ci smoke failure"}]
}
JSON

fail_json="$(${brief_script} --report "${fail_report}" --format json)"
if ! printf '%s\n' "${fail_json}" | jq -e '
  .overall == "FAIL"
  and .failedStep == "ci-smoke"
  and .frictionCount == 1
  and (.recommendations[] | contains("ci-smoke"))
' >/dev/null; then
  echo "expected FAIL kickoff brief json contract with failed-step recommendation" >&2
  exit 1
fi

if ${brief_script} --report "${tmp_dir}/missing-report.json" --format json >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected kickoff brief script to fail on missing report" >&2
  exit 1
fi
if ! rg -Fq 'missing rehearsal report:' "${tmp_dir}/missing.log"; then
  echo "expected missing-report diagnostic from kickoff brief script" >&2
  exit 1
fi

echo "m18 kickoff brief generator test passed"
