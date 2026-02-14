#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary_script="${root_dir}/scripts/build-m18-track-convergence-summary.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

closure_pass="${tmp_dir}/closure-pass.json"
cat > "${closure_pass}" <<'JSON'
{
  "repo": ".",
  "overall": "PASS",
  "message": "all tracked closure checks satisfied",
  "pendingCount": 0,
  "gates": [
    {"gate":"M18-D","status":"PASS","check":"editor","evidence":"x"},
    {"gate":"M18-E","status":"PASS","check":"release","evidence":"x"},
    {"gate":"M18-F","status":"PASS","check":"runtime","evidence":"x"}
  ]
}
JSON

pass_json="$(${summary_script} --closure-json "${closure_pass}" --format json)"
if ! printf '%s\n' "${pass_json}" | jq -e '
  .version == "0.1"
  and .overall == "PASS"
  and (.tracks | length) == 3
  and (.tracks[] | select(.gate == "M18-D")).status == "PASS"
  and (.tracks[] | select(.gate == "M18-E")).status == "PASS"
  and (.tracks[] | select(.gate == "M18-F")).status == "PASS"
  and .nextAction == "Advance to post-M18 transition planning."
' >/dev/null; then
  echo "expected deterministic PASS convergence summary json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/summary.md"
${summary_script} --closure-json "${closure_pass}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected convergence summary markdown output file" >&2
  exit 1
fi
if ! rg -Fq '| editor | M18-D |' "${markdown_output}"; then
  echo "expected markdown convergence summary to include editor row" >&2
  exit 1
fi
if ! rg -Fq '`PASS`' "${markdown_output}"; then
  echo "expected markdown convergence summary to include PASS status" >&2
  exit 1
fi

closure_pending="${tmp_dir}/closure-pending.json"
cat > "${closure_pending}" <<'JSON'
{
  "repo": ".",
  "overall": "PENDING",
  "message": "1 check pending",
  "pendingCount": 1,
  "gates": [
    {"gate":"M18-D","status":"PASS","check":"editor","evidence":"x"},
    {"gate":"M18-E","status":"PENDING","check":"release","evidence":"x"},
    {"gate":"M18-F","status":"PASS","check":"runtime","evidence":"x"}
  ]
}
JSON

pending_json="$(${summary_script} --closure-json "${closure_pending}" --format json)"
if ! printf '%s\n' "${pending_json}" | jq -e '
  .overall == "PENDING"
  and (.tracks[] | select(.gate == "M18-E")).status == "PENDING"
  and .nextAction == "Resolve pending M18 track gates before transition."
' >/dev/null; then
  echo "expected deterministic PENDING convergence summary json contract" >&2
  exit 1
fi

closure_missing_gate="${tmp_dir}/closure-missing-gate.json"
cat > "${closure_missing_gate}" <<'JSON'
{
  "repo": ".",
  "overall": "PASS",
  "pendingCount": 0,
  "gates": [
    {"gate":"M18-D","status":"PASS","check":"editor","evidence":"x"},
    {"gate":"M18-F","status":"PASS","check":"runtime","evidence":"x"}
  ]
}
JSON

if ${summary_script} --closure-json "${closure_missing_gate}" --format json >"${tmp_dir}/missing-gate.log" 2>&1; then
  echo "expected convergence summary builder to fail when required M18 gate is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing required M18 gate in closure json: M18-E' "${tmp_dir}/missing-gate.log"; then
  echo "expected missing-gate diagnostic from convergence summary builder" >&2
  exit 1
fi

echo "m18 track convergence summary test passed"
