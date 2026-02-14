#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner_script="${root_dir}/scripts/run-m18-runtime-track.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

runtime_selector="${tmp_dir}/selector-runtime.json"
cat > "${runtime_selector}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendation": {
    "id": "M18-S6-runtime-confidence-hardening",
    "closureGate": "M18-C"
  }
}
JSON

runner_json="$(${runner_script} --selector-json "${runtime_selector}" --runtime-smoke-root "${tmp_dir}/runtime-smoke" --runtime-smoke-index "${tmp_dir}/runtime-smoke/index.json" --dry-run --format json)"
if ! printf '%s\n' "${runner_json}" | jq -e '
  .version == "0.1"
  and .selectedTrack == "runtime"
  and .recommendationId == "M18-S6-runtime-confidence-hardening"
  and .closureGate == "M18-C"
  and .dryRun == true
  and .status == "DRY_RUN"
  and (.commands | length) == 2
  and (.commands[0] | contains("scripts/check-runtime-smoke-bundle.sh"))
  and (.commands[1] == "scripts/check-milestone-closure.sh --fail-on-pending")
' >/dev/null; then
  echo "expected deterministic runtime-track dry-run json contract" >&2
  exit 1
fi

editor_selector="${tmp_dir}/selector-editor.json"
cat > "${editor_selector}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "editor",
  "recommendation": {
    "id": "M18-S4-editor-contract-expansion",
    "closureGate": "M18-C"
  }
}
JSON

if ${runner_script} --selector-json "${editor_selector}" --dry-run --format json >"${tmp_dir}/editor.log" 2>&1; then
  echo "expected runtime-track runner to fail for non-runtime selected track" >&2
  exit 1
fi
if ! rg -Fq 'selector did not choose runtime track: editor' "${tmp_dir}/editor.log"; then
  echo "expected non-runtime selected-track diagnostic" >&2
  exit 1
fi

invalid_runtime_selector="${tmp_dir}/selector-runtime-invalid-id.json"
cat > "${invalid_runtime_selector}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "recommendation": {
    "id": "M18-S5-release-publish-integrity-contract-expansion",
    "closureGate": "M18-C"
  }
}
JSON

if ${runner_script} --selector-json "${invalid_runtime_selector}" --dry-run --format json >"${tmp_dir}/invalid-id.log" 2>&1; then
  echo "expected runtime-track runner to fail for non-M18-S6 runtime recommendation id" >&2
  exit 1
fi
if ! rg -Fq 'selector runtime recommendation id is not an M18-S6 runtime slice' "${tmp_dir}/invalid-id.log"; then
  echo "expected invalid runtime recommendation id diagnostic" >&2
  exit 1
fi

invalid_selector="${tmp_dir}/selector-invalid.json"
cat > "${invalid_selector}" <<'JSON'
{"selectedTrack":"runtime"}
JSON

if ${runner_script} --selector-json "${invalid_selector}" --dry-run --format json >"${tmp_dir}/invalid-selector.log" 2>&1; then
  echo "expected runtime-track runner to fail for invalid selector contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid selector contract:' "${tmp_dir}/invalid-selector.log"; then
  echo "expected invalid-selector contract diagnostic" >&2
  exit 1
fi

echo "m18 runtime-track runner test passed"
