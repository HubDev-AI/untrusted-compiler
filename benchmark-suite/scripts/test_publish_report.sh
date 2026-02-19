#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"
cp "$root_dir/scripts/testdata/sample-rust-report.json" "$tmp/rust-report.json"

matrix="$tmp/compare-matrix.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$matrix" >/dev/null

analysis="$tmp/analysis.json"
"$root_dir/scripts/analyze_matrix.sh" "$matrix" "$analysis" >/dev/null

out="$tmp/report.md"
"$root_dir/scripts/publish_report.sh" "$matrix" "$out" "$root_dir/../baselines/sec-audit/default-secure-prod.hello.json" "$analysis" "$root_dir/scripts/testdata/sample-step-matrix.json" "$root_dir/scripts/testdata/sample-saturation-boost-summary.md" >/dev/null

if ! grep -q '^# Benchmark Comparative Report (v0.1)$' "$out"; then
  echo "missing report title" >&2
  exit 1
fi

if ! grep -q '^## Endpoint Leaders$' "$out"; then
  echo "missing endpoint leaders section" >&2
  exit 1
fi
if ! grep -q '^## Evidence Quality$' "$out"; then
  echo "missing evidence quality section" >&2
  exit 1
fi
if ! grep -q '^- Run mode: constant-rate$' "$out"; then
  echo "missing constant-rate evidence mode summary" >&2
  exit 1
fi
if ! grep -q '^- Quality status: PASS$' "$out"; then
  echo "missing PASS evidence quality status" >&2
  exit 1
fi

if ! grep -q '^- Implementations in matrix (4):' "$out"; then
  echo "missing matrix implementation scope header" >&2
  exit 1
fi
if ! grep -q 'sec4' "$out" || ! grep -q 'go' "$out" || ! grep -q 'node' "$out" || ! grep -q 'rust' "$out"; then
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

if ! grep -q '^## Step-Load Signals$' "$out"; then
  echo "missing step-load signals section" >&2
  exit 1
fi
if ! grep -q 'Knee detections: 2' "$out"; then
  echo "missing step-load knee detection summary" >&2
  exit 1
fi
if ! grep -q 'decode: leader=go, kneeTarget=3000' "$out"; then
  echo "missing step-load leader detail" >&2
  exit 1
fi

if ! grep -q 'Policy: default-secure-prod' "$out"; then
  echo "missing security policy summary" >&2
  exit 1
fi
if ! grep -q '^## LASM Saturation Boost Tuning$' "$out"; then
  echo "missing saturation boost section" >&2
  exit 1
fi
if ! grep -q 'Recommended boost step: 4' "$out"; then
  echo "missing saturation boost recommendation summary" >&2
  exit 1
fi
if ! grep -q 'Selection mode: pass-first' "$out"; then
  echo "missing saturation boost selection mode summary" >&2
  exit 1
fi

echo "publish_report test passed"
