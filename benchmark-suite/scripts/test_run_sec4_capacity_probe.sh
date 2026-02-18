#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

out="$("${root_dir}/scripts/run_sec4_capacity_probe.sh" --dry-run --endpoint decode --duration 7s --target-rps 1234 --threads 2 --connections 16 --target-requests 4567 --port 19090 --out results/summaries/custom-capacity.json 2>&1)"

if ! grep -q 'endpoint=decode' <<<"$out"; then
  echo "capacity probe dry-run missing endpoint output" >&2
  exit 1
fi
if ! grep -q 'targetRps=1234' <<<"$out"; then
  echo "capacity probe dry-run missing targetRps output" >&2
  exit 1
fi
if ! grep -q 'targetRequests=4567' <<<"$out"; then
  echo "capacity probe dry-run missing targetRequests output" >&2
  exit 1
fi
if ! grep -q 'baseUrl=http://127.0.0.1:19090' <<<"$out"; then
  echo "capacity probe dry-run missing baseUrl output" >&2
  exit 1
fi
if ! grep -q "out=${root_dir}/results/summaries/custom-capacity.json" <<<"$out"; then
  echo "capacity probe dry-run missing resolved out path" >&2
  exit 1
fi

repo_root="$(cd "${root_dir}/.." && pwd)"
prefixed_out="$("${root_dir}/scripts/run_sec4_capacity_probe.sh" --dry-run --out benchmark-suite/results/summaries/prefixed-capacity.json 2>&1)"
if ! grep -q "out=${repo_root}/benchmark-suite/results/summaries/prefixed-capacity.json" <<<"$prefixed_out"; then
  echo "capacity probe dry-run did not normalize benchmark-suite/ prefixed out path" >&2
  exit 1
fi

if "${root_dir}/scripts/run_sec4_capacity_probe.sh" --dry-run --endpoint bogus >/tmp/capacity-probe-invalid.log 2>&1; then
  echo "capacity probe accepted invalid endpoint" >&2
  exit 1
fi
if ! grep -q 'unsupported endpoint: bogus' /tmp/capacity-probe-invalid.log; then
  echo "capacity probe invalid endpoint error missing" >&2
  exit 1
fi

echo "run_sec4_capacity_probe test passed"
