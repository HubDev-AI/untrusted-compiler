#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/scripts/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/scripts/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/scripts/testdata/sample-node-report.json" "$tmp/node-report.json"

out="$tmp/compare.json"
"$root_dir/scripts/compare_reports.sh" "$tmp" ping "$out" >/dev/null

if ! grep -q '"endpoint": "ping"' "$out"; then
  echo "missing endpoint in compare output" >&2
  exit 1
fi
if ! grep -q '"impl": "go"' "$out"; then
  echo "missing go row in compare output" >&2
  exit 1
fi
if ! grep -q '"impl": "sec4"' "$out"; then
  echo "missing sec4 row in compare output" >&2
  exit 1
fi
if ! grep -q '"leader": {' "$out"; then
  echo "missing leader object" >&2
  exit 1
fi
if ! grep -q '"impl": "go"' <(jq '.leader' "$out"); then
  echo "expected go leader for constant-rate sample reports" >&2
  exit 1
fi
if ! jq -e '.leader.constantRate == true and .leader.loadGenerator == "wrk2"' "$out" >/dev/null; then
  echo "expected default quality metadata on leader row" >&2
  exit 1
fi
if ! jq -e '. as $doc | ($doc.compared | all(.[]; .endpoint == "ping")) and ($doc.leader.endpoint == "ping")' "$out" >/dev/null; then
  echo "expected compare-reports rows to carry requested endpoint" >&2
  exit 1
fi
if ! jq -e '. as $doc | any($doc.compared[]; .impl == $doc.leader.impl and .endpoint == $doc.leader.endpoint and .targetRps == $doc.leader.targetRps and .requestsPerSec == $doc.leader.requestsPerSec and .p99 == $doc.leader.p99 and .loadGenerator == $doc.leader.loadGenerator and .constantRate == $doc.leader.constantRate)' "$out" >/dev/null; then
  echo "expected leader row to be present in compared rows" >&2
  exit 1
fi

quality_dir="$tmp/quality"
mkdir -p "$quality_dir"

cat > "$quality_dir/steady-report.json" <<'EOF'
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

cat > "$quality_dir/burst-report.json" <<'EOF'
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

quality_out="$tmp/compare-quality.json"
"$root_dir/scripts/compare_reports.sh" "$quality_dir" ping "$quality_out" >/dev/null

if ! jq -e '.leader.impl == "steady"' "$quality_out" >/dev/null; then
  echo "compare-reports should prefer constant-rate leader over higher non-constant throughput" >&2
  exit 1
fi
if ! jq -e '.compared | length == 2 and .[0].constantRate == true and .[1].constantRate == false' "$quality_out" >/dev/null; then
  echo "compare-reports quality ordering mismatch for constant-rate prioritization" >&2
  exit 1
fi

echo "compare_reports test passed"
