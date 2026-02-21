#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs sec4 LASM cluster mode under load, samples peak RSS, and writes a
capacity probe result JSON.

Options:
  --project-path <path>                            Project path passed to sec4 run (default: examples/lasm-alpha-full)
  --request-path <path>                            Probe HTTP path (default: /health)
  --request-header <value>                         Header passed to readiness + wrk (default: Authorization: Bearer token123)
  --duration <duration>                            wrk duration (default: 40s)
  --threads <n>                                    wrk threads (default: 8)
  --connections <n>                                wrk connections (default: 256)
  --target-requests <n>                            Minimum total requests required to pass (default: 1000000)
  --port <n>                                       Service port (default: 18096)
  --instances <n>                                  LASM min instances (default: 4)
  --autoscale-max-instances <n>                    LASM max instances (default: 8)
  --autoscale-target-connections <n>               LASM autoscale target per instance (default: 256)
  --autoscale-check-ms <n>                         LASM autoscale check interval (default: 1000)
  --autoscale-scale-up-cooldown-ms <n>             LASM scale-up cooldown (default: 250)
  --autoscale-scale-down-cooldown-ms <n>           LASM scale-down cooldown (default: 2000)
  --autoscale-scale-up-step <n>                    LASM max scale-up workers per autoscale check (default: 2)
  --autoscale-scale-down-step <n>                  LASM max scale-down workers per autoscale check (default: 1)
  --autoscale-saturation-boost-step <n>            LASM max scale-up workers per check when relay saturation is observed (default: 4)
  --cluster-relay-workers <n>                      Optional relay worker override
  --cluster-relay-queue <n>                        Optional relay queue override
  --cluster-accept-workers <n>                     Optional relay accept-worker override
  --cluster-relay-accept-batch-max <n>             Optional relay accept batch max override
  --out <path>                                     Output JSON path (default: results/summaries/sec4-lasm-cluster-capacity-probe.json)
  --skip-build                                     Skip sec4 binary rebuild
  --dry-run                                        Print execution plan only
  -h, --help                                       Show this help
USAGE
}

is_number() {
  local value="$1"
  awk -v x="$value" 'BEGIN { exit !(x ~ /^-?[0-9]+([.][0-9]+)?$/) }'
}

project_path="${LASM_CAPACITY_PROJECT_PATH:-examples/lasm-alpha-full}"
request_path="${LASM_CAPACITY_REQUEST_PATH:-/health}"
request_header="${LASM_CAPACITY_REQUEST_HEADER:-Authorization: Bearer token123}"
duration="${LASM_CAPACITY_DURATION:-40s}"
threads="${LASM_CAPACITY_THREADS:-8}"
connections="${LASM_CAPACITY_CONNECTIONS:-256}"
target_requests="${LASM_CAPACITY_TARGET_REQUESTS:-1000000}"
port="${LASM_CAPACITY_PORT:-18096}"
instances="${LASM_CAPACITY_INSTANCES:-4}"
autoscale_max_instances="${LASM_CAPACITY_AUTOSCALE_MAX_INSTANCES:-8}"
autoscale_target_connections="${LASM_CAPACITY_AUTOSCALE_TARGET_CONNECTIONS:-256}"
autoscale_check_ms="${LASM_CAPACITY_AUTOSCALE_CHECK_MS:-1000}"
autoscale_scale_up_cooldown_ms="${LASM_CAPACITY_AUTOSCALE_SCALE_UP_COOLDOWN_MS:-250}"
autoscale_scale_down_cooldown_ms="${LASM_CAPACITY_AUTOSCALE_SCALE_DOWN_COOLDOWN_MS:-2000}"
autoscale_scale_up_step="${LASM_CAPACITY_AUTOSCALE_SCALE_UP_STEP:-2}"
autoscale_scale_down_step="${LASM_CAPACITY_AUTOSCALE_SCALE_DOWN_STEP:-1}"
autoscale_saturation_boost_step="${LASM_CAPACITY_AUTOSCALE_SATURATION_BOOST_STEP:-4}"
cluster_relay_workers="${LASM_CAPACITY_CLUSTER_RELAY_WORKERS:-}"
cluster_relay_queue="${LASM_CAPACITY_CLUSTER_RELAY_QUEUE:-}"
cluster_accept_workers="${LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS:-}"
cluster_relay_accept_batch_max="${LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX:-}"
out_rel="${LASM_CAPACITY_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe.json}"
skip_build="false"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --project-path)
      project_path="${2:-}"
      shift 2
      ;;
    --request-path)
      request_path="${2:-}"
      shift 2
      ;;
    --request-header)
      request_header="${2:-}"
      shift 2
      ;;
    --duration)
      duration="${2:-}"
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
    --instances)
      instances="${2:-}"
      shift 2
      ;;
    --autoscale-max-instances)
      autoscale_max_instances="${2:-}"
      shift 2
      ;;
    --autoscale-target-connections)
      autoscale_target_connections="${2:-}"
      shift 2
      ;;
    --autoscale-check-ms)
      autoscale_check_ms="${2:-}"
      shift 2
      ;;
    --autoscale-scale-up-cooldown-ms)
      autoscale_scale_up_cooldown_ms="${2:-}"
      shift 2
      ;;
    --autoscale-scale-down-cooldown-ms)
      autoscale_scale_down_cooldown_ms="${2:-}"
      shift 2
      ;;
    --autoscale-scale-up-step)
      autoscale_scale_up_step="${2:-}"
      shift 2
      ;;
    --autoscale-scale-down-step)
      autoscale_scale_down_step="${2:-}"
      shift 2
      ;;
    --autoscale-saturation-boost-step)
      autoscale_saturation_boost_step="${2:-}"
      shift 2
      ;;
    --cluster-relay-workers)
      cluster_relay_workers="${2:-}"
      shift 2
      ;;
    --cluster-relay-queue)
      cluster_relay_queue="${2:-}"
      shift 2
      ;;
    --cluster-accept-workers)
      cluster_accept_workers="${2:-}"
      shift 2
      ;;
    --cluster-relay-accept-batch-max)
      cluster_relay_accept_batch_max="${2:-}"
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

for field in threads connections target_requests port instances autoscale_max_instances autoscale_target_connections autoscale_check_ms autoscale_scale_up_cooldown_ms autoscale_scale_down_cooldown_ms autoscale_scale_up_step autoscale_scale_down_step autoscale_saturation_boost_step; do
  value="${!field}"
  if ! is_number "$value"; then
    echo "${field//_/-} must be numeric, got: $value" >&2
    exit 2
  fi
done
if [ -n "$cluster_relay_workers" ] && ! is_number "$cluster_relay_workers"; then
  echo "cluster-relay-workers must be numeric, got: $cluster_relay_workers" >&2
  exit 2
fi
if [ -n "$cluster_relay_queue" ] && ! is_number "$cluster_relay_queue"; then
  echo "cluster-relay-queue must be numeric, got: $cluster_relay_queue" >&2
  exit 2
fi
if [ -n "$cluster_accept_workers" ] && ! is_number "$cluster_accept_workers"; then
  echo "cluster-accept-workers must be numeric, got: $cluster_accept_workers" >&2
  exit 2
fi
if [ -n "$cluster_relay_accept_batch_max" ] && ! is_number "$cluster_relay_accept_batch_max"; then
  echo "cluster-relay-accept-batch-max must be numeric, got: $cluster_relay_accept_batch_max" >&2
  exit 2
fi
if [ -z "$request_header" ] || [[ "$request_header" != *:* ]]; then
  echo "request-header must include ':' (example: Authorization: Bearer token123)" >&2
  exit 2
fi

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
sec4_bin="${repo_root}/target/debug/sec4"
if [[ "$project_path" = /* ]]; then
  project_abs="$project_path"
else
  project_abs="${repo_root}/${project_path}"
fi
base_url="http://127.0.0.1:${port}"

if [[ "$out_rel" = /* ]]; then
  out_path="$out_rel"
elif [[ "$out_rel" == benchmark-suite/* ]]; then
  out_path="${repo_root}/${out_rel}"
else
  out_path="${root_dir}/${out_rel}"
fi

probe_label="$(echo "${request_path}" | tr '/?' '__' | tr -cd '[:alnum:]_-')"
probe_label="${probe_label#_}"
if [ -z "$probe_label" ]; then
  probe_label="root"
fi
raw_file="${root_dir}/results/raw/sec4-lasm-cluster-${probe_label}.txt"
server_log="${root_dir}/results/raw/sec4-lasm-cluster-capacity-server.log"
rss_peak_file="${root_dir}/results/raw/sec4-lasm-cluster-capacity-peak-rss-kb.txt"
status_json_file="${root_dir}/results/raw/sec4-lasm-cluster-capacity-status-${port}.json"

mkdir -p "$(dirname "$out_path")" "$(dirname "$raw_file")"

cat <<PLAN
sec4 LASM cluster capacity probe plan:
  projectPath=$project_abs
  baseUrl=$base_url
  requestPath=$request_path
  requestHeader=$request_header
  duration=$duration
  threads=$threads
  connections=$connections
  targetRequests=$target_requests
  instances=$instances
  autoscaleMaxInstances=$autoscale_max_instances
  autoscaleTargetConnections=$autoscale_target_connections
  autoscaleCheckMs=$autoscale_check_ms
  autoscaleScaleUpCooldownMs=$autoscale_scale_up_cooldown_ms
  autoscaleScaleDownCooldownMs=$autoscale_scale_down_cooldown_ms
  autoscaleScaleUpStep=$autoscale_scale_up_step
  autoscaleScaleDownStep=$autoscale_scale_down_step
  autoscaleSaturationBoostStep=$autoscale_saturation_boost_step
  clusterRelayWorkers=${cluster_relay_workers:-auto}
  clusterRelayQueue=${cluster_relay_queue:-auto}
  clusterAcceptWorkers=${cluster_accept_workers:-auto}
  clusterRelayAcceptBatchMax=${cluster_relay_accept_batch_max:-auto}
  clusterStatusJson=${status_json_file}
  skipBuild=$skip_build
  out=$out_path
PLAN

if [ "$dry_run" = "true" ]; then
  exit 0
fi

for required_bin in curl jq ps wrk; do
  if ! command -v "$required_bin" >/dev/null 2>&1; then
    echo "required command not found: $required_bin" >&2
    exit 127
  fi
done
if [ ! -f "${project_abs}/sec4.toml" ]; then
  echo "project not found: ${project_abs}" >&2
  exit 1
fi

if [ "$skip_build" != "true" ]; then
  (cd "$repo_root" && cargo build -p sec4 >/dev/null)
fi
if [ ! -x "$sec4_bin" ]; then
  echo "sec4 binary not found or not executable: $sec4_bin" >&2
  exit 1
fi

run_args=(
  run
  --path "$project_abs"
  --backend lasm
  --port "$port"
  --instances "$instances"
  --autoscale-max-instances "$autoscale_max_instances"
  --autoscale-target-connections "$autoscale_target_connections"
  --autoscale-check-ms "$autoscale_check_ms"
  --autoscale-scale-up-cooldown-ms "$autoscale_scale_up_cooldown_ms"
  --autoscale-scale-down-cooldown-ms "$autoscale_scale_down_cooldown_ms"
  --autoscale-scale-up-step "$autoscale_scale_up_step"
  --autoscale-scale-down-step "$autoscale_scale_down_step"
  --autoscale-saturation-boost-step "$autoscale_saturation_boost_step"
  --cluster-status-json "$status_json_file"
)
if [ -n "$cluster_relay_workers" ]; then
  run_args+=(--cluster-relay-workers "$cluster_relay_workers")
fi
if [ -n "$cluster_relay_queue" ]; then
  run_args+=(--cluster-relay-queue "$cluster_relay_queue")
fi
if [ -n "$cluster_accept_workers" ]; then
  run_args+=(--cluster-accept-workers "$cluster_accept_workers")
fi
if [ -n "$cluster_relay_accept_batch_max" ]; then
  run_args+=(--cluster-relay-accept-batch-max "$cluster_relay_accept_batch_max")
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

"$sec4_bin" "${run_args[@]}" >"$server_log" 2>&1 &
server_pid=$!

ready="false"
for _ in $(seq 1 200); do
  if curl -fsS -H "$request_header" "${base_url}${request_path}" >/dev/null 2>&1; then
    ready="true"
    break
  fi
  if ! kill -0 "$server_pid" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done
if [ "$ready" != "true" ]; then
  echo "LASM capacity probe failed: service did not become ready on ${base_url}${request_path}" >&2
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
if ! wrk -t"$threads" -c"$connections" -d"$duration" --latency -H "$request_header" "${base_url}${request_path}" >"$raw_file" 2>&1; then
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
p99=""
resolved_relay_worker_count="null"
resolved_relay_accept_workers="null"
resolved_relay_accept_batch_max="null"
resolved_relay_queue_capacity="null"
resolved_relay_queue_shard_capacity="null"
if [ -f "$raw_file" ]; then
  observed_requests_raw="$(awk '/requests in/ {gsub(/,/,"",$1); print $1; exit}' "$raw_file")"
  if is_number "$observed_requests_raw"; then
    observed_requests="$observed_requests_raw"
  fi
  observed_requests_per_sec_raw="$(awk '/^Requests\/sec:/ {print $2; exit}' "$raw_file")"
  if is_number "$observed_requests_per_sec_raw"; then
    observed_requests_per_sec="$observed_requests_per_sec_raw"
  fi
  p99_raw="$(awk '$1 == "99%" {print $2; exit}' "$raw_file")"
  if [ -n "$p99_raw" ]; then
    p99="$p99_raw"
  fi
fi
if [ -f "$rss_peak_file" ]; then
  peak_rss_raw="$(cat "$rss_peak_file")"
  if is_number "$peak_rss_raw"; then
    peak_rss_kb="$peak_rss_raw"
  fi
fi
if [ -f "$status_json_file" ]; then
  status_relay_worker_count="$(jq -r '.relayWorkerCount // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_worker_count"; then
    resolved_relay_worker_count="$status_relay_worker_count"
  fi
  status_relay_accept_workers="$(jq -r '.relayAcceptWorkers // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_accept_workers"; then
    resolved_relay_accept_workers="$status_relay_accept_workers"
  fi
  status_relay_accept_batch_max="$(jq -r '.relayAcceptBatchMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_accept_batch_max"; then
    resolved_relay_accept_batch_max="$status_relay_accept_batch_max"
  fi
  status_relay_queue_capacity="$(jq -r '.relayQueueCapacity // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_queue_capacity"; then
    resolved_relay_queue_capacity="$status_relay_queue_capacity"
  fi
  status_relay_queue_shard_capacity="$(jq -r '.relayQueueShardCapacity // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_queue_shard_capacity"; then
    resolved_relay_queue_shard_capacity="$status_relay_queue_shard_capacity"
  fi
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
  --arg impl "sec4-lasm-cluster" \
  --arg projectPath "$project_abs" \
  --arg baseUrl "$base_url" \
  --arg requestPath "$request_path" \
  --arg requestHeader "$request_header" \
  --arg duration "$duration" \
  --arg p99 "$p99" \
  --arg rawFile "$raw_file" \
  --arg serverLog "$server_log" \
  --arg statusJsonFile "$status_json_file" \
  --argjson threads "$threads" \
  --argjson connections "$connections" \
  --argjson targetRequests "$target_requests" \
  --argjson observedRequests "$observed_requests" \
  --argjson observedRequestsPerSec "$observed_requests_per_sec" \
  --argjson peakRssKb "$peak_rss_kb" \
  --argjson runExitCode "$bench_rc" \
  --argjson requestsTargetMet "$target_met" \
  --argjson pass "$overall_pass" \
  --argjson instances "$instances" \
  --argjson autoscaleMaxInstances "$autoscale_max_instances" \
  --argjson autoscaleTargetConnections "$autoscale_target_connections" \
  --argjson autoscaleCheckMs "$autoscale_check_ms" \
  --argjson autoscaleScaleUpCooldownMs "$autoscale_scale_up_cooldown_ms" \
  --argjson autoscaleScaleDownCooldownMs "$autoscale_scale_down_cooldown_ms" \
  --argjson autoscaleScaleUpStep "$autoscale_scale_up_step" \
  --argjson autoscaleScaleDownStep "$autoscale_scale_down_step" \
  --argjson autoscaleSaturationBoostStep "$autoscale_saturation_boost_step" \
  --argjson resolvedRelayWorkerCount "$resolved_relay_worker_count" \
  --argjson resolvedRelayAcceptWorkers "$resolved_relay_accept_workers" \
  --argjson resolvedRelayAcceptBatchMax "$resolved_relay_accept_batch_max" \
  --argjson resolvedRelayQueueCapacity "$resolved_relay_queue_capacity" \
  --argjson resolvedRelayQueueShardCapacity "$resolved_relay_queue_shard_capacity" \
  --arg relayWorkers "${cluster_relay_workers:-auto}" \
  --arg relayQueue "${cluster_relay_queue:-auto}" \
  --arg acceptWorkers "${cluster_accept_workers:-auto}" \
  --arg relayAcceptBatchMax "${cluster_relay_accept_batch_max:-auto}" \
  '{
    impl: $impl,
    projectPath: $projectPath,
    baseUrl: $baseUrl,
    requestPath: $requestPath,
    requestHeader: $requestHeader,
    pass: $pass,
    runExitCode: $runExitCode,
    requestsTargetMet: $requestsTargetMet,
    run: {
      duration: $duration,
      threads: $threads,
      connections: $connections,
      targetRequests: $targetRequests,
      instances: $instances,
      autoscaleMaxInstances: $autoscaleMaxInstances,
      autoscaleTargetConnections: $autoscaleTargetConnections,
      autoscaleCheckMs: $autoscaleCheckMs,
      autoscaleScaleUpCooldownMs: $autoscaleScaleUpCooldownMs,
      autoscaleScaleDownCooldownMs: $autoscaleScaleDownCooldownMs,
      autoscaleScaleUpStep: $autoscaleScaleUpStep,
      autoscaleScaleDownStep: $autoscaleScaleDownStep,
      autoscaleSaturationBoostStep: $autoscaleSaturationBoostStep,
      clusterRelayWorkers: $relayWorkers,
      clusterRelayQueue: $relayQueue,
      clusterAcceptWorkers: $acceptWorkers,
      clusterRelayAcceptBatchMax: $relayAcceptBatchMax,
      clusterRelayWorkersResolved: $resolvedRelayWorkerCount,
      clusterAcceptWorkersResolved: $resolvedRelayAcceptWorkers,
      clusterRelayAcceptBatchMaxResolved: $resolvedRelayAcceptBatchMax,
      clusterRelayQueueCapacityResolved: $resolvedRelayQueueCapacity,
      clusterRelayQueueShardCapacityResolved: $resolvedRelayQueueShardCapacity
    },
    observed: {
      requests: $observedRequests,
      requestsPerSec: $observedRequestsPerSec,
      peakRssKb: $peakRssKb,
      p99: $p99
    },
    artifacts: {
      raw: $rawFile,
      serverLog: $serverLog,
      clusterStatusJson: $statusJsonFile
    }
  }' >"$out_path"

echo "wrote $out_path"
if [ "$overall_pass" != "true" ]; then
  echo "LASM capacity probe failed: pass=false (runExitCode=$bench_rc, observedRequests=$observed_requests, targetRequests=$target_requests)" >&2
  exit 1
fi
echo "LASM capacity probe passed"
