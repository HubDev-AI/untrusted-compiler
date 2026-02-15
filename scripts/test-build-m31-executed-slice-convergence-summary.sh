#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary_script="${root_dir}/scripts/build-m31-executed-slice-convergence-summary.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

plan_json="${tmp_dir}/plan.json"
cat > "${plan_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "closureGate": "M31-C",
  "plan": [
    {"order":1,"id":"M31-S4-runtime-db-destub","title":"DB de-stub","domain":"db"}
  ]
}
JSON

runtime_execution_json="${tmp_dir}/runtime-execution.json"
cat > "${runtime_execution_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSlice": {
    "id": "M31-S4-runtime-db-destub",
    "domain": "db"
  },
  "executionStatus": "DRY_RUN",
  "closureGate": "M31-D"
}
JSON

summary_json="$(${summary_script} --plan-json "${plan_json}" --runtime-execution-json "${runtime_execution_json}" --format json)"
if ! printf '%s\n' "${summary_json}" | jq -e '
  .version == "0.1"
  and .selectedTrack == "runtime"
  and .selectedSliceId == "M31-S4-runtime-db-destub"
  and .planClosureGate == "M31-C"
  and .runtimeClosureGate == "M31-D"
  and .runtimeStatus == "DRY_RUN"
  and .executionPass == true
  and .overall == "PASS"
  and .convergenceClosureGate == "M31-E"
' >/dev/null; then
  echo "expected deterministic PASS executed-slice convergence summary json contract for M31" >&2
  exit 1
fi

markdown_output="${tmp_dir}/summary.md"
${summary_script} --plan-json "${plan_json}" --runtime-execution-json "${runtime_execution_json}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected M31 executed-slice convergence summary markdown output file" >&2
  exit 1
fi
if ! rg -Fq '# M31 Executed Slice Convergence Summary' "${markdown_output}"; then
  echo "expected markdown summary heading in M31 executed-slice convergence output" >&2
  exit 1
fi
if ! rg -Fq '`M31-S4-runtime-db-destub`' "${markdown_output}"; then
  echo "expected markdown summary to include selected slice id" >&2
  exit 1
fi

runtime_mismatch_json="${tmp_dir}/runtime-mismatch.json"
cat > "${runtime_mismatch_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "selectedSlice": {
    "id": "M31-S4-runtime-net-destub",
    "domain": "net"
  },
  "executionStatus": "PASS",
  "closureGate": "M31-D"
}
JSON

if ${summary_script} --plan-json "${plan_json}" --runtime-execution-json "${runtime_mismatch_json}" --format json >"${tmp_dir}/mismatch.log" 2>&1; then
  echo "expected M31 executed-slice convergence summary to fail for selected-slice mismatch" >&2
  exit 1
fi
if ! rg -Fq 'runtime execution selectedSlice.id does not match plan slice id' "${tmp_dir}/mismatch.log"; then
  echo "expected selected-slice mismatch diagnostic from M31 executed-slice convergence summary" >&2
  exit 1
fi

if ${summary_script} --plan-json "${plan_json}" --runtime-execution-json "${tmp_dir}/missing-runtime.json" --format json >"${tmp_dir}/missing-runtime.log" 2>&1; then
  echo "expected M31 executed-slice convergence summary to fail when runtime execution json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M31 runtime execution json:' "${tmp_dir}/missing-runtime.log"; then
  echo "expected missing-runtime diagnostic from M31 executed-slice convergence summary" >&2
  exit 1
fi

echo "m31 executed-slice convergence summary test passed"
