#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs two LASM capacity probes with the same workload profile:
  1) proxy-relay cluster mode
  2) fixed reuse-port cluster mode

Then emits one comparison JSON artifact with throughput/latency/memory deltas
and a deterministic recommended mode.

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
  --autoscale-max-instances <n>                    LASM max instances for proxy-relay probe (default: 8)
  --autoscale-target-connections <n>               LASM autoscale target per instance (default: 256)
  --autoscale-check-ms <n>                         LASM autoscale check interval (default: 1000)
  --autoscale-scale-up-cooldown-ms <n>             LASM scale-up cooldown (default: 250)
  --autoscale-scale-down-cooldown-ms <n>           LASM scale-down cooldown (default: 2000)
  --autoscale-scale-up-step <n>                    LASM max scale-up workers per autoscale check (default: 2)
  --autoscale-scale-down-step <n>                  LASM max scale-down workers per autoscale check (default: 1)
  --autoscale-saturation-boost-step <n>            LASM max scale-up workers per check when relay saturation is observed (default: 4)
  --cluster-relay-workers <n>                      Optional relay worker override for proxy-relay probe
  --cluster-relay-queue <n>                        Optional relay queue override for proxy-relay probe
  --cluster-accept-workers <n>                     Optional relay accept-worker override for proxy-relay probe
  --cluster-relay-accept-batch-max <n>             Optional relay accept batch max override for proxy-relay probe
  --cluster-relay-pump-batch-max <n>               Optional relay pump batch max override for proxy-relay probe
  --proxy-out <path>                               Proxy probe output path (default: results/summaries/sec4-lasm-cluster-capacity-probe-mode-compare-proxy.json)
  --fixed-out <path>                               Fixed probe output path (default: results/summaries/sec4-lasm-cluster-capacity-probe-mode-compare-fixed.json)
  --out <path>                                     Comparison output path (default: results/summaries/sec4-lasm-cluster-mode-compare.json)
  --skip-build                                     Skip sec4 binary rebuild for first probe
  --dry-run                                        Print execution plan only
  -h, --help                                       Show this help
USAGE
}

is_number() {
  local value="$1"
  awk -v x="$value" 'BEGIN { exit !(x ~ /^-?[0-9]+([.][0-9]+)?$/) }'
}

resolve_path() {
  local root_dir="$1"
  local repo_root="$2"
  local value="$3"
  if [[ "${value}" = /* ]]; then
    printf "%s" "${value}"
  elif [[ "${value}" == benchmark-suite/* ]]; then
    printf "%s" "${repo_root}/${value}"
  else
    printf "%s" "${root_dir}/${value}"
  fi
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
cluster_relay_pump_batch_max="${LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX:-}"
proxy_out_rel="${LASM_CAPACITY_MODE_COMPARE_PROXY_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe-mode-compare-proxy.json}"
fixed_out_rel="${LASM_CAPACITY_MODE_COMPARE_FIXED_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe-mode-compare-fixed.json}"
out_rel="${LASM_CAPACITY_MODE_COMPARE_OUT:-results/summaries/sec4-lasm-cluster-mode-compare.json}"
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
    --cluster-relay-pump-batch-max)
      cluster_relay_pump_batch_max="${2:-}"
      shift 2
      ;;
    --proxy-out)
      proxy_out_rel="${2:-}"
      shift 2
      ;;
    --fixed-out)
      fixed_out_rel="${2:-}"
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
if [ -n "$cluster_relay_pump_batch_max" ] && ! is_number "$cluster_relay_pump_batch_max"; then
  echo "cluster-relay-pump-batch-max must be numeric, got: $cluster_relay_pump_batch_max" >&2
  exit 2
fi
if [ -z "$request_header" ] || [[ "$request_header" != *:* ]]; then
  echo "request-header must include ':' (example: Authorization: Bearer token123)" >&2
  exit 2
fi

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
probe_script="${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh"
if [ ! -x "${probe_script}" ]; then
  echo "missing executable probe script: ${probe_script}" >&2
  exit 1
fi

proxy_out_path="$(resolve_path "${root_dir}" "${repo_root}" "${proxy_out_rel}")"
fixed_out_path="$(resolve_path "${root_dir}" "${repo_root}" "${fixed_out_rel}")"
out_path="$(resolve_path "${root_dir}" "${repo_root}" "${out_rel}")"
mkdir -p "$(dirname "${proxy_out_path}")" "$(dirname "${fixed_out_path}")" "$(dirname "${out_path}")"

cat <<PLAN
sec4 LASM cluster mode compare plan:
  projectPath=${project_path}
  requestPath=${request_path}
  duration=${duration}
  threads=${threads}
  connections=${connections}
  targetRequests=${target_requests}
  instances=${instances}
  autoscaleMaxInstances=${autoscale_max_instances}
  autoscaleTargetConnections=${autoscale_target_connections}
  autoscaleCheckMs=${autoscale_check_ms}
  autoscaleScaleUpCooldownMs=${autoscale_scale_up_cooldown_ms}
  autoscaleScaleDownCooldownMs=${autoscale_scale_down_cooldown_ms}
  autoscaleScaleUpStep=${autoscale_scale_up_step}
  autoscaleScaleDownStep=${autoscale_scale_down_step}
  autoscaleSaturationBoostStep=${autoscale_saturation_boost_step}
  proxyClusterRelayWorkers=${cluster_relay_workers:-auto}
  proxyClusterRelayQueue=${cluster_relay_queue:-auto}
  proxyClusterAcceptWorkers=${cluster_accept_workers:-auto}
  proxyClusterRelayAcceptBatchMax=${cluster_relay_accept_batch_max:-auto}
  proxyClusterRelayPumpBatchMax=${cluster_relay_pump_batch_max:-auto}
  proxyOut=${proxy_out_path}
  fixedOut=${fixed_out_path}
  out=${out_path}
  skipBuild=${skip_build}
PLAN

common_args=(
  --project-path "${project_path}"
  --request-path "${request_path}"
  --request-header "${request_header}"
  --duration "${duration}"
  --threads "${threads}"
  --connections "${connections}"
  --target-requests "${target_requests}"
  --port "${port}"
  --instances "${instances}"
  --autoscale-max-instances "${autoscale_max_instances}"
  --autoscale-target-connections "${autoscale_target_connections}"
  --autoscale-check-ms "${autoscale_check_ms}"
  --autoscale-scale-up-cooldown-ms "${autoscale_scale_up_cooldown_ms}"
  --autoscale-scale-down-cooldown-ms "${autoscale_scale_down_cooldown_ms}"
  --autoscale-scale-up-step "${autoscale_scale_up_step}"
  --autoscale-scale-down-step "${autoscale_scale_down_step}"
  --autoscale-saturation-boost-step "${autoscale_saturation_boost_step}"
)

proxy_cmd=(
  "${probe_script}"
  "${common_args[@]}"
  --out "${proxy_out_path}"
)
if [ -n "${cluster_relay_workers}" ]; then
  proxy_cmd+=(--cluster-relay-workers "${cluster_relay_workers}")
fi
if [ -n "${cluster_relay_queue}" ]; then
  proxy_cmd+=(--cluster-relay-queue "${cluster_relay_queue}")
fi
if [ -n "${cluster_accept_workers}" ]; then
  proxy_cmd+=(--cluster-accept-workers "${cluster_accept_workers}")
fi
if [ -n "${cluster_relay_accept_batch_max}" ]; then
  proxy_cmd+=(--cluster-relay-accept-batch-max "${cluster_relay_accept_batch_max}")
fi
if [ -n "${cluster_relay_pump_batch_max}" ]; then
  proxy_cmd+=(--cluster-relay-pump-batch-max "${cluster_relay_pump_batch_max}")
fi

fixed_cmd=(
  "${probe_script}"
  "${common_args[@]}"
  --fixed-reuse-port-mode
  --out "${fixed_out_path}"
)

if [ "${dry_run}" = "true" ]; then
  proxy_cmd+=(--dry-run)
  fixed_cmd+=(--dry-run)
  "${proxy_cmd[@]}"
  "${fixed_cmd[@]}"
  exit 0
fi

if [ "${skip_build}" = "true" ]; then
  proxy_cmd+=(--skip-build)
fi
"${proxy_cmd[@]}"

fixed_cmd+=(--skip-build)
"${fixed_cmd[@]}"

if ! command -v jq >/dev/null 2>&1; then
  echo "required command not found: jq" >&2
  exit 127
fi

jq -n \
  --arg proxyPath "${proxy_out_path}" \
  --arg fixedPath "${fixed_out_path}" \
  --argjson proxy "$(cat "${proxy_out_path}")" \
  --argjson fixed "$(cat "${fixed_out_path}")" \
  '
  def p99_to_ms($value):
    if ($value | type) != "string" then null
    elif ($value | test("^[0-9]+(\\.[0-9]+)?ms$")) then ($value | sub("ms$"; "") | tonumber)
    elif ($value | test("^[0-9]+(\\.[0-9]+)?us$")) then (($value | sub("us$"; "") | tonumber) / 1000)
    elif ($value | test("^[0-9]+(\\.[0-9]+)?s$")) then (($value | sub("s$"; "") | tonumber) * 1000)
    else null
    end;
  def recommended_mode:
    if .fixed.pass == true and .proxy.pass == false then "fixed-reuse-port"
    elif .proxy.pass == true and .fixed.pass == false then "proxy-relay"
    elif .fixed.pass == true and .proxy.pass == true then
      if (.fixed.observed.requestsPerSec // 0) > (.proxy.observed.requestsPerSec // 0) then
        "fixed-reuse-port"
      elif (.fixed.observed.requestsPerSec // 0) < (.proxy.observed.requestsPerSec // 0) then
        "proxy-relay"
      else
        if (p99_to_ms(.fixed.observed.p99 // "") // 1000000000) < (p99_to_ms(.proxy.observed.p99 // "") // 1000000000) then
          "fixed-reuse-port"
        else
          "proxy-relay"
        end
      end
    else
      if (.fixed.observed.requestsPerSec // 0) >= (.proxy.observed.requestsPerSec // 0) then
        "fixed-reuse-port"
      else
        "proxy-relay"
      end
    end;
  {
    version: "0.1",
    run: {
      projectPath: ($proxy.projectPath // $fixed.projectPath // ""),
      requestPath: ($proxy.requestPath // $fixed.requestPath // ""),
      requestHeader: ($proxy.requestHeader // $fixed.requestHeader // ""),
      duration: ($proxy.run.duration // $fixed.run.duration // ""),
      threads: ($proxy.run.threads // $fixed.run.threads // 0),
      connections: ($proxy.run.connections // $fixed.run.connections // 0),
      targetRequests: ($proxy.run.targetRequests // $fixed.run.targetRequests // 0),
      proxySummaryPath: $proxyPath,
      fixedSummaryPath: $fixedPath
    },
    proxy: $proxy,
    fixed: $fixed,
    comparison: {
      recommendedMode: recommended_mode,
      requestsPerSecDelta: (($fixed.observed.requestsPerSec // 0) - ($proxy.observed.requestsPerSec // 0)),
      requestsPerSecGainPctVsProxy: (
        if ($proxy.observed.requestsPerSec // 0) > 0 then
          (((($fixed.observed.requestsPerSec // 0) - ($proxy.observed.requestsPerSec // 0)) / ($proxy.observed.requestsPerSec // 0)) * 100)
        else
          null
        end
      ),
      p99ProxyMs: p99_to_ms($proxy.observed.p99 // ""),
      p99FixedMs: p99_to_ms($fixed.observed.p99 // ""),
      peakRssDeltaKb: (($fixed.observed.peakRssKb // 0) - ($proxy.observed.peakRssKb // 0)),
      relayDispatchShortCircuitTotalProxy: ($proxy.run.clusterRelayDispatchSaturationShortCircuitTotal // null),
      relayDispatchShortCircuitTotalFixed: ($fixed.run.clusterRelayDispatchSaturationShortCircuitTotal // null),
      relayDispatchShortCircuitPerSecProxy: ($proxy.run.clusterRelayDispatchSaturationShortCircuitPerSec // null),
      relayDispatchShortCircuitPerSecFixed: ($fixed.run.clusterRelayDispatchSaturationShortCircuitPerSec // null),
      relayDispatchShortCircuitTotalDelta: (
        if ($proxy.run.clusterRelayDispatchSaturationShortCircuitTotal // null) == null
           or ($fixed.run.clusterRelayDispatchSaturationShortCircuitTotal // null) == null
        then null
        else (($fixed.run.clusterRelayDispatchSaturationShortCircuitTotal // 0) - ($proxy.run.clusterRelayDispatchSaturationShortCircuitTotal // 0))
        end
      )
    }
  }
  ' > "${out_path}"

echo "wrote ${out_path}"
