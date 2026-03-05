#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 5 ] || [ "$#" -gt 7 ]; then
  echo "usage: $0 <raw_wrk2.txt> <impl> <endpoint> <target_rps> <out_summary.json> [rss_kb] [rss_source]" >&2
  exit 2
fi

raw="$1"
impl="$2"
endpoint="$3"
target_rps="$4"
out="$5"
rss_kb="${6:-}"
rss_source="${7:-unavailable}"

if [ ! -f "$raw" ]; then
  echo "raw wrk2 output not found: $raw" >&2
  exit 2
fi

rss_kb_json="null"
if [ -n "$rss_kb" ]; then
  if ! [[ "$rss_kb" =~ ^[0-9]+$ ]]; then
    echo "rss_kb must be an integer value when provided: ${rss_kb}" >&2
    exit 2
  fi
  rss_kb_json="$rss_kb"
fi

if [ -z "$rss_source" ]; then
  rss_source="unavailable"
fi

mkdir -p "$(dirname "$out")"

requests_sec="$(awk '/^Requests\/sec:/ {print $2; exit}' "$raw")"
latency_max="$(awk '/^[[:space:]]*Latency[[:space:]]+/ && $1=="Latency" {print $4; exit}' "$raw")"
p50="$(awk '/^[[:space:]]*50(\.000)?%/ {print $2; exit}' "$raw")"
p95="$(awk '/^[[:space:]]*95(\.000)?%/ {print $2; exit}' "$raw")"
p99="$(awk '/^[[:space:]]*99(\.000)?%/ {print $2; exit}' "$raw")"

load_generator="wrk"
constant_rate="false"
load_bin_marker="$(awk -F= '/^# sec4-bench-load-bin=/{print $2; exit}' "$raw")"
load_supports_rate_marker="$(awk -F= '/^# sec4-bench-load-supports-rate=/{print $2; exit}' "$raw")"

if [ "$load_supports_rate_marker" = "true" ]; then
  load_generator="wrk2"
  constant_rate="true"
elif [ "$load_supports_rate_marker" = "false" ]; then
  if printf '%s' "$load_bin_marker" | grep -qi 'wrk2'; then
    load_generator="wrk2"
  fi
elif grep -q 'Thread calibration:' "$raw"; then
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
  "memory": {
    "rssKb": ${rss_kb_json},
    "sampleSource": "${rss_source}"
  },
  "latency": {
    "p50": "${p50}",
    "p95": "${p95}",
    "p99": "${p99}",
    "max": "${latency_max}"
  }
}
JSON

echo "wrote $out"
