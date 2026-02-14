#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary_script="${root_dir}/scripts/build-m27-executed-slice-convergence-summary.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

selector_json="${tmp_dir}/selector.json"
cat > "${selector_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendation": {
    "id": "M27-S4-runtime-hardening",
    "closureGate": "M27-C"
  }
}
JSON

runtime_execution_json="${tmp_dir}/runtime-execution.json"
cat > "${runtime_execution_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendationId": "M27-S4-runtime-hardening",
  "closureGate": "M27-C",
  "status": "DRY_RUN"
}
JSON

summary_json="$(${summary_script} --selector-json "${selector_json}" --runtime-execution-json "${runtime_execution_json}" --format json)"
if ! printf '%s\n' "${summary_json}" | jq -e '
  .version == "0.1"
  and .selectedTrack == "runtime"
  and .recommendationId == "M27-S4-runtime-hardening"
  and .closureGate == "M27-C"
  and .runtimeStatus == "DRY_RUN"
  and .executionPass == true
  and .overall == "PASS"
' >/dev/null; then
  echo "expected deterministic PASS executed-slice convergence summary json contract" >&2
  exit 1
fi

markdown_output="${tmp_dir}/summary.md"
${summary_script} --selector-json "${selector_json}" --runtime-execution-json "${runtime_execution_json}" --output "${markdown_output}" >/dev/null
if [ ! -f "${markdown_output}" ]; then
  echo "expected executed-slice convergence summary markdown output file" >&2
  exit 1
fi
if ! rg -Fq '# M27 Executed Slice Convergence Summary' "${markdown_output}"; then
  echo "expected markdown summary heading in executed-slice convergence output" >&2
  exit 1
fi
if ! rg -Fq '`M27-S4-runtime-hardening`' "${markdown_output}"; then
  echo "expected markdown summary to include recommendation id" >&2
  exit 1
fi

runtime_mismatch_json="${tmp_dir}/runtime-mismatch.json"
cat > "${runtime_mismatch_json}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendationId": "M27-S4-release-hardening",
  "status": "PASS"
}
JSON

if ${summary_script} --selector-json "${selector_json}" --runtime-execution-json "${runtime_mismatch_json}" --format json >"${tmp_dir}/mismatch.log" 2>&1; then
  echo "expected executed-slice convergence summary script to fail for recommendation mismatch" >&2
  exit 1
fi
if ! rg -Fq 'runtime execution recommendationId does not match selector recommendation id' "${tmp_dir}/mismatch.log"; then
  echo "expected recommendation mismatch diagnostic from executed-slice convergence summary script" >&2
  exit 1
fi

if ${summary_script} --selector-json "${selector_json}" --runtime-execution-json "${tmp_dir}/missing-runtime.json" --format json >"${tmp_dir}/missing-runtime.log" 2>&1; then
  echo "expected executed-slice convergence summary script to fail when runtime execution json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing runtime execution json:' "${tmp_dir}/missing-runtime.log"; then
  echo "expected missing-runtime diagnostic from executed-slice convergence summary script" >&2
  exit 1
fi

echo "m27 executed-slice convergence summary test passed"
