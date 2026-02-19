#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs the full LASM saturation boost tuning bundle:
  1) matrix/analysis (and optional recommended-step verify probe)
  2) markdown summary rendering

Options:
  --boost-steps <csv>                              Boost-step values (default: 2,4,6)
  --project-path <path>                            Project path passed to sec4 run (default: examples/lasm-alpha-full)
  --request-path <path>                            Probe HTTP path (default: /health)
  --request-header <value>                         Header passed to readiness + wrk (default: Authorization: Bearer token123)
  --duration <duration>                            wrk duration (default: 40s)
  --threads <n>                                    wrk threads (default: 8)
  --connections <n>                                wrk connections (default: 256)
  --target-requests <n>                            Minimum total requests required to pass each probe (default: 1000000)
  --port <n>                                       Service port (default: 18096)
  --instances <n>                                  LASM min instances (default: 4)
  --autoscale-max-instances <n>                    LASM max instances (default: 8)
  --autoscale-target-connections <n>               LASM autoscale target per instance (default: 256)
  --autoscale-check-ms <n>                         LASM autoscale check interval (default: 1000)
  --autoscale-scale-up-cooldown-ms <n>             LASM scale-up cooldown (default: 250)
  --autoscale-scale-down-cooldown-ms <n>           LASM scale-down cooldown (default: 2000)
  --autoscale-scale-up-step <n>                    LASM max scale-up workers per autoscale check (default: 2)
  --autoscale-scale-down-step <n>                  LASM max scale-down workers per autoscale check (default: 1)
  --cluster-relay-workers <n>                      Optional relay worker override
  --cluster-relay-queue <n>                        Optional relay queue override
  --skip-verify                                    Skip recommended-step follow-up probe
  --matrix-out <path>                              Matrix summary output path (default: results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json)
  --analysis-out <path>                            Analysis output path (default: results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json)
  --verify-out <path>                              Recommended-step verify output path (default: results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json)
  --summary-out <path>                             Markdown summary output path (default: results/summaries/sec4-lasm-cluster-saturation-boost-summary.md)
  --skip-build                                     Skip sec4 binary rebuild for probes
  --dry-run                                        Print command plan only
  -h, --help                                       Show this help
USAGE
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
cluster_relay_workers="${LASM_CAPACITY_CLUSTER_RELAY_WORKERS:-}"
cluster_relay_queue="${LASM_CAPACITY_CLUSTER_RELAY_QUEUE:-}"
boost_steps_csv="${LASM_CAPACITY_SATURATION_BOOST_STEPS:-2,4,6}"
matrix_out_rel="${LASM_CAPACITY_SATURATION_MATRIX_OUT:-results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json}"
analysis_out_rel="${LASM_CAPACITY_SATURATION_ANALYSIS_OUT:-results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json}"
verify_out_rel="${LASM_CAPACITY_SATURATION_VERIFY_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json}"
summary_out_rel="${LASM_CAPACITY_SATURATION_SUMMARY_OUT:-results/summaries/sec4-lasm-cluster-saturation-boost-summary.md}"
verify_recommended="true"
skip_build="false"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --boost-steps)
      boost_steps_csv="${2:-}"
      shift 2
      ;;
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
    --cluster-relay-workers)
      cluster_relay_workers="${2:-}"
      shift 2
      ;;
    --cluster-relay-queue)
      cluster_relay_queue="${2:-}"
      shift 2
      ;;
    --skip-verify)
      verify_recommended="false"
      shift
      ;;
    --matrix-out)
      matrix_out_rel="${2:-}"
      shift 2
      ;;
    --analysis-out)
      analysis_out_rel="${2:-}"
      shift 2
      ;;
    --verify-out)
      verify_out_rel="${2:-}"
      shift 2
      ;;
    --summary-out)
      summary_out_rel="${2:-}"
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

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
matrix_script="${root_dir}/scripts/run_lasm_cluster_saturation_boost_matrix.sh"
summary_script="${root_dir}/scripts/render_lasm_cluster_saturation_boost_summary.sh"

if [ ! -x "${matrix_script}" ]; then
  echo "missing executable matrix script: ${matrix_script}" >&2
  exit 1
fi
if [ ! -x "${summary_script}" ]; then
  echo "missing executable summary script: ${summary_script}" >&2
  exit 1
fi

resolve_path() {
  local value="$1"
  if [[ "${value}" = /* ]]; then
    printf "%s" "${value}"
  elif [[ "${value}" == benchmark-suite/* ]]; then
    printf "%s" "${repo_root}/${value}"
  else
    printf "%s" "${root_dir}/${value}"
  fi
}

matrix_out_path="$(resolve_path "${matrix_out_rel}")"
analysis_out_path="$(resolve_path "${analysis_out_rel}")"
verify_out_path="$(resolve_path "${verify_out_rel}")"
summary_out_path="$(resolve_path "${summary_out_rel}")"

mkdir -p "$(dirname "${matrix_out_path}")" "$(dirname "${analysis_out_path}")" "$(dirname "${verify_out_path}")" "$(dirname "${summary_out_path}")"

cat <<PLAN
sec4 LASM saturation boost bundle plan:
  projectPath=${project_path}
  requestPath=${request_path}
  duration=${duration}
  threads=${threads}
  connections=${connections}
  targetRequests=${target_requests}
  boostSteps=${boost_steps_csv}
  verifyRecommended=${verify_recommended}
  matrixOut=${matrix_out_path}
  analysisOut=${analysis_out_path}
  verifyOut=${verify_out_path}
  summaryOut=${summary_out_path}
  skipBuild=${skip_build}
  dryRun=${dry_run}
PLAN

matrix_cmd=(
  "${matrix_script}"
  --boost-steps "${boost_steps_csv}"
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
  --out "${matrix_out_path}"
  --analysis-out "${analysis_out_path}"
)

if [ "${verify_recommended}" = "true" ]; then
  matrix_cmd+=(--verify-recommended --verify-out "${verify_out_path}")
fi
if [ -n "${cluster_relay_workers}" ]; then
  matrix_cmd+=(--cluster-relay-workers "${cluster_relay_workers}")
fi
if [ -n "${cluster_relay_queue}" ]; then
  matrix_cmd+=(--cluster-relay-queue "${cluster_relay_queue}")
fi
if [ "${skip_build}" = "true" ]; then
  matrix_cmd+=(--skip-build)
fi
if [ "${dry_run}" = "true" ]; then
  matrix_cmd+=(--dry-run)
fi

if [ "${dry_run}" = "true" ]; then
  "${matrix_cmd[@]}"
  if [ "${verify_recommended}" = "true" ]; then
    echo "summaryCmd=${summary_script} ${matrix_out_path} ${analysis_out_path} ${summary_out_path} ${verify_out_path}"
  else
    echo "summaryCmd=${summary_script} ${matrix_out_path} ${analysis_out_path} ${summary_out_path}"
  fi
  exit 0
fi

"${matrix_cmd[@]}"

if [ "${verify_recommended}" = "true" ]; then
  "${summary_script}" "${matrix_out_path}" "${analysis_out_path}" "${summary_out_path}" "${verify_out_path}"
else
  "${summary_script}" "${matrix_out_path}" "${analysis_out_path}" "${summary_out_path}"
fi

echo "wroteBundleSummary=${summary_out_path}"
