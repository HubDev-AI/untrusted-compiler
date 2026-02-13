#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-ailang-report.json" "$tmp/ailang-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"
cp "$root_dir/scripts/testdata/sample-rust-report.json" "$tmp/rust-report.json"

matrix="$tmp/compare-matrix.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$matrix" >/dev/null

analysis="$tmp/analysis.json"
"$root_dir/scripts/analyze_matrix.sh" "$matrix" "$analysis" >/dev/null

out="$tmp/report.md"
"$root_dir/scripts/publish_report.sh" "$matrix" "$out" "$root_dir/../baselines/sec-audit/default-secure-prod.hello.json" "$analysis" >/dev/null

if ! grep -q '^# Benchmark Comparative Report (v0.1)$' "$out"; then
  echo "missing report title" >&2
  exit 1
fi

if ! grep -q '^## Endpoint Leaders$' "$out"; then
  echo "missing endpoint leaders section" >&2
  exit 1
fi

if ! grep -q '^- Implementations in matrix (4):' "$out"; then
  echo "missing matrix implementation scope header" >&2
  exit 1
fi
if ! grep -q 'ailang' "$out" || ! grep -q 'go' "$out" || ! grep -q 'node' "$out" || ! grep -q 'rust' "$out"; then
  echo "matrix implementation scope header missing expected implementation names" >&2
  exit 1
fi

if ! grep -q '^- Endpoints in matrix (3):' "$out"; then
  echo "missing matrix endpoint scope header" >&2
  exit 1
fi
if ! grep -q 'decode' "$out" || ! grep -q 'ping' "$out" || ! grep -q 'users-post' "$out"; then
  echo "matrix endpoint scope header missing expected endpoint names" >&2
  exit 1
fi

if ! grep -q '| ping | go |' "$out"; then
  echo "missing ping leader row" >&2
  exit 1
fi

if ! grep -q '^## Tail Latency Signals$' "$out"; then
  echo "missing tail latency section" >&2
  exit 1
fi

if ! grep -q '^## Matrix Analysis$' "$out"; then
  echo "missing matrix analysis section" >&2
  exit 1
fi

if ! grep -q 'Highest severity: MEDIUM' "$out"; then
  echo "missing analysis highest severity summary" >&2
  exit 1
fi

if ! grep -q '^## Security Posture$' "$out"; then
  echo "missing security posture section" >&2
  exit 1
fi

if ! grep -q 'Policy: default-secure-prod' "$out"; then
  echo "missing security policy summary" >&2
  exit 1
fi

echo "publish_report test passed"
