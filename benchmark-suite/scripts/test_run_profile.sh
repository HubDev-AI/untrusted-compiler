#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
out="$("${root_dir}/scripts/run_profile.sh" --dry-run sec4 ping 2>&1)"

if ! grep -q 'endpoint=ping' <<<"$out"; then
  echo "run_profile dry-run missing endpoint output" >&2
  exit 1
fi
if ! grep -q 'targetRps=10000' <<<"$out"; then
  echo "run_profile dry-run missing targetRps output" >&2
  exit 1
fi
if ! grep -q 'command: wrk2 --latency' <<<"$out"; then
  echo "run_profile dry-run missing wrk2 command" >&2
  exit 1
fi

override_out="$(BENCH_THREADS=2 BENCH_CONNECTIONS=16 BENCH_DURATION=7s BENCH_TARGET=1234 "${root_dir}/scripts/run_profile.sh" --dry-run sec4 decode 2>&1)"
if ! grep -q 'targetRps=1234' <<<"$override_out"; then
  echo "run_profile override missing targetRps output" >&2
  exit 1
fi
if ! grep -q -- '-t2 -c16 -d7s -R1234' <<<"$override_out"; then
  echo "run_profile override did not apply threads/connections/duration/target" >&2
  exit 1
fi

users_get_out="$(BENCH_TARGET_USERS_GET=3456 "${root_dir}/scripts/run_profile.sh" --dry-run sec4 users-get 2>&1)"
if ! grep -q 'endpoint=users-get' <<<"$users_get_out"; then
  echo "run_profile users-get dry-run missing endpoint output" >&2
  exit 1
fi
if ! grep -q 'targetRps=3456' <<<"$users_get_out"; then
  echo "run_profile users-get override missing targetRps output" >&2
  exit 1
fi
if ! grep -q 'users-get seedUserId: 6f1c2e7c-9c4a-4d0d-8c1b-2b59a4c8f8e1' <<<"$users_get_out"; then
  echo "run_profile users-get dry-run missing seed user id output" >&2
  exit 1
fi

echo "run_profile test passed"
