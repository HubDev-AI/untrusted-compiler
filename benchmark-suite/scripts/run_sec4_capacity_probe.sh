#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs the sec4 benchmark service under load, samples peak RSS, and writes a
capacity probe result JSON.

Options:
  --endpoint <ping|decode|users-post|users-get>   Benchmark endpoint (default: ping)
  --duration <duration>                            Load duration (default: 40s)
  --target-rps <n>                                 Target requests/sec passed to run_profile (default: 50000)
  --threads <n>                                    Load generator threads (default: 8)
  --connections <n>                                Load generator connections (default: 256)
  --target-requests <n>                            Minimum total requests required to pass (default: 1000000)
  --port <n>                                       Service port (default: 18086)
  --out <path>                                     Output JSON path (default: results/summaries/sec4-capacity-probe.json)
  --skip-build                                     Skip sec4 benchmark service rebuild
  --dry-run                                        Print execution plan only
  -h, --help                                       Show this help
USAGE
}

is_number() {
  local value="$1"
  awk -v x="$value" 'BEGIN { exit !(x ~ /^-?[0-9]+([.][0-9]+)?$/) }'
}

endpoint="${BENCH_CAPACITY_ENDPOINT:-ping}"
duration="${BENCH_CAPACITY_DURATION:-40s}"
target_rps="${BENCH_CAPACITY_TARGET_RPS:-50000}"
threads="${BENCH_CAPACITY_THREADS:-8}"
connections="${BENCH_CAPACITY_CONNECTIONS:-256}"
target_requests="${BENCH_CAPACITY_TARGET_REQUESTS:-1000000}"
port="${BENCH_CAPACITY_PORT:-18086}"
out_rel="${BENCH_CAPACITY_OUT:-results/summaries/sec4-capacity-probe.json}"
skip_build="false"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --endpoint)
      endpoint="${2:-}"
      shift 2
      ;;
    --duration)
      duration="${2:-}"
      shift 2
      ;;
    --target-rps)
      target_rps="${2:-}"
      shift 2
      ;;
    --threads)
      threads="${2:-}"
      shift 2
      ;;
    --connections)
      connections="${2:-}"
      shift 2
      ;;
    --target-requests)
      target_requests="${2:-}"
      shift 2
      ;;
    --port)
      port="${2:-}"
      shift 2
      ;;
    --out)
      out_rel="${2:-}"
      shift 2
      ;;
    --skip-build)
      skip_build="true"
      shift
      ;;
    --dry-run)
      dry_run="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

case "$endpoint" in
  ping|decode|users-post|users-get) ;;
  *)
    echo "unsupported endpoint: $endpoint" >&2
    exit 2
    ;;
esac

if ! is_number "$target_rps"; then
  echo "target-rps must be numeric, got: $target_rps" >&2
  exit 2
fi
if ! is_number "$threads"; then
  echo "threads must be numeric, got: $threads" >&2
  exit 2
fi
if ! is_number "$connections"; then
  echo "connections must be numeric, got: $connections" >&2
  exit 2
fi
if ! is_number "$target_requests"; then
  echo "target-requests must be numeric, got: $target_requests" >&2
  exit 2
fi
if ! is_number "$port"; then
  echo "port must be numeric, got: $port" >&2
  exit 2
fi

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
service_dir="${root_dir}/services/sec4"
base_url="http://127.0.0.1:${port}"

if [[ "$out_rel" = /* ]]; then
  out_path="$out_rel"
elif [[ "$out_rel" == benchmark-suite/* ]]; then
  out_path="${repo_root}/${out_rel}"
else
  out_path="${root_dir}/${out_rel}"
fi

raw_file="${root_dir}/results/raw/sec4-${endpoint}.txt"
summary_file="${root_dir}/results/summaries/sec4-${endpoint}.json"
server_log="${root_dir}/results/raw/sec4-capacity-server.log"
run_log="${root_dir}/results/raw/sec4-capacity-run.log"
rss_peak_file="${root_dir}/results/raw/sec4-capacity-peak-rss-kb.txt"

mkdir -p "$(dirname "$out_path")" "$(dirname "$raw_file")" "$(dirname "$summary_file")"

cat <<PLAN
sec4 capacity probe plan:
  endpoint=$endpoint
  baseUrl=$base_url
  duration=$duration
  targetRps=$target_rps
  threads=$threads
  connections=$connections
  targetRequests=$target_requests
  skipBuild=$skip_build
  out=$out_path
PLAN

if [ "$dry_run" = "true" ]; then
  exit 0
fi

for required_bin in curl jq ps; do
  if ! command -v "$required_bin" >/dev/null 2>&1; then
    echo "required command not found: $required_bin" >&2
    exit 127
  fi
done

if [ "$skip_build" != "true" ]; then
  "${service_dir}/build.sh"
fi

server_pid=""
sampler_pid=""
cleanup() {
  if [ -n "$sampler_pid" ] && kill -0 "$sampler_pid" >/dev/null 2>&1; then
    kill "$sampler_pid" >/dev/null 2>&1 || true
    wait "$sampler_pid" >/dev/null 2>&1 || true
  fi
  if [ -n "$server_pid" ] && kill -0 "$server_pid" >/dev/null 2>&1; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

PORT="$port" "${service_dir}/sec4-bench-server" >"$server_log" 2>&1 &
server_pid=$!

ready="false"
for _ in $(seq 1 200); do
  if curl -fsS "${base_url}/ping" >/dev/null 2>&1; then
    ready="true"
    break
  fi
  if ! kill -0 "$server_pid" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

if [ "$ready" != "true" ]; then
  echo "sec4 capacity probe failed: service did not become ready on ${base_url}" >&2
  exit 1
fi

echo 0 >"$rss_peak_file"
(
  while kill -0 "$server_pid" >/dev/null 2>&1; do
    rss_kb="$(ps -o rss= -p "$server_pid" | tr -d ' ' || true)"
    if is_number "$rss_kb"; then
      current="$(cat "$rss_peak_file")"
      if [ "$rss_kb" -gt "$current" ]; then
        echo "$rss_kb" >"$rss_peak_file"
      fi
    fi
    sleep 0.1
  done
) &
sampler_pid=$!

bench_rc=0
if ! BENCH_DURATION="$duration" \
  BENCH_THREADS="$threads" \
  BENCH_CONNECTIONS="$connections" \
  BENCH_TARGET="$target_rps" \
  "${root_dir}/scripts/run_profile.sh" sec4 "$endpoint" "$base_url" >"$run_log" 2>&1; then
  bench_rc=$?
fi

if kill -0 "$server_pid" >/dev/null 2>&1; then
  kill "$server_pid" >/dev/null 2>&1 || true
  wait "$server_pid" >/dev/null 2>&1 || true
fi
if kill -0 "$sampler_pid" >/dev/null 2>&1; then
  kill "$sampler_pid" >/dev/null 2>&1 || true
  wait "$sampler_pid" >/dev/null 2>&1 || true
fi

observed_requests=0
observed_requests_per_sec=0
peak_rss_kb=0
load_generator=""
constant_rate="false"
p99=""

if [ -f "$raw_file" ]; then
  observed_requests_raw="$(awk '/requests in/ {gsub(/,/,"",$1); print $1; exit}' "$raw_file")"
  if is_number "$observed_requests_raw"; then
    observed_requests="$observed_requests_raw"
  fi
  observed_requests_per_sec_raw="$(awk '/^Requests\/sec:/ {print $2; exit}' "$raw_file")"
  if is_number "$observed_requests_per_sec_raw"; then
    observed_requests_per_sec="$observed_requests_per_sec_raw"
  fi
fi

if [ -f "$rss_peak_file" ]; then
  peak_rss_raw="$(cat "$rss_peak_file")"
  if is_number "$peak_rss_raw"; then
    peak_rss_kb="$peak_rss_raw"
  fi
fi

if [ -f "$summary_file" ] && command -v jq >/dev/null 2>&1; then
  load_generator="$(jq -r '.loadGenerator // ""' "$summary_file" 2>/dev/null || true)"
  constant_rate="$(jq -r '.constantRate // false' "$summary_file" 2>/dev/null || echo false)"
  p99="$(jq -r '.latency.p99 // ""' "$summary_file" 2>/dev/null || true)"
fi

target_met="false"
if awk -v observed="$observed_requests" -v target="$target_requests" 'BEGIN { exit !(observed + 0 >= target + 0) }'; then
  target_met="true"
fi

overall_pass="false"
if [ "$bench_rc" -eq 0 ] && [ "$target_met" = "true" ]; then
  overall_pass="true"
fi

jq -n \
  --arg impl "sec4" \
  --arg endpoint "$endpoint" \
  --arg baseUrl "$base_url" \
  --arg duration "$duration" \
  --arg loadGenerator "$load_generator" \
  --arg p99 "$p99" \
  --arg rawFile "$raw_file" \
  --arg summaryFile "$summary_file" \
  --arg serverLog "$server_log" \
  --arg runLog "$run_log" \
  --argjson targetRps "$target_rps" \
  --argjson threads "$threads" \
  --argjson connections "$connections" \
  --argjson targetRequests "$target_requests" \
  --argjson observedRequests "$observed_requests" \
  --argjson observedRequestsPerSec "$observed_requests_per_sec" \
  --argjson peakRssKb "$peak_rss_kb" \
  --argjson runExitCode "$bench_rc" \
  --argjson constantRate "$constant_rate" \
  --argjson requestsTargetMet "$target_met" \
  --argjson pass "$overall_pass" \
  '{
    impl: $impl,
    endpoint: $endpoint,
    baseUrl: $baseUrl,
    pass: $pass,
    runExitCode: $runExitCode,
    requestsTargetMet: $requestsTargetMet,
    run: {
      duration: $duration,
      targetRps: $targetRps,
      threads: $threads,
      connections: $connections,
      targetRequests: $targetRequests
    },
    observed: {
      requests: $observedRequests,
      requestsPerSec: $observedRequestsPerSec,
      peakRssKb: $peakRssKb,
      loadGenerator: $loadGenerator,
      constantRate: $constantRate,
      p99: $p99
    },
    artifacts: {
      raw: $rawFile,
      summary: $summaryFile,
      serverLog: $serverLog,
      runLog: $runLog
    }
  }' >"$out_path"

echo "wrote $out_path"
if [ "$overall_pass" != "true" ]; then
  echo "capacity probe failed: pass=false (runExitCode=$bench_rc, observedRequests=$observed_requests, targetRequests=$target_requests)" >&2
  exit 1
fi

echo "capacity probe passed"
