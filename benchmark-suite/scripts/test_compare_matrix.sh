#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"
cp "$root_dir/scripts/testdata/sample-rust-report.json" "$tmp/rust-report.json"

out="$tmp/compare-matrix.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$out" >/dev/null

if ! jq -e '.version == "0.1"' "$out" >/dev/null; then
  echo "compare-matrix output missing version" >&2
  exit 1
fi

if ! jq -e '.endpoints | length == 3' "$out" >/dev/null; then
  echo "compare-matrix expected 3 endpoints" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .leader.impl == "go"' "$out" >/dev/null; then
  echo "compare-matrix expected go to lead ping" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "decode") | .leader.impl == "rust"' "$out" >/dev/null; then
  echo "compare-matrix expected rust to lead decode" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "users-post") | .leader.impl == "sec4"' "$out" >/dev/null; then
  echo "compare-matrix expected sec4 to lead users-post" >&2
  exit 1
fi
if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .leader.constantRate == true' "$out" >/dev/null; then
  echo "compare-matrix expected default constantRate=true metadata" >&2
  exit 1
fi
if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .leader | has("rssKb")' "$out" >/dev/null; then
  echo "compare-matrix expected rssKb field on leader row" >&2
  exit 1
fi

cat > "$tmp/steady-report.json" <<'EOF'
{
  "version": "0.1",
  "impl": "steady",
  "summaries": [
    {
      "endpoint": "ping",
      "targetRps": 10000,
      "requestsPerSec": 50000,
      "constantRate": true,
      "loadGenerator": "wrk2",
      "latency": {"p99": "10.00ms"}
    }
  ]
}
EOF

cat > "$tmp/burst-report.json" <<'EOF'
{
  "version": "0.1",
  "impl": "burst",
  "summaries": [
    {
      "endpoint": "ping",
      "targetRps": 10000,
      "requestsPerSec": 90000,
      "constantRate": false,
      "loadGenerator": "wrk",
      "latency": {"p99": "2.00ms"}
    }
  ]
}
EOF

quality_out="$tmp/compare-matrix-quality.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$quality_out" "steady,burst" >/dev/null

if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .leader.impl == "steady"' "$quality_out" >/dev/null; then
  echo "compare-matrix should prefer constant-rate leader over higher non-constant throughput" >&2
  exit 1
fi
if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .compared[0].constantRate == true and .compared[1].constantRate == false' "$quality_out" >/dev/null; then
  echo "compare-matrix quality ordering mismatch for constant-rate prioritization" >&2
  exit 1
fi

filtered_out="$tmp/compare-matrix-filtered.json"
"$root_dir/scripts/compare_matrix.sh" "$tmp" "$filtered_out" "node,go" >/dev/null

if ! jq -e '.endpoints[] | select(.endpoint == "ping") | (.compared | length) == 2' "$filtered_out" >/dev/null; then
  echo "compare-matrix filtered run expected 2 compared implementations for ping" >&2
  exit 1
fi

if ! jq -e '.endpoints[] | select(.endpoint == "ping") | .leader.impl == "go"' "$filtered_out" >/dev/null; then
  echo "compare-matrix filtered run expected go to lead ping" >&2
  exit 1
fi

echo "compare_matrix test passed"
