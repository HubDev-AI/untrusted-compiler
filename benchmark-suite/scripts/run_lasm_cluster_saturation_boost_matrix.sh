#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs a matrix of LASM cluster capacity probes across multiple
--autoscale-saturation-boost-step values.

Options:
  --boost-steps <csv>                              Boost-step values (default: 2,4,6)
  --profile <ping|db-hot-write|db-hot-write-tx>   Probe profile (default: ping)
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
  --fixed-reuse-port-mode                          Run probes in fixed reuse-port cluster mode
  --cluster-relay-workers <n>                      Optional relay worker override
  --cluster-relay-queue <n>                        Optional relay queue override
  --cluster-accept-workers <n>                     Optional relay accept-worker override
  --cluster-relay-accept-batch-max <n>             Optional relay accept batch max override
  --cluster-relay-pump-batch-max <n>               Optional relay pump batch max override
  --db-base <path>                                 Optional LASM db base directory
  --db-adapter <records-log|sqlite|postgres>       Optional LASM db adapter override
  --db-postgres-dsn-file <path>                    Optional LASM postgres DSN file path
  --db-postgres-tls-mode <auto|disable|require>    Optional LASM postgres TLS mode override
  --db-max-tx-handles <n>                          Optional LASM db max tx handles override
  --db-records-max <n>                             Optional LASM db records max override
  --db-postgres-statement-cache-max <n>            Optional LASM postgres statement cache max override
  --db-postgres-placeholder-cache-max <n>          Optional LASM postgres placeholder cache max override
  --db-postgres-statement-timeout-ms <n>           Optional LASM postgres statement timeout override
  --db-postgres-lock-timeout-ms <n>                Optional LASM postgres lock timeout override
  --db-postgres-connect-timeout-ms <n>             Optional LASM postgres connect timeout override
  --db-sqlite-busy-timeout-ms <n>                  Optional LASM sqlite busy timeout override
  --db-sqlite-journal-mode <mode>                  Optional LASM sqlite journal mode override
  --db-sqlite-synchronous <mode>                   Optional LASM sqlite synchronous override
  --db-postgres-retryable-conflict-retry-max <n>   Optional LASM postgres retryable conflict retry max
  --db-sqlite-lock-retry-max <n>                   Optional LASM sqlite lock retry max
  --db-sqlite-lock-retry-delay-ms <n>              Optional LASM sqlite lock retry delay override
  --build-profile <debug|release>                  sec4 build profile forwarded to probe runs (default: release)
  --samples <n>                                    Number of wrk samples per probe run (default: 1)
  --wrk-processes <n>                              Number of parallel wrk processes per probe run (default: 1)
  --out <path>                                     Matrix summary output path (default: results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json)
  --analysis-out <path>                            Analysis output path (default: results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json)
  --skip-analysis                                  Skip post-run matrix analysis/recommendation output
  --verify-recommended                             Run one additional capacity probe using the recommended boost step from analysis
  --verify-out <path>                              Recommended-step verification output path (default: results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json)
  --skip-build                                     Skip sec4 binary rebuild for all probes
  --dry-run                                        Print matrix probe plan only
  -h, --help                                       Show this help
USAGE
}

profile="${LASM_CAPACITY_PROFILE:-ping}"
project_path="${LASM_CAPACITY_PROJECT_PATH:-}"
project_path_explicit="false"
if [ -n "${LASM_CAPACITY_PROJECT_PATH:-}" ]; then
  project_path_explicit="true"
fi
request_path="${LASM_CAPACITY_REQUEST_PATH:-}"
request_path_explicit="false"
if [ -n "${LASM_CAPACITY_REQUEST_PATH:-}" ]; then
  request_path_explicit="true"
fi
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
fixed_reuse_port_mode="${LASM_CAPACITY_FIXED_REUSE_PORT_MODE:-false}"
cluster_relay_workers="${LASM_CAPACITY_CLUSTER_RELAY_WORKERS:-}"
cluster_relay_queue="${LASM_CAPACITY_CLUSTER_RELAY_QUEUE:-}"
cluster_accept_workers="${LASM_CAPACITY_CLUSTER_ACCEPT_WORKERS:-}"
cluster_relay_accept_batch_max="${LASM_CAPACITY_CLUSTER_RELAY_ACCEPT_BATCH_MAX:-}"
cluster_relay_pump_batch_max="${LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX:-}"
db_base="${LASM_CAPACITY_DB_BASE:-}"
db_adapter="${LASM_CAPACITY_DB_ADAPTER:-}"
db_postgres_dsn_file="${LASM_CAPACITY_DB_POSTGRES_DSN_FILE:-}"
db_postgres_tls_mode="${LASM_CAPACITY_DB_POSTGRES_TLS_MODE:-}"
db_max_tx_handles="${LASM_CAPACITY_DB_MAX_TX_HANDLES:-}"
db_records_max="${LASM_CAPACITY_DB_RECORDS_MAX:-}"
db_postgres_statement_cache_max="${LASM_CAPACITY_DB_POSTGRES_STATEMENT_CACHE_MAX:-}"
db_postgres_placeholder_cache_max="${LASM_CAPACITY_DB_POSTGRES_PLACEHOLDER_CACHE_MAX:-}"
db_postgres_statement_timeout_ms="${LASM_CAPACITY_DB_POSTGRES_STATEMENT_TIMEOUT_MS:-}"
db_postgres_lock_timeout_ms="${LASM_CAPACITY_DB_POSTGRES_LOCK_TIMEOUT_MS:-}"
db_postgres_connect_timeout_ms="${LASM_CAPACITY_DB_POSTGRES_CONNECT_TIMEOUT_MS:-}"
db_sqlite_busy_timeout_ms="${LASM_CAPACITY_DB_SQLITE_BUSY_TIMEOUT_MS:-}"
db_sqlite_journal_mode="${LASM_CAPACITY_DB_SQLITE_JOURNAL_MODE:-}"
db_sqlite_synchronous="${LASM_CAPACITY_DB_SQLITE_SYNCHRONOUS:-}"
db_postgres_retryable_conflict_retry_max="${LASM_CAPACITY_DB_POSTGRES_RETRYABLE_CONFLICT_RETRY_MAX:-}"
db_sqlite_lock_retry_max="${LASM_CAPACITY_DB_SQLITE_LOCK_RETRY_MAX:-}"
db_sqlite_lock_retry_delay_ms="${LASM_CAPACITY_DB_SQLITE_LOCK_RETRY_DELAY_MS:-}"
build_profile="${LASM_CAPACITY_BUILD_PROFILE:-release}"
samples="${LASM_CAPACITY_SAMPLES:-1}"
wrk_processes="${LASM_CAPACITY_WRK_PROCESSES:-1}"
boost_steps_csv="${LASM_CAPACITY_SATURATION_BOOST_STEPS:-2,4,6}"
out_rel="${LASM_CAPACITY_SATURATION_MATRIX_OUT:-results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json}"
analysis_out_rel="${LASM_CAPACITY_SATURATION_ANALYSIS_OUT:-results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json}"
skip_analysis="false"
verify_recommended="false"
verify_out_rel="${LASM_CAPACITY_SATURATION_VERIFY_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json}"
skip_build="false"
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --boost-steps)
      boost_steps_csv="${2:-}"
      shift 2
      ;;
    --boost-steps=*)
      boost_steps_csv="${1#--boost-steps=}"
      shift
      ;;
    --profile)
      profile="${2:-}"
      shift 2
      ;;
    --project-path)
      project_path="${2:-}"
      project_path_explicit="true"
      shift 2
      ;;
    --request-path)
      request_path="${2:-}"
      request_path_explicit="true"
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
    --fixed-reuse-port-mode)
      fixed_reuse_port_mode="true"
      shift
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
    --db-base)
      db_base="${2:-}"
      shift 2
      ;;
    --db-adapter)
      db_adapter="${2:-}"
      shift 2
      ;;
    --db-postgres-dsn-file)
      db_postgres_dsn_file="${2:-}"
      shift 2
      ;;
    --db-postgres-tls-mode)
      db_postgres_tls_mode="${2:-}"
      shift 2
      ;;
    --db-max-tx-handles)
      db_max_tx_handles="${2:-}"
      shift 2
      ;;
    --db-records-max)
      db_records_max="${2:-}"
      shift 2
      ;;
    --db-postgres-statement-cache-max)
      db_postgres_statement_cache_max="${2:-}"
      shift 2
      ;;
    --db-postgres-placeholder-cache-max)
      db_postgres_placeholder_cache_max="${2:-}"
      shift 2
      ;;
    --db-postgres-statement-timeout-ms)
      db_postgres_statement_timeout_ms="${2:-}"
      shift 2
      ;;
    --db-postgres-lock-timeout-ms)
      db_postgres_lock_timeout_ms="${2:-}"
      shift 2
      ;;
    --db-postgres-connect-timeout-ms)
      db_postgres_connect_timeout_ms="${2:-}"
      shift 2
      ;;
    --db-sqlite-busy-timeout-ms)
      db_sqlite_busy_timeout_ms="${2:-}"
      shift 2
      ;;
    --db-sqlite-journal-mode)
      db_sqlite_journal_mode="${2:-}"
      shift 2
      ;;
    --db-sqlite-synchronous)
      db_sqlite_synchronous="${2:-}"
      shift 2
      ;;
    --db-postgres-retryable-conflict-retry-max)
      db_postgres_retryable_conflict_retry_max="${2:-}"
      shift 2
      ;;
    --db-sqlite-lock-retry-max)
      db_sqlite_lock_retry_max="${2:-}"
      shift 2
      ;;
    --db-sqlite-lock-retry-delay-ms)
      db_sqlite_lock_retry_delay_ms="${2:-}"
      shift 2
      ;;
    --build-profile)
      build_profile="${2:-}"
      shift 2
      ;;
    --samples)
      samples="${2:-}"
      shift 2
      ;;
    --wrk-processes)
      wrk_processes="${2:-}"
      shift 2
      ;;
    --out)
      out_rel="${2:-}"
      shift 2
      ;;
    --analysis-out)
      analysis_out_rel="${2:-}"
      shift 2
      ;;
    --skip-analysis)
      skip_analysis="true"
      shift
      ;;
    --verify-recommended)
      verify_recommended="true"
      shift
      ;;
    --verify-out)
      verify_out_rel="${2:-}"
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

if [ -z "${boost_steps_csv}" ]; then
  echo "boost-steps must not be empty" >&2
  exit 2
fi

IFS=',' read -r -a raw_boost_steps <<< "${boost_steps_csv}"
boost_steps=()
for raw in "${raw_boost_steps[@]}"; do
  step="${raw// /}"
  if [ -z "${step}" ]; then
    continue
  fi
  if ! [[ "${step}" =~ ^[0-9]+$ ]]; then
    echo "boost-steps must contain positive integers, got: ${step}" >&2
    exit 2
  fi
  if [ "${step}" -le 0 ]; then
    echo "boost-steps must be >= 1, got: ${step}" >&2
    exit 2
  fi
  boost_steps+=("${step}")
done

if [ "${#boost_steps[@]}" -eq 0 ]; then
  echo "boost-steps must contain at least one positive integer" >&2
  exit 2
fi
if [ "${verify_recommended}" = "true" ] && [ "${skip_analysis}" = "true" ]; then
  echo "verify-recommended requires analysis; remove --skip-analysis" >&2
  exit 2
fi
if [ "${fixed_reuse_port_mode}" != "true" ] && [ "${fixed_reuse_port_mode}" != "false" ]; then
  echo "fixed-reuse-port-mode must be true or false, got: ${fixed_reuse_port_mode}" >&2
  exit 2
fi
case "${build_profile}" in
  debug|release) ;;
  *)
    echo "build-profile must be one of: debug, release (got: ${build_profile})" >&2
    exit 2
    ;;
esac
if ! [[ "${samples}" =~ ^[0-9]+$ ]]; then
  echo "samples must be an integer >= 1, got: ${samples}" >&2
  exit 2
fi
if [ "${samples}" -lt 1 ]; then
  echo "samples must be >= 1, got: ${samples}" >&2
  exit 2
fi
if ! [[ "${wrk_processes}" =~ ^[0-9]+$ ]]; then
  echo "wrk-processes must be an integer >= 1, got: ${wrk_processes}" >&2
  exit 2
fi
if [ "${wrk_processes}" -lt 1 ]; then
  echo "wrk-processes must be >= 1, got: ${wrk_processes}" >&2
  exit 2
fi
case "${profile}" in
  ping|db-hot-write|db-hot-write-tx) ;;
  *)
    echo "profile must be one of: ping, db-hot-write, db-hot-write-tx (got: ${profile})" >&2
    exit 2
    ;;
esac

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
if [ "${project_path_explicit}" != "true" ]; then
  case "${profile}" in
    ping)
      project_path="examples/lasm-alpha-full"
      ;;
    db-hot-write|db-hot-write-tx)
      project_path="benchmark-suite/services/sec4-lasm"
      ;;
  esac
fi
if [ "${request_path_explicit}" != "true" ]; then
  case "${profile}" in
    ping)
      request_path="/health"
      ;;
    db-hot-write)
      request_path="/db/hot-write"
      ;;
    db-hot-write-tx)
      request_path="/db/hot-write-tx"
      ;;
  esac
fi
probe_script="${root_dir}/scripts/run_lasm_cluster_capacity_probe.sh"
analyze_script="${root_dir}/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh"

if [ ! -x "${probe_script}" ]; then
  echo "missing executable probe script: ${probe_script}" >&2
  exit 1
fi
if [ "${skip_analysis}" != "true" ] && [ ! -x "${analyze_script}" ]; then
  echo "missing executable analysis script: ${analyze_script}" >&2
  exit 1
fi

if [[ "${out_rel}" = /* ]]; then
  out_path="${out_rel}"
elif [[ "${out_rel}" == benchmark-suite/* ]]; then
  out_path="${repo_root}/${out_rel}"
else
  out_path="${root_dir}/${out_rel}"
fi
out_dir="$(dirname "${out_path}")"
mkdir -p "${out_dir}"

if [[ "${analysis_out_rel}" = /* ]]; then
  analysis_out_path="${analysis_out_rel}"
elif [[ "${analysis_out_rel}" == benchmark-suite/* ]]; then
  analysis_out_path="${repo_root}/${analysis_out_rel}"
else
  analysis_out_path="${root_dir}/${analysis_out_rel}"
fi
analysis_out_dir="$(dirname "${analysis_out_path}")"
mkdir -p "${analysis_out_dir}"

if [[ "${verify_out_rel}" = /* ]]; then
  verify_out_path="${verify_out_rel}"
elif [[ "${verify_out_rel}" == benchmark-suite/* ]]; then
  verify_out_path="${repo_root}/${verify_out_rel}"
else
  verify_out_path="${root_dir}/${verify_out_rel}"
fi
verify_out_dir="$(dirname "${verify_out_path}")"
mkdir -p "${verify_out_dir}"

boost_steps_joined="$(IFS=,; echo "${boost_steps[*]}")"

cat <<PLAN
sec4 LASM saturation boost matrix plan:
  profile=${profile}
  boostSteps=${boost_steps_joined}
  projectPath=${project_path}
  requestPath=${request_path}
  duration=${duration}
  threads=${threads}
  connections=${connections}
  targetRequests=${target_requests}
  fixedReusePortMode=${fixed_reuse_port_mode}
  clusterRelayWorkers=${cluster_relay_workers:-auto}
  clusterRelayQueue=${cluster_relay_queue:-auto}
  clusterAcceptWorkers=${cluster_accept_workers:-auto}
  clusterRelayAcceptBatchMax=${cluster_relay_accept_batch_max:-auto}
  clusterRelayPumpBatchMax=${cluster_relay_pump_batch_max:-auto}
  dbBase=${db_base:-auto}
  dbAdapter=${db_adapter:-auto}
  dbPostgresDsnFile=${db_postgres_dsn_file:-auto}
  dbPostgresTlsMode=${db_postgres_tls_mode:-auto}
  dbMaxTxHandles=${db_max_tx_handles:-auto}
  dbRecordsMax=${db_records_max:-auto}
  dbPostgresStatementCacheMax=${db_postgres_statement_cache_max:-auto}
  dbPostgresPlaceholderCacheMax=${db_postgres_placeholder_cache_max:-auto}
  dbPostgresStatementTimeoutMs=${db_postgres_statement_timeout_ms:-auto}
  dbPostgresLockTimeoutMs=${db_postgres_lock_timeout_ms:-auto}
  dbPostgresConnectTimeoutMs=${db_postgres_connect_timeout_ms:-auto}
  dbSqliteBusyTimeoutMs=${db_sqlite_busy_timeout_ms:-auto}
  dbSqliteJournalMode=${db_sqlite_journal_mode:-auto}
  dbSqliteSynchronous=${db_sqlite_synchronous:-auto}
  dbPostgresRetryableConflictRetryMax=${db_postgres_retryable_conflict_retry_max:-auto}
  dbSqliteLockRetryMax=${db_sqlite_lock_retry_max:-auto}
  dbSqliteLockRetryDelayMs=${db_sqlite_lock_retry_delay_ms:-auto}
  buildProfile=${build_profile}
  samples=${samples}
  wrkProcesses=${wrk_processes}
  out=${out_path}
  analysisOut=${analysis_out_path}
  skipAnalysis=${skip_analysis}
  verifyRecommended=${verify_recommended}
  verifyOut=${verify_out_path}
PLAN

runs_json='[]'
run_index=0

for step in "${boost_steps[@]}"; do
  step_out="${out_dir}/sec4-lasm-cluster-capacity-probe-sat-boost-${step}.json"
  cmd=(
    "${probe_script}"
    --profile "${profile}"
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
    --autoscale-saturation-boost-step "${step}"
    --build-profile "${build_profile}"
    --samples "${samples}"
    --wrk-processes "${wrk_processes}"
    --out "${step_out}"
  )
  if [ "${fixed_reuse_port_mode}" = "true" ]; then
    cmd+=(--fixed-reuse-port-mode)
  fi

  if [ -n "${cluster_relay_workers}" ]; then
    cmd+=(--cluster-relay-workers "${cluster_relay_workers}")
  fi
  if [ -n "${cluster_relay_queue}" ]; then
    cmd+=(--cluster-relay-queue "${cluster_relay_queue}")
  fi
  if [ -n "${cluster_accept_workers}" ]; then
    cmd+=(--cluster-accept-workers "${cluster_accept_workers}")
  fi
  if [ -n "${cluster_relay_accept_batch_max}" ]; then
    cmd+=(--cluster-relay-accept-batch-max "${cluster_relay_accept_batch_max}")
  fi
  if [ -n "${cluster_relay_pump_batch_max}" ]; then
    cmd+=(--cluster-relay-pump-batch-max "${cluster_relay_pump_batch_max}")
  fi
  if [ -n "${db_base}" ]; then
    cmd+=(--db-base "${db_base}")
  fi
  if [ -n "${db_adapter}" ]; then
    cmd+=(--db-adapter "${db_adapter}")
  fi
  if [ -n "${db_postgres_dsn_file}" ]; then
    cmd+=(--db-postgres-dsn-file "${db_postgres_dsn_file}")
  fi
  if [ -n "${db_postgres_tls_mode}" ]; then
    cmd+=(--db-postgres-tls-mode "${db_postgres_tls_mode}")
  fi
  if [ -n "${db_max_tx_handles}" ]; then
    cmd+=(--db-max-tx-handles "${db_max_tx_handles}")
  fi
  if [ -n "${db_records_max}" ]; then
    cmd+=(--db-records-max "${db_records_max}")
  fi
  if [ -n "${db_postgres_statement_cache_max}" ]; then
    cmd+=(--db-postgres-statement-cache-max "${db_postgres_statement_cache_max}")
  fi
  if [ -n "${db_postgres_placeholder_cache_max}" ]; then
    cmd+=(--db-postgres-placeholder-cache-max "${db_postgres_placeholder_cache_max}")
  fi
  if [ -n "${db_postgres_statement_timeout_ms}" ]; then
    cmd+=(--db-postgres-statement-timeout-ms "${db_postgres_statement_timeout_ms}")
  fi
  if [ -n "${db_postgres_lock_timeout_ms}" ]; then
    cmd+=(--db-postgres-lock-timeout-ms "${db_postgres_lock_timeout_ms}")
  fi
  if [ -n "${db_postgres_connect_timeout_ms}" ]; then
    cmd+=(--db-postgres-connect-timeout-ms "${db_postgres_connect_timeout_ms}")
  fi
  if [ -n "${db_sqlite_busy_timeout_ms}" ]; then
    cmd+=(--db-sqlite-busy-timeout-ms "${db_sqlite_busy_timeout_ms}")
  fi
  if [ -n "${db_sqlite_journal_mode}" ]; then
    cmd+=(--db-sqlite-journal-mode "${db_sqlite_journal_mode}")
  fi
  if [ -n "${db_sqlite_synchronous}" ]; then
    cmd+=(--db-sqlite-synchronous "${db_sqlite_synchronous}")
  fi
  if [ -n "${db_postgres_retryable_conflict_retry_max}" ]; then
    cmd+=(--db-postgres-retryable-conflict-retry-max "${db_postgres_retryable_conflict_retry_max}")
  fi
  if [ -n "${db_sqlite_lock_retry_max}" ]; then
    cmd+=(--db-sqlite-lock-retry-max "${db_sqlite_lock_retry_max}")
  fi
  if [ -n "${db_sqlite_lock_retry_delay_ms}" ]; then
    cmd+=(--db-sqlite-lock-retry-delay-ms "${db_sqlite_lock_retry_delay_ms}")
  fi

  if [ "${dry_run}" = "true" ]; then
    cmd+=(--dry-run)
  elif [ "${skip_build}" = "true" ] || [ "${run_index}" -gt 0 ]; then
    cmd+=(--skip-build)
  fi

  echo "=== saturationBoostStep=${step} ==="
  "${cmd[@]}"

  if [ "${dry_run}" != "true" ]; then
    run_item="$(jq -n \
      --argjson saturationBoostStep "${step}" \
      --arg summaryFile "${step_out}" \
      --argjson pass "$(jq '.pass' "${step_out}")" \
      --argjson requests "$(jq '.observed.requests' "${step_out}")" \
      --argjson requestsPerSec "$(jq '.observed.requestsPerSec' "${step_out}")" \
      --argjson peakRssKb "$(jq '.observed.peakRssKb' "${step_out}")" \
      --arg p99 "$(jq -r '.observed.p99 // ""' "${step_out}")" \
      --argjson clusterRelayWorkersResolved "$(jq '.run.clusterRelayWorkersResolved // null' "${step_out}")" \
      --argjson clusterAcceptWorkersResolved "$(jq '.run.clusterAcceptWorkersResolved // null' "${step_out}")" \
      --argjson clusterRelayAcceptBatchMaxResolved "$(jq '.run.clusterRelayAcceptBatchMaxResolved // null' "${step_out}")" \
      --argjson clusterRelayPumpBatchMaxResolved "$(jq '.run.clusterRelayPumpBatchMaxResolved // null' "${step_out}")" \
      --argjson clusterRelayQueueCapacityResolved "$(jq '.run.clusterRelayQueueCapacityResolved // null' "${step_out}")" \
      --argjson clusterRelayQueueShardCapacityResolved "$(jq '.run.clusterRelayQueueShardCapacityResolved // null' "${step_out}")" \
      --argjson clusterRelayDispatchSaturationShortCircuitTotal "$(jq '.run.clusterRelayDispatchSaturationShortCircuitTotal // null' "${step_out}")" \
      --argjson clusterRelayDispatchSaturationShortCircuitPerSec "$(jq '.run.clusterRelayDispatchSaturationShortCircuitPerSec // null' "${step_out}")" \
      --argjson clusterRelayLiveSenderCountResolved "$(jq '.run.clusterRelayLiveSenderCountResolved // null' "${step_out}")" \
      --arg clusterDbAdapterResolved "$(jq -r '.run.clusterDbAdapterResolved // ""' "${step_out}")" \
      --arg clusterDbPostgresTlsModeResolved "$(jq -r '.run.clusterDbPostgresTlsModeResolved // ""' "${step_out}")" \
      --argjson clusterDbMaxTxHandlesResolved "$(jq '.run.clusterDbMaxTxHandlesResolved // null' "${step_out}")" \
      --argjson clusterDbRecordsMaxResolved "$(jq '.run.clusterDbRecordsMaxResolved // null' "${step_out}")" \
      --argjson clusterDbPostgresStatementCacheMaxResolved "$(jq '.run.clusterDbPostgresStatementCacheMaxResolved // null' "${step_out}")" \
      --argjson clusterDbPostgresPlaceholderCacheMaxResolved "$(jq '.run.clusterDbPostgresPlaceholderCacheMaxResolved // null' "${step_out}")" \
      --argjson clusterDbPostgresStatementTimeoutMsResolved "$(jq '.run.clusterDbPostgresStatementTimeoutMsResolved // null' "${step_out}")" \
      --argjson clusterDbPostgresLockTimeoutMsResolved "$(jq '.run.clusterDbPostgresLockTimeoutMsResolved // null' "${step_out}")" \
      --argjson clusterDbPostgresConnectTimeoutMsResolved "$(jq '.run.clusterDbPostgresConnectTimeoutMsResolved // null' "${step_out}")" \
      --argjson clusterDbSqliteBusyTimeoutMsResolved "$(jq '.run.clusterDbSqliteBusyTimeoutMsResolved // null' "${step_out}")" \
      --arg clusterDbSqliteJournalModeResolved "$(jq -r '.run.clusterDbSqliteJournalModeResolved // ""' "${step_out}")" \
      --arg clusterDbSqliteSynchronousResolved "$(jq -r '.run.clusterDbSqliteSynchronousResolved // ""' "${step_out}")" \
      --argjson clusterDbPostgresRetryableConflictRetryMaxResolved "$(jq '.run.clusterDbPostgresRetryableConflictRetryMaxResolved // null' "${step_out}")" \
      --argjson clusterDbSqliteLockRetryMaxResolved "$(jq '.run.clusterDbSqliteLockRetryMaxResolved // null' "${step_out}")" \
      --argjson clusterDbSqliteLockRetryDelayMsResolved "$(jq '.run.clusterDbSqliteLockRetryDelayMsResolved // null' "${step_out}")" \
      --argjson wrkProcesses "$(jq '.run.wrkProcesses // null' "${step_out}")" \
      --argjson wrkTotalConnections "$(jq '.run.wrkTotalConnections // null' "${step_out}")" \
      '{
        saturationBoostStep: $saturationBoostStep,
        summaryFile: $summaryFile,
        pass: $pass,
        requests: $requests,
        requestsPerSec: $requestsPerSec,
        peakRssKb: $peakRssKb,
        p99: $p99,
        wrkProcesses: $wrkProcesses,
        wrkTotalConnections: $wrkTotalConnections,
        clusterRelayWorkersResolved: $clusterRelayWorkersResolved,
        clusterAcceptWorkersResolved: $clusterAcceptWorkersResolved,
        clusterRelayAcceptBatchMaxResolved: $clusterRelayAcceptBatchMaxResolved,
        clusterRelayPumpBatchMaxResolved: $clusterRelayPumpBatchMaxResolved,
        clusterRelayQueueCapacityResolved: $clusterRelayQueueCapacityResolved,
        clusterRelayQueueShardCapacityResolved: $clusterRelayQueueShardCapacityResolved,
        clusterRelayDispatchSaturationShortCircuitTotal: $clusterRelayDispatchSaturationShortCircuitTotal,
        clusterRelayDispatchSaturationShortCircuitPerSec: $clusterRelayDispatchSaturationShortCircuitPerSec,
        clusterRelayLiveSenderCountResolved: $clusterRelayLiveSenderCountResolved,
        clusterDbAdapterResolved: (if $clusterDbAdapterResolved == "" then null else $clusterDbAdapterResolved end),
        clusterDbPostgresTlsModeResolved: (if $clusterDbPostgresTlsModeResolved == "" then null else $clusterDbPostgresTlsModeResolved end),
        clusterDbMaxTxHandlesResolved: $clusterDbMaxTxHandlesResolved,
        clusterDbRecordsMaxResolved: $clusterDbRecordsMaxResolved,
        clusterDbPostgresStatementCacheMaxResolved: $clusterDbPostgresStatementCacheMaxResolved,
        clusterDbPostgresPlaceholderCacheMaxResolved: $clusterDbPostgresPlaceholderCacheMaxResolved,
        clusterDbPostgresStatementTimeoutMsResolved: $clusterDbPostgresStatementTimeoutMsResolved,
        clusterDbPostgresLockTimeoutMsResolved: $clusterDbPostgresLockTimeoutMsResolved,
        clusterDbPostgresConnectTimeoutMsResolved: $clusterDbPostgresConnectTimeoutMsResolved,
        clusterDbSqliteBusyTimeoutMsResolved: $clusterDbSqliteBusyTimeoutMsResolved,
        clusterDbSqliteJournalModeResolved: (if $clusterDbSqliteJournalModeResolved == "" then null else $clusterDbSqliteJournalModeResolved end),
        clusterDbSqliteSynchronousResolved: (if $clusterDbSqliteSynchronousResolved == "" then null else $clusterDbSqliteSynchronousResolved end),
        clusterDbPostgresRetryableConflictRetryMaxResolved: $clusterDbPostgresRetryableConflictRetryMaxResolved,
        clusterDbSqliteLockRetryMaxResolved: $clusterDbSqliteLockRetryMaxResolved,
        clusterDbSqliteLockRetryDelayMsResolved: $clusterDbSqliteLockRetryDelayMsResolved
      }'
    )"
    runs_json="$(jq --argjson item "${run_item}" '. + [$item]' <<<"${runs_json}")"
  fi

  run_index=$((run_index + 1))
done

if [ "${dry_run}" = "true" ]; then
  if [ "${skip_analysis}" != "true" ]; then
    echo "analysisCmd=${analyze_script} ${out_path} ${analysis_out_path}"
    if [ "${verify_recommended}" = "true" ]; then
      echo "verifyRecommendedAfterAnalysis=true"
      echo "verifyCmd=${probe_script} ... --autoscale-saturation-boost-step <recommended> --build-profile ${build_profile} --samples ${samples} --wrk-processes ${wrk_processes} --out ${verify_out_path} --skip-build"
    fi
  fi
  exit 0
fi
wrk_total_connections="$(awk -v c="${connections}" -v p="${wrk_processes}" 'BEGIN { printf "%.0f", (c + 0) * (p + 0) }')"

jq -n \
  --arg impl "sec4-lasm-cluster" \
  --arg profile "${profile}" \
  --arg projectPath "${project_path}" \
  --arg requestPath "${request_path}" \
  --arg requestHeader "${request_header}" \
  --arg duration "${duration}" \
  --arg buildProfile "${build_profile}" \
  --arg clusterRelayWorkers "${cluster_relay_workers:-auto}" \
  --arg clusterRelayQueue "${cluster_relay_queue:-auto}" \
  --arg clusterAcceptWorkers "${cluster_accept_workers:-auto}" \
  --arg clusterRelayAcceptBatchMax "${cluster_relay_accept_batch_max:-auto}" \
  --arg clusterRelayPumpBatchMax "${cluster_relay_pump_batch_max:-auto}" \
  --arg dbBase "${db_base}" \
  --arg dbAdapter "${db_adapter}" \
  --arg dbPostgresDsnFile "${db_postgres_dsn_file}" \
  --arg dbPostgresTlsMode "${db_postgres_tls_mode}" \
  --argjson dbMaxTxHandles "${db_max_tx_handles:-null}" \
  --argjson dbRecordsMax "${db_records_max:-null}" \
  --argjson dbPostgresStatementCacheMax "${db_postgres_statement_cache_max:-null}" \
  --argjson dbPostgresPlaceholderCacheMax "${db_postgres_placeholder_cache_max:-null}" \
  --argjson dbPostgresStatementTimeoutMs "${db_postgres_statement_timeout_ms:-null}" \
  --argjson dbPostgresLockTimeoutMs "${db_postgres_lock_timeout_ms:-null}" \
  --argjson dbPostgresConnectTimeoutMs "${db_postgres_connect_timeout_ms:-null}" \
  --argjson dbSqliteBusyTimeoutMs "${db_sqlite_busy_timeout_ms:-null}" \
  --arg dbSqliteJournalMode "${db_sqlite_journal_mode}" \
  --arg dbSqliteSynchronous "${db_sqlite_synchronous}" \
  --argjson dbPostgresRetryableConflictRetryMax "${db_postgres_retryable_conflict_retry_max:-null}" \
  --argjson dbSqliteLockRetryMax "${db_sqlite_lock_retry_max:-null}" \
  --argjson dbSqliteLockRetryDelayMs "${db_sqlite_lock_retry_delay_ms:-null}" \
  --argjson samples "${samples}" \
  --argjson wrkProcesses "${wrk_processes}" \
  --argjson wrkTotalConnections "${wrk_total_connections}" \
  --argjson fixedReusePortMode "${fixed_reuse_port_mode}" \
  --argjson threads "${threads}" \
  --argjson connections "${connections}" \
  --argjson targetRequests "${target_requests}" \
  --argjson boostSteps "$(jq -n --arg csv "${boost_steps_joined}" '$csv | split(",") | map(tonumber)')" \
  --argjson runs "${runs_json}" \
  '{
    impl: $impl,
    run: {
      profile: $profile,
      projectPath: $projectPath,
      requestPath: $requestPath,
      requestHeader: $requestHeader,
      duration: $duration,
      buildProfile: $buildProfile,
      threads: $threads,
      connections: $connections,
      samples: $samples,
      wrkProcesses: $wrkProcesses,
      wrkTotalConnections: $wrkTotalConnections,
      targetRequests: $targetRequests,
      clusterRelayWorkers: $clusterRelayWorkers,
      clusterRelayQueue: $clusterRelayQueue,
      clusterAcceptWorkers: $clusterAcceptWorkers,
      clusterRelayAcceptBatchMax: $clusterRelayAcceptBatchMax,
      clusterRelayPumpBatchMax: $clusterRelayPumpBatchMax,
      dbBase: (if $dbBase == "" then null else $dbBase end),
      dbAdapter: (if $dbAdapter == "" then null else $dbAdapter end),
      dbPostgresDsnFile: (if $dbPostgresDsnFile == "" then null else $dbPostgresDsnFile end),
      dbPostgresTlsMode: (if $dbPostgresTlsMode == "" then null else $dbPostgresTlsMode end),
      dbMaxTxHandles: $dbMaxTxHandles,
      dbRecordsMax: $dbRecordsMax,
      dbPostgresStatementCacheMax: $dbPostgresStatementCacheMax,
      dbPostgresPlaceholderCacheMax: $dbPostgresPlaceholderCacheMax,
      dbPostgresStatementTimeoutMs: $dbPostgresStatementTimeoutMs,
      dbPostgresLockTimeoutMs: $dbPostgresLockTimeoutMs,
      dbPostgresConnectTimeoutMs: $dbPostgresConnectTimeoutMs,
      dbSqliteBusyTimeoutMs: $dbSqliteBusyTimeoutMs,
      dbSqliteJournalMode: (if $dbSqliteJournalMode == "" then null else $dbSqliteJournalMode end),
      dbSqliteSynchronous: (if $dbSqliteSynchronous == "" then null else $dbSqliteSynchronous end),
      dbPostgresRetryableConflictRetryMax: $dbPostgresRetryableConflictRetryMax,
      dbSqliteLockRetryMax: $dbSqliteLockRetryMax,
      dbSqliteLockRetryDelayMs: $dbSqliteLockRetryDelayMs,
      fixedReusePortMode: $fixedReusePortMode
    },
    boostSteps: $boostSteps,
    runs: $runs
  }' > "${out_path}"

echo "wrote ${out_path}"

if [ "${skip_analysis}" != "true" ]; then
  "${analyze_script}" "${out_path}" "${analysis_out_path}"
  recommended_step="$(jq -r '.summary.recommendedBoostStep' "${analysis_out_path}")"
  echo "recommendedSaturationBoostStep=${recommended_step}"

  if [ "${verify_recommended}" = "true" ]; then
    verify_cmd=(
      "${probe_script}"
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
      --autoscale-saturation-boost-step "${recommended_step}"
      --build-profile "${build_profile}"
      --samples "${samples}"
      --wrk-processes "${wrk_processes}"
      --out "${verify_out_path}"
      --skip-build
    )
    if [ "${fixed_reuse_port_mode}" = "true" ]; then
      verify_cmd+=(--fixed-reuse-port-mode)
    fi
    if [ -n "${cluster_relay_workers}" ]; then
      verify_cmd+=(--cluster-relay-workers "${cluster_relay_workers}")
    fi
    if [ -n "${cluster_relay_queue}" ]; then
      verify_cmd+=(--cluster-relay-queue "${cluster_relay_queue}")
    fi
    if [ -n "${cluster_accept_workers}" ]; then
      verify_cmd+=(--cluster-accept-workers "${cluster_accept_workers}")
    fi
    if [ -n "${cluster_relay_accept_batch_max}" ]; then
      verify_cmd+=(--cluster-relay-accept-batch-max "${cluster_relay_accept_batch_max}")
    fi
    if [ -n "${cluster_relay_pump_batch_max}" ]; then
      verify_cmd+=(--cluster-relay-pump-batch-max "${cluster_relay_pump_batch_max}")
    fi
    if [ -n "${db_base}" ]; then
      verify_cmd+=(--db-base "${db_base}")
    fi
    if [ -n "${db_adapter}" ]; then
      verify_cmd+=(--db-adapter "${db_adapter}")
    fi
    if [ -n "${db_postgres_dsn_file}" ]; then
      verify_cmd+=(--db-postgres-dsn-file "${db_postgres_dsn_file}")
    fi
    if [ -n "${db_postgres_tls_mode}" ]; then
      verify_cmd+=(--db-postgres-tls-mode "${db_postgres_tls_mode}")
    fi
    if [ -n "${db_max_tx_handles}" ]; then
      verify_cmd+=(--db-max-tx-handles "${db_max_tx_handles}")
    fi
    if [ -n "${db_records_max}" ]; then
      verify_cmd+=(--db-records-max "${db_records_max}")
    fi
    if [ -n "${db_postgres_statement_cache_max}" ]; then
      verify_cmd+=(--db-postgres-statement-cache-max "${db_postgres_statement_cache_max}")
    fi
    if [ -n "${db_postgres_placeholder_cache_max}" ]; then
      verify_cmd+=(--db-postgres-placeholder-cache-max "${db_postgres_placeholder_cache_max}")
    fi
    if [ -n "${db_postgres_statement_timeout_ms}" ]; then
      verify_cmd+=(--db-postgres-statement-timeout-ms "${db_postgres_statement_timeout_ms}")
    fi
    if [ -n "${db_postgres_lock_timeout_ms}" ]; then
      verify_cmd+=(--db-postgres-lock-timeout-ms "${db_postgres_lock_timeout_ms}")
    fi
    if [ -n "${db_postgres_connect_timeout_ms}" ]; then
      verify_cmd+=(--db-postgres-connect-timeout-ms "${db_postgres_connect_timeout_ms}")
    fi
    if [ -n "${db_sqlite_busy_timeout_ms}" ]; then
      verify_cmd+=(--db-sqlite-busy-timeout-ms "${db_sqlite_busy_timeout_ms}")
    fi
    if [ -n "${db_sqlite_journal_mode}" ]; then
      verify_cmd+=(--db-sqlite-journal-mode "${db_sqlite_journal_mode}")
    fi
    if [ -n "${db_sqlite_synchronous}" ]; then
      verify_cmd+=(--db-sqlite-synchronous "${db_sqlite_synchronous}")
    fi
    if [ -n "${db_postgres_retryable_conflict_retry_max}" ]; then
      verify_cmd+=(--db-postgres-retryable-conflict-retry-max "${db_postgres_retryable_conflict_retry_max}")
    fi
    if [ -n "${db_sqlite_lock_retry_max}" ]; then
      verify_cmd+=(--db-sqlite-lock-retry-max "${db_sqlite_lock_retry_max}")
    fi
    if [ -n "${db_sqlite_lock_retry_delay_ms}" ]; then
      verify_cmd+=(--db-sqlite-lock-retry-delay-ms "${db_sqlite_lock_retry_delay_ms}")
    fi
    echo "verifyingRecommendedBoostStep=${recommended_step}"
    "${verify_cmd[@]}"
    echo "recommendedVerificationOut=${verify_out_path}"
  fi
fi
