#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/results/summaries"

cat > "$tmp/results/benchmark-report.md" <<'MD'
# Benchmark Comparative Report (v0.1)
MD

cat > "$tmp/results/summaries/compare-matrix.json" <<'JSON'
{"version":"0.1","endpoints":[{"endpoint":"ping","compared":[{"impl":"node","requestsPerSec":1,"targetRps":1,"p99":"1.0ms"}],"leader":{"impl":"node","requestsPerSec":1,"targetRps":1,"p99":"1.0ms"}}]}
JSON

cat > "$tmp/results/summaries/analysis.json" <<'JSON'
{"version":"0.1","endpoints":[{"endpoint":"ping","metrics":{"p99MinMs":1,"p99MaxMs":1,"p99SpreadX":1,"leaderTargetCoveragePct":100},"findings":[]}],"summary":{"endpointCount":1,"findingCounts":{"LOW":0,"MEDIUM":0,"HIGH":0,"CRITICAL":0},"highestSeverity":"LOW"}}
JSON

cat > "$tmp/results/summaries/step-matrix.json" <<'JSON'
{"version":"0.1","endpoints":[{"endpoint":"ping","compared":[{"impl":"node","kneeDetected":false,"kneeAtTargetRps":0,"kneeObservedRps":0,"achievedRatioMin":1,"achievedRatioMax":1,"p99MinMs":1,"p99MaxMs":1,"stepCount":1}],"leader":{"impl":"node","kneeDetected":false,"kneeAtTargetRps":0,"kneeObservedRps":0,"achievedRatioMin":1,"achievedRatioMax":1,"p99MinMs":1,"p99MaxMs":1,"stepCount":1}}],"summary":{"endpointCount":1,"implementationCount":1,"kneeDetectedCount":0}}
JSON

cat > "$tmp/results/summaries/node-report.json" <<'JSON'
{"version":"0.1","impl":"node","summaries":[{"endpoint":"ping","targetRps":1,"requestsPerSec":1,"latency":{"p99":"1.0ms"}}]}
JSON

cat > "$tmp/results/summaries/node-ping.json" <<'JSON'
{"version":"0.1","impl":"node","endpoint":"ping","targetRps":1,"requestsPerSec":1,"latency":{"p99":"1.0ms"}}
JSON

cat > "$tmp/results/summaries/node-ping-step.json" <<'JSON'
{"version":"0.1","impl":"node","endpoint":"ping","stepRates":[1],"stepDuration":"1s","steps":[{"targetRps":1,"requestsPerSec":1,"latency":{"p99":"1.0ms"}}]}
JSON

cat > "$tmp/results/summaries/node-ping-step-analysis.json" <<'JSON'
{"version":"0.1","impl":"node","endpoint":"ping","stepCount":1,"summary":{"achievedRatioMin":1,"achievedRatioMax":1,"p99MinMs":1,"p99MaxMs":1,"kneeDetected":false,"kneeAtTargetRps":0,"kneeObservedRps":0}}
JSON

"$root_dir/scripts/build_artifact_manifest.sh" "$tmp/results" "$tmp/results/artifact-manifest.json" >/dev/null

"$root_dir/scripts/verify_benchmark_bundle.sh" "$tmp/results" "node" "ping" >/dev/null

# mutate an artifact (keep valid JSON) to force hash mismatch only
cat > "$tmp/results/summaries/node-ping-step-analysis.json" <<'JSON'
{"version":"0.1","impl":"node","endpoint":"ping","stepCount":1,"summary":{"achievedRatioMin":0.5,"achievedRatioMax":1,"p99MinMs":1,"p99MaxMs":1,"kneeDetected":false,"kneeAtTargetRps":0,"kneeObservedRps":0}}
JSON
if "$root_dir/scripts/verify_benchmark_bundle.sh" "$tmp/results" "node" "ping" >/dev/null 2>&1; then
  echo "expected hash mismatch to fail verification" >&2
  exit 1
fi

# skip-hash-check should still pass structural checks with mutated file present
if ! "$root_dir/scripts/verify_benchmark_bundle.sh" --skip-hash-check "$tmp/results" "node" "ping" >/dev/null 2>&1; then
  echo "expected skip-hash-check to pass with structural artifacts present" >&2
  exit 1
fi

echo "verify_benchmark_bundle test passed"
