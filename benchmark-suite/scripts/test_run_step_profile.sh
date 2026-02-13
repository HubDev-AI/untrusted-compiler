#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$("${root_dir}/scripts/run_step_profile.sh" --dry-run sec4 decode 2>&1)"

if ! grep -q 'step profile impl=sec4 endpoint=decode rate=1000 duration=30s' <<<"$out"; then
  echo "run_step_profile missing default decode step 1000" >&2
  exit 1
fi
if ! grep -q 'step profile impl=sec4 endpoint=decode rate=2000 duration=30s' <<<"$out"; then
  echo "run_step_profile missing default decode step 2000" >&2
  exit 1
fi
if ! grep -q 'step profile impl=sec4 endpoint=decode rate=3000 duration=30s' <<<"$out"; then
  echo "run_step_profile missing default decode step 3000" >&2
  exit 1
fi

override_out="$(BENCH_STEP_RATES=111,222 BENCH_STEP_DURATION=9s "${root_dir}/scripts/run_step_profile.sh" --dry-run sec4 ping 2>&1)"
if ! grep -q 'rate=111 duration=9s' <<<"$override_out"; then
  echo "run_step_profile missing override step 111" >&2
  exit 1
fi
if ! grep -q 'rate=222 duration=9s' <<<"$override_out"; then
  echo "run_step_profile missing override step 222" >&2
  exit 1
fi

if "${root_dir}/scripts/run_step_profile.sh" --dry-run sec4 unknown >/dev/null 2>&1; then
  echo "expected unknown endpoint to fail" >&2
  exit 1
fi

echo "run_step_profile test passed"
