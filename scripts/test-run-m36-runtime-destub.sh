#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner_script="${root_dir}/scripts/run-m36-runtime-destub.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

plan_db="${tmp_dir}/plan-db.json"
cat > "${plan_db}" <<'JSON'
{
  "version": "0.1",
  "selectedTrack": "runtime",
  "stabilizationMode": false,
  "closureGate": "M36-C",
  "plan": [
    {"order":1,"id":"M36-S4-runtime-db-destub","title":"DB de-stub","domain":"db"},
    {"order":2,"id":"M36-S4-runtime-net-destub","title":"Net de-stub","domain":"net"}
  ],
  "selectedSlice": {
    "id": "M36-S4-runtime-db-destub",
    "title": "DB de-stub",
    "domain": "db"
  },
  "selectedSliceRationale": "Top-priority matrix track controls deterministic first-slice runtime de-stub ordering."
}
JSON

dry_run_json="$(${runner_script} --plan-json "${plan_db}" --output-dir "${tmp_dir}/run-dry" --dry-run --format json)"
if ! printf '%s\n' "${dry_run_json}" | jq -e '
  .version == "0.1"
  and .selectedTrack == "runtime"
  and .selectedSlice.id == "M36-S4-runtime-db-destub"
  and .selectedSlice.domain == "db"
  and .runMode == "dry-run"
  and .executionStatus == "DRY_RUN"
  and .executed == false
  and .closureGate == "M36-D"
  and (.plannedSteps | length) == 3
' >/dev/null; then
  echo "expected deterministic dry-run contract from M36 runtime de-stub runner" >&2
  exit 1
fi
if [ ! -f "${tmp_dir}/run-dry/execution-status.json" ]; then
  echo "expected dry-run to emit execution-status.json artifact" >&2
  exit 1
fi

execute_json="$(${runner_script} --plan-json "${plan_db}" --output-dir "${tmp_dir}/run-execute" --execute --format json)"
if ! printf '%s\n' "${execute_json}" | jq -e '
  .runMode == "execute"
  and .executionStatus == "PASS"
  and .executed == true
  and .selectedSlice.id == "M36-S4-runtime-db-destub"
' >/dev/null; then
  echo "expected deterministic execute contract from M36 runtime de-stub runner" >&2
  exit 1
fi
if [ ! -f "${tmp_dir}/run-execute/executed-M36-S4-runtime-db-destub.marker" ]; then
  echo "expected execute mode to emit runtime de-stub marker artifact" >&2
  exit 1
fi

if ${runner_script} --plan-json "${tmp_dir}/missing-plan.json" --dry-run --format json >"${tmp_dir}/missing-plan.log" 2>&1; then
  echo "expected M36 runtime de-stub runner to fail when plan json is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M36 runtime de-stub plan json:' "${tmp_dir}/missing-plan.log"; then
  echo "expected missing-plan diagnostic from M36 runtime de-stub runner" >&2
  exit 1
fi

invalid_plan="${tmp_dir}/invalid-plan.json"
cat > "${invalid_plan}" <<'JSON'
{"version":"0.1","selectedTrack":"runtime","plan":[]}
JSON

if ${runner_script} --plan-json "${invalid_plan}" --dry-run --format json >"${tmp_dir}/invalid-plan.log" 2>&1; then
  echo "expected M36 runtime de-stub runner to fail for invalid plan contract" >&2
  exit 1
fi
if ! rg -Fq 'invalid M36 runtime de-stub plan contract:' "${tmp_dir}/invalid-plan.log"; then
  echo "expected invalid-plan diagnostic from M36 runtime de-stub runner" >&2
  exit 1
fi

echo "m36 runtime de-stub runner test passed"
