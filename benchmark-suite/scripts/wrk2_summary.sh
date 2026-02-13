#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 5 ]; then
  echo "usage: $0 <raw_wrk2.txt> <impl> <endpoint> <target_rps> <out_summary.json>" >&2
  exit 2
fi

raw="$1"
impl="$2"
endpoint="$3"
target_rps="$4"
out="$5"

if [ ! -f "$raw" ]; then
  echo "raw wrk2 output not found: $raw" >&2
  exit 2
fi

mkdir -p "$(dirname "$out")"

requests_sec="$(awk '/^Requests\/sec:/ {print $2; exit}' "$raw")"
latency_max="$(awk '/^[[:space:]]*Latency[[:space:]]+/ && $1=="Latency" {print $4; exit}' "$raw")"
p50="$(awk '/^[[:space:]]*50(\.000)?%/ {print $2; exit}' "$raw")"
p95="$(awk '/^[[:space:]]*95(\.000)?%/ {print $2; exit}' "$raw")"
p99="$(awk '/^[[:space:]]*99(\.000)?%/ {print $2; exit}' "$raw")"

load_generator="wrk"
constant_rate="false"
if grep -q 'Thread calibration:' "$raw"; then
  load_generator="wrk2"
  constant_rate="true"
fi

cat > "$out" <<JSON
{
  "impl": "${impl}",
  "endpoint": "${endpoint}",
  "targetRps": ${target_rps},
  "requestsPerSec": ${requests_sec:-0},
  "loadGenerator": "${load_generator}",
  "constantRate": ${constant_rate},
  "latency": {
    "p50": "${p50}",
    "p95": "${p95}",
    "p99": "${p99}",
    "max": "${latency_max}"
  }
}
JSON

echo "wrote $out"
