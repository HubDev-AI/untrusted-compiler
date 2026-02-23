#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs sec4 LASM cluster mode under load, samples peak RSS, and writes a
capacity probe result JSON.

Options:
  --profile <ping|db-hot-write|db-hot-write-tx|db-hot-query-one>
                                                 Probe profile (default: ping)
  --project-path <path>                            Project path passed to sec4 run (default: examples/lasm-alpha-full)
  --request-path <path>                            Probe HTTP path (default: /health)
  --warmup-path <path>                             Optional warmup HTTP path hit before readiness checks
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
  --fixed-reuse-port-mode                          Run fixed reuse-port cluster mode (forces autoscale-max-instances=instances; relay proxy tuning flags are disallowed)
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
  --build-profile <debug|release>                  sec4 build profile used for probe run (default: release)
  --samples <n>                                    Number of wrk samples (best sample is reported, default: 1)
  --wrk-processes <n>                              Number of parallel wrk processes per sample (default: 1)
  --keep-cluster-status-json                       Keep raw cluster status json artifact after probe
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

sum_numbers() {
  local left="$1"
  local right="$2"
  awk -v a="$left" -v b="$right" 'BEGIN { printf "%.6f", (a + 0) + (b + 0) }'
}

latency_to_ms() {
  local value="$1"
  awk -v raw="$value" '
    BEGIN {
      unit = "";
      amount = raw;
      if (raw ~ /us$/) {
        unit = "us";
        sub(/us$/, "", amount);
      } else if (raw ~ /ms$/) {
        unit = "ms";
        sub(/ms$/, "", amount);
      } else if (raw ~ /s$/) {
        unit = "s";
        sub(/s$/, "", amount);
      } else if (raw ~ /m$/) {
        unit = "m";
        sub(/m$/, "", amount);
      } else {
        exit 1;
      }
      if (amount !~ /^-?[0-9]+([.][0-9]+)?$/) {
        exit 1;
      }
      amount += 0;
      if (unit == "us") {
        printf "%.6f", amount / 1000.0;
      } else if (unit == "ms") {
        printf "%.6f", amount;
      } else if (unit == "s") {
        printf "%.6f", amount * 1000.0;
      } else if (unit == "m") {
        printf "%.6f", amount * 60000.0;
      } else {
        exit 1;
      }
    }
  '
}

format_latency_ms() {
  local value_ms="$1"
  awk -v x="$value_ms" 'BEGIN { printf "%.3fms", x + 0 }'
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
warmup_path="${LASM_CAPACITY_WARMUP_PATH:-}"
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
out_rel="${LASM_CAPACITY_OUT:-results/summaries/sec4-lasm-cluster-capacity-probe.json}"
fixed_reuse_port_mode="${LASM_CAPACITY_FIXED_REUSE_PORT_MODE:-false}"
skip_build="false"
dry_run="false"
keep_cluster_status_json="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
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
    --warmup-path)
      warmup_path="${2:-}"
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
    --keep-cluster-status-json)
      keep_cluster_status_json="true"
      shift
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
for field in db_max_tx_handles db_records_max db_postgres_statement_cache_max db_postgres_placeholder_cache_max db_postgres_statement_timeout_ms db_postgres_lock_timeout_ms db_postgres_connect_timeout_ms db_sqlite_busy_timeout_ms db_postgres_retryable_conflict_retry_max db_sqlite_lock_retry_max db_sqlite_lock_retry_delay_ms; do
  value="${!field}"
  if [ -n "$value" ] && ! is_number "$value"; then
    echo "${field//_/-} must be numeric, got: $value" >&2
    exit 2
  fi
done
if [ -n "$db_adapter" ]; then
  case "$db_adapter" in
    records-log|sqlite|postgres) ;;
    *)
      echo "db-adapter must be one of: records-log, sqlite, postgres (got: $db_adapter)" >&2
      exit 2
      ;;
  esac
fi
if [ -n "$db_postgres_tls_mode" ]; then
  case "$db_postgres_tls_mode" in
    auto|disable|require) ;;
    *)
      echo "db-postgres-tls-mode must be one of: auto, disable, require (got: $db_postgres_tls_mode)" >&2
      exit 2
      ;;
  esac
fi
if [ "$fixed_reuse_port_mode" != "true" ] && [ "$fixed_reuse_port_mode" != "false" ]; then
  echo "fixed-reuse-port-mode must be true or false, got: $fixed_reuse_port_mode" >&2
  exit 2
fi
case "$build_profile" in
  debug|release) ;;
  *)
    echo "build-profile must be one of: debug, release (got: $build_profile)" >&2
    exit 2
    ;;
esac
if ! [[ "$samples" =~ ^[0-9]+$ ]]; then
  echo "samples must be an integer >= 1, got: $samples" >&2
  exit 2
fi
if [ "$samples" -lt 1 ]; then
  echo "samples must be >= 1, got: $samples" >&2
  exit 2
fi
if ! [[ "$wrk_processes" =~ ^[0-9]+$ ]]; then
  echo "wrk-processes must be an integer >= 1, got: $wrk_processes" >&2
  exit 2
fi
if [ "$wrk_processes" -lt 1 ]; then
  echo "wrk-processes must be >= 1, got: $wrk_processes" >&2
  exit 2
fi
if [ -z "$request_header" ] || [[ "$request_header" != *:* ]]; then
  echo "request-header must include ':' (example: Authorization: Bearer token123)" >&2
  exit 2
fi
case "$profile" in
  ping|db-hot-write|db-hot-write-tx|db-hot-query-one) ;;
  *)
    echo "profile must be one of: ping, db-hot-write, db-hot-write-tx, db-hot-query-one (got: $profile)" >&2
    exit 2
    ;;
esac
if [ "$fixed_reuse_port_mode" = "true" ]; then
  autoscale_max_instances="$instances"
  if [ -n "$cluster_relay_workers" ]; then
    echo "cluster-relay-workers is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
  if [ -n "$cluster_relay_queue" ]; then
    echo "cluster-relay-queue is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
  if [ -n "$cluster_accept_workers" ]; then
    echo "cluster-accept-workers is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
  if [ -n "$cluster_relay_accept_batch_max" ]; then
    echo "cluster-relay-accept-batch-max is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
  if [ -n "$cluster_relay_pump_batch_max" ]; then
    echo "cluster-relay-pump-batch-max is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
  if [ "$keep_cluster_status_json" = "true" ]; then
    echo "keep-cluster-status-json is not supported in fixed-reuse-port-mode" >&2
    exit 2
  fi
fi

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "${root_dir}/.." && pwd)"
if [ "$project_path_explicit" != "true" ]; then
  case "$profile" in
    ping)
      project_path="examples/lasm-alpha-full"
      ;;
    db-hot-write|db-hot-write-tx|db-hot-query-one)
      project_path="benchmark-suite/services/sec4-lasm"
      ;;
  esac
fi
if [ "$request_path_explicit" != "true" ]; then
  case "$profile" in
    ping)
      request_path="/health"
      ;;
    db-hot-write)
      request_path="/db/hot-write"
      ;;
    db-hot-write-tx)
      request_path="/db/hot-write-tx"
      ;;
    db-hot-query-one)
      request_path="/db/hot-query-one"
      ;;
  esac
fi
if [ "$profile" = "db-hot-query-one" ] && [ -z "$warmup_path" ]; then
  warmup_path="/db/hot-write"
fi
if [ "$profile" != "ping" ] && [ -z "$db_adapter" ]; then
  db_adapter="records-log"
fi
if [ "$profile" != "ping" ] && [ -z "$db_base" ]; then
  db_base="${root_dir}/results/raw/sec4-lasm-cluster-db"
fi
if [ "$build_profile" = "release" ]; then
  sec4_bin="${repo_root}/target/release/sec4"
  cargo_build_profile_arg=(--release)
else
  sec4_bin="${repo_root}/target/debug/sec4"
  cargo_build_profile_arg=()
fi
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
  profile=$profile
  projectPath=$project_abs
  baseUrl=$base_url
  requestPath=$request_path
  warmupPath=${warmup_path:-none}
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
  fixedReusePortMode=$fixed_reuse_port_mode
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
  buildProfile=$build_profile
  samples=$samples
  wrkProcesses=$wrk_processes
  clusterStatusJson=$(if [ "$fixed_reuse_port_mode" = "true" ]; then printf "%s" "n/a (fixed-reuse-port-mode)"; else printf "%s" "$status_json_file"; fi)
  keepClusterStatusJson=${keep_cluster_status_json}
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
  (cd "$repo_root" && cargo build -p sec4 "${cargo_build_profile_arg[@]}" >/dev/null)
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
)
if [ "$fixed_reuse_port_mode" != "true" ]; then
  run_args+=(--cluster-status-json "$status_json_file")
fi
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
if [ -n "$cluster_relay_pump_batch_max" ]; then
  run_args+=(--cluster-relay-pump-batch-max "$cluster_relay_pump_batch_max")
fi
if [ -n "$db_base" ]; then
  run_args+=(--db-base "$db_base")
fi
if [ -n "$db_adapter" ]; then
  run_args+=(--db-adapter "$db_adapter")
fi
if [ -n "$db_postgres_dsn_file" ]; then
  run_args+=(--db-postgres-dsn-file "$db_postgres_dsn_file")
fi
if [ -n "$db_postgres_tls_mode" ]; then
  run_args+=(--db-postgres-tls-mode "$db_postgres_tls_mode")
fi
if [ -n "$db_max_tx_handles" ]; then
  run_args+=(--db-max-tx-handles "$db_max_tx_handles")
fi
if [ -n "$db_records_max" ]; then
  run_args+=(--db-records-max "$db_records_max")
fi
if [ -n "$db_postgres_statement_cache_max" ]; then
  run_args+=(--db-postgres-statement-cache-max "$db_postgres_statement_cache_max")
fi
if [ -n "$db_postgres_placeholder_cache_max" ]; then
  run_args+=(--db-postgres-placeholder-cache-max "$db_postgres_placeholder_cache_max")
fi
if [ -n "$db_postgres_statement_timeout_ms" ]; then
  run_args+=(--db-postgres-statement-timeout-ms "$db_postgres_statement_timeout_ms")
fi
if [ -n "$db_postgres_lock_timeout_ms" ]; then
  run_args+=(--db-postgres-lock-timeout-ms "$db_postgres_lock_timeout_ms")
fi
if [ -n "$db_postgres_connect_timeout_ms" ]; then
  run_args+=(--db-postgres-connect-timeout-ms "$db_postgres_connect_timeout_ms")
fi
if [ -n "$db_sqlite_busy_timeout_ms" ]; then
  run_args+=(--db-sqlite-busy-timeout-ms "$db_sqlite_busy_timeout_ms")
fi
if [ -n "$db_sqlite_journal_mode" ]; then
  run_args+=(--db-sqlite-journal-mode "$db_sqlite_journal_mode")
fi
if [ -n "$db_sqlite_synchronous" ]; then
  run_args+=(--db-sqlite-synchronous "$db_sqlite_synchronous")
fi
if [ -n "$db_postgres_retryable_conflict_retry_max" ]; then
  run_args+=(--db-postgres-retryable-conflict-retry-max "$db_postgres_retryable_conflict_retry_max")
fi
if [ -n "$db_sqlite_lock_retry_max" ]; then
  run_args+=(--db-sqlite-lock-retry-max "$db_sqlite_lock_retry_max")
fi
if [ -n "$db_sqlite_lock_retry_delay_ms" ]; then
  run_args+=(--db-sqlite-lock-retry-delay-ms "$db_sqlite_lock_retry_delay_ms")
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
warmup_done="false"
for _ in $(seq 1 200); do
  if [ -n "$warmup_path" ] && [ "$warmup_done" != "true" ]; then
    if curl -fsS -H "$request_header" "${base_url}${warmup_path}" >/dev/null 2>&1; then
      warmup_done="true"
    fi
  fi
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
sample_success_count=0
sample_failure_count=0
best_sample_index=0
best_sample_raw_file="$raw_file"
best_sample_requests=0
best_sample_requests_per_sec=0
best_sample_p99=""
sample_entries_jsonl="${root_dir}/results/raw/sec4-lasm-cluster-${probe_label}-samples.jsonl"
: >"$sample_entries_jsonl"
for sample_index in $(seq 1 "$samples"); do
  sample_raw_file="$raw_file"
  if [ "$samples" -gt 1 ] || [ "$wrk_processes" -gt 1 ]; then
    sample_raw_file="${raw_file%.txt}-sample-${sample_index}.txt"
  fi

  sample_exit_code=0
  sample_requests=0
  sample_requests_per_sec=0
  sample_p99_ms=""
  sample_process_entries_jsonl="${root_dir}/results/raw/sec4-lasm-cluster-${probe_label}-sample-${sample_index}-wrk-processes.jsonl"
  : >"$sample_process_entries_jsonl"
  sample_process_pids=()
  sample_process_raw_files=()

  for process_index in $(seq 1 "$wrk_processes"); do
    sample_process_raw_file="$sample_raw_file"
    if [ "$wrk_processes" -gt 1 ]; then
      sample_process_raw_file="${sample_raw_file%.txt}-wrk-${process_index}.txt"
    fi
    sample_process_raw_files+=("$sample_process_raw_file")
    wrk -t"$threads" -c"$connections" -d"$duration" --latency -H "$request_header" "${base_url}${request_path}" >"$sample_process_raw_file" 2>&1 &
    sample_process_pids+=("$!")
  done

  for process_offset in "${!sample_process_pids[@]}"; do
    process_pid="${sample_process_pids[$process_offset]}"
    process_index=$((process_offset + 1))
    sample_process_raw_file="${sample_process_raw_files[$process_offset]}"

    process_exit_code=0
    if ! wait "$process_pid"; then
      process_exit_code=$?
      if [ "$sample_exit_code" -eq 0 ]; then
        sample_exit_code=$process_exit_code
      fi
      if [ "$bench_rc" -eq 0 ]; then
        bench_rc=$process_exit_code
      fi
    fi

    process_requests=0
    process_requests_per_sec=0
    process_p99=""
    process_p99_ms=""
    if [ -f "$sample_process_raw_file" ]; then
      process_requests_raw="$(awk '/requests in/ {gsub(/,/,"",$1); print $1; exit}' "$sample_process_raw_file")"
      if is_number "$process_requests_raw"; then
        process_requests="$process_requests_raw"
      fi
      process_requests_per_sec_raw="$(awk '/^Requests\/sec:/ {print $2; exit}' "$sample_process_raw_file")"
      if is_number "$process_requests_per_sec_raw"; then
        process_requests_per_sec="$process_requests_per_sec_raw"
      fi
      process_p99_raw="$(awk '$1 == "99%" {print $2; exit}' "$sample_process_raw_file")"
      if [ -n "$process_p99_raw" ]; then
        process_p99_ms="$(latency_to_ms "$process_p99_raw" 2>/dev/null || true)"
        if [ -n "$process_p99_ms" ]; then
          process_p99="$(format_latency_ms "$process_p99_ms")"
        fi
      fi
    fi

    sample_requests=$((sample_requests + process_requests))
    sample_requests_per_sec="$(sum_numbers "$sample_requests_per_sec" "$process_requests_per_sec")"
    if [ -n "$process_p99_ms" ]; then
      if [ -z "$sample_p99_ms" ] || awk -v current="$process_p99_ms" -v best="$sample_p99_ms" 'BEGIN { exit !(current + 0 > best + 0) }'; then
        sample_p99_ms="$process_p99_ms"
      fi
    fi

    jq -n \
      --argjson process "$process_index" \
      --arg raw "$sample_process_raw_file" \
      --argjson requests "$process_requests" \
      --argjson requestsPerSec "$process_requests_per_sec" \
      --arg p99 "$process_p99" \
      --argjson runExitCode "$process_exit_code" \
      '{
        process: $process,
        raw: $raw,
        requests: $requests,
        requestsPerSec: $requestsPerSec,
        p99: $p99,
        runExitCode: $runExitCode
      }' >>"$sample_process_entries_jsonl"
  done

  if [ "$wrk_processes" -gt 1 ]; then
    {
      printf "# sec4 LASM cluster capacity sample %s (%s wrk processes)\n" "$sample_index" "$wrk_processes"
      for process_offset in "${!sample_process_raw_files[@]}"; do
        process_index=$((process_offset + 1))
        sample_process_raw_file="${sample_process_raw_files[$process_offset]}"
        printf "\n## wrk process %s (%s)\n\n" "$process_index" "$sample_process_raw_file"
        if [ -f "$sample_process_raw_file" ]; then
          cat "$sample_process_raw_file"
        else
          echo "(missing wrk output)"
        fi
      done
    } >"$sample_raw_file"
  fi

  sample_p99=""
  if [ -n "$sample_p99_ms" ]; then
    sample_p99="$(format_latency_ms "$sample_p99_ms")"
  fi
  sample_process_entries_json="$(jq -s '.' "$sample_process_entries_jsonl")"
  rm -f "$sample_process_entries_jsonl"

  if [ "$sample_exit_code" -eq 0 ]; then
    sample_success_count=$((sample_success_count + 1))
  else
    sample_failure_count=$((sample_failure_count + 1))
  fi

  jq -n \
    --argjson index "$sample_index" \
    --arg raw "$sample_raw_file" \
    --argjson requests "$sample_requests" \
    --argjson requestsPerSec "$sample_requests_per_sec" \
    --arg p99 "$sample_p99" \
    --argjson wrkProcesses "$wrk_processes" \
    --argjson wrkProcessRuns "$sample_process_entries_json" \
    --argjson runExitCode "$sample_exit_code" \
    '{
      index: $index,
      raw: $raw,
      requests: $requests,
      requestsPerSec: $requestsPerSec,
      p99: $p99,
      wrkProcesses: $wrkProcesses,
      wrkProcessRuns: $wrkProcessRuns,
      runExitCode: $runExitCode
    }' >>"$sample_entries_jsonl"

  if awk -v current="$sample_requests_per_sec" -v best="$best_sample_requests_per_sec" 'BEGIN { exit !(current + 0 > best + 0) }'; then
    best_sample_index="$sample_index"
    best_sample_raw_file="$sample_raw_file"
    best_sample_requests="$sample_requests"
    best_sample_requests_per_sec="$sample_requests_per_sec"
    best_sample_p99="$sample_p99"
  fi
done
if [ "$sample_success_count" -gt 0 ]; then
  bench_rc=0
fi
raw_file="$best_sample_raw_file"

if kill -0 "$server_pid" >/dev/null 2>&1; then
  kill "$server_pid" >/dev/null 2>&1 || true
  wait "$server_pid" >/dev/null 2>&1 || true
fi
if kill -0 "$sampler_pid" >/dev/null 2>&1; then
  kill "$sampler_pid" >/dev/null 2>&1 || true
  wait "$sampler_pid" >/dev/null 2>&1 || true
fi

observed_requests="$best_sample_requests"
observed_requests_per_sec="$best_sample_requests_per_sec"
peak_rss_kb=0
p99="$best_sample_p99"
resolved_relay_worker_count="null"
resolved_relay_accept_workers="null"
resolved_relay_accept_batch_max="null"
resolved_relay_pump_batch_max="null"
resolved_relay_queue_capacity="null"
resolved_relay_queue_shard_capacity="null"
resolved_relay_dispatch_short_circuit_total="null"
resolved_relay_dispatch_short_circuit_per_sec="null"
resolved_relay_live_sender_count="null"
resolved_db_adapter=""
resolved_db_postgres_tls_mode=""
resolved_db_max_tx_handles="null"
resolved_db_records_max="null"
resolved_db_postgres_statement_cache_max="null"
resolved_db_postgres_placeholder_cache_max="null"
resolved_db_postgres_statement_timeout_ms="null"
resolved_db_postgres_lock_timeout_ms="null"
resolved_db_postgres_connect_timeout_ms="null"
resolved_db_sqlite_busy_timeout_ms="null"
resolved_db_sqlite_journal_mode=""
resolved_db_sqlite_synchronous=""
resolved_db_postgres_retryable_conflict_retry_max="null"
resolved_db_sqlite_lock_retry_max="null"
resolved_db_sqlite_lock_retry_delay_ms="null"
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
  status_relay_pump_batch_max="$(jq -r '.relayPumpBatchMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_pump_batch_max"; then
    resolved_relay_pump_batch_max="$status_relay_pump_batch_max"
  fi
  status_relay_queue_capacity="$(jq -r '.relayQueueCapacity // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_queue_capacity"; then
    resolved_relay_queue_capacity="$status_relay_queue_capacity"
  fi
  status_relay_queue_shard_capacity="$(jq -r '.relayQueueShardCapacity // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_queue_shard_capacity"; then
    resolved_relay_queue_shard_capacity="$status_relay_queue_shard_capacity"
  fi
  status_relay_dispatch_short_circuit_total="$(jq -r '.relayDispatchSaturationShortCircuitTotal // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_dispatch_short_circuit_total"; then
    resolved_relay_dispatch_short_circuit_total="$status_relay_dispatch_short_circuit_total"
  fi
  status_relay_dispatch_short_circuit_per_sec="$(jq -r '.relayDispatchSaturationShortCircuitPerSec // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_dispatch_short_circuit_per_sec"; then
    resolved_relay_dispatch_short_circuit_per_sec="$status_relay_dispatch_short_circuit_per_sec"
  fi
  status_relay_live_sender_count="$(jq -r '.relayLiveSenderCount // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_relay_live_sender_count"; then
    resolved_relay_live_sender_count="$status_relay_live_sender_count"
  fi
  status_db_adapter="$(jq -r '.dbAdapter // empty' "$status_json_file" 2>/dev/null || true)"
  if [ -n "$status_db_adapter" ]; then
    resolved_db_adapter="$status_db_adapter"
  fi
  status_db_postgres_tls_mode="$(jq -r '.dbPostgresTlsMode // empty' "$status_json_file" 2>/dev/null || true)"
  if [ -n "$status_db_postgres_tls_mode" ]; then
    resolved_db_postgres_tls_mode="$status_db_postgres_tls_mode"
  fi
  status_db_max_tx_handles="$(jq -r '.dbMaxTxHandles // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_max_tx_handles"; then
    resolved_db_max_tx_handles="$status_db_max_tx_handles"
  fi
  status_db_records_max="$(jq -r '.dbRecordsMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_records_max"; then
    resolved_db_records_max="$status_db_records_max"
  fi
  status_db_postgres_statement_cache_max="$(jq -r '.dbPostgresStatementCacheMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_statement_cache_max"; then
    resolved_db_postgres_statement_cache_max="$status_db_postgres_statement_cache_max"
  fi
  status_db_postgres_placeholder_cache_max="$(jq -r '.dbPostgresPlaceholderCacheMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_placeholder_cache_max"; then
    resolved_db_postgres_placeholder_cache_max="$status_db_postgres_placeholder_cache_max"
  fi
  status_db_postgres_statement_timeout_ms="$(jq -r '.dbPostgresStatementTimeoutMs // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_statement_timeout_ms"; then
    resolved_db_postgres_statement_timeout_ms="$status_db_postgres_statement_timeout_ms"
  fi
  status_db_postgres_lock_timeout_ms="$(jq -r '.dbPostgresLockTimeoutMs // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_lock_timeout_ms"; then
    resolved_db_postgres_lock_timeout_ms="$status_db_postgres_lock_timeout_ms"
  fi
  status_db_postgres_connect_timeout_ms="$(jq -r '.dbPostgresConnectTimeoutMs // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_connect_timeout_ms"; then
    resolved_db_postgres_connect_timeout_ms="$status_db_postgres_connect_timeout_ms"
  fi
  status_db_sqlite_busy_timeout_ms="$(jq -r '.dbSqliteBusyTimeoutMs // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_sqlite_busy_timeout_ms"; then
    resolved_db_sqlite_busy_timeout_ms="$status_db_sqlite_busy_timeout_ms"
  fi
  status_db_sqlite_journal_mode="$(jq -r '.dbSqliteJournalMode // empty' "$status_json_file" 2>/dev/null || true)"
  if [ -n "$status_db_sqlite_journal_mode" ]; then
    resolved_db_sqlite_journal_mode="$status_db_sqlite_journal_mode"
  fi
  status_db_sqlite_synchronous="$(jq -r '.dbSqliteSynchronous // empty' "$status_json_file" 2>/dev/null || true)"
  if [ -n "$status_db_sqlite_synchronous" ]; then
    resolved_db_sqlite_synchronous="$status_db_sqlite_synchronous"
  fi
  status_db_postgres_retryable_conflict_retry_max="$(jq -r '.dbPostgresRetryableConflictRetryMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_postgres_retryable_conflict_retry_max"; then
    resolved_db_postgres_retryable_conflict_retry_max="$status_db_postgres_retryable_conflict_retry_max"
  fi
  status_db_sqlite_lock_retry_max="$(jq -r '.dbSqliteLockRetryMax // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_sqlite_lock_retry_max"; then
    resolved_db_sqlite_lock_retry_max="$status_db_sqlite_lock_retry_max"
  fi
  status_db_sqlite_lock_retry_delay_ms="$(jq -r '.dbSqliteLockRetryDelayMs // empty' "$status_json_file" 2>/dev/null || true)"
  if is_number "$status_db_sqlite_lock_retry_delay_ms"; then
    resolved_db_sqlite_lock_retry_delay_ms="$status_db_sqlite_lock_retry_delay_ms"
  fi
fi

sample_entries_json="$(jq -s '.' "$sample_entries_jsonl")"
wrk_total_connections="$(awk -v c="$connections" -v p="$wrk_processes" 'BEGIN { printf "%.0f", (c + 0) * (p + 0) }')"

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
  --arg profile "$profile" \
  --arg projectPath "$project_abs" \
  --arg baseUrl "$base_url" \
  --arg requestPath "$request_path" \
  --arg warmupPath "$warmup_path" \
  --arg requestHeader "$request_header" \
  --arg duration "$duration" \
  --arg buildProfile "$build_profile" \
  --arg p99 "$p99" \
  --arg rawFile "$raw_file" \
  --arg serverLog "$server_log" \
  --arg statusJsonFile "$status_json_file" \
  --argjson threads "$threads" \
  --argjson connections "$connections" \
  --argjson wrkProcesses "$wrk_processes" \
  --argjson wrkTotalConnections "$wrk_total_connections" \
  --argjson sampleCount "$samples" \
  --argjson sampleSuccessCount "$sample_success_count" \
  --argjson sampleFailureCount "$sample_failure_count" \
  --argjson selectedSample "$best_sample_index" \
  --argjson targetRequests "$target_requests" \
  --argjson observedRequests "$observed_requests" \
  --argjson observedRequestsPerSec "$observed_requests_per_sec" \
  --argjson sampleRuns "$sample_entries_json" \
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
  --argjson fixedReusePortMode "$fixed_reuse_port_mode" \
  --argjson resolvedRelayWorkerCount "$resolved_relay_worker_count" \
  --argjson resolvedRelayAcceptWorkers "$resolved_relay_accept_workers" \
  --argjson resolvedRelayAcceptBatchMax "$resolved_relay_accept_batch_max" \
  --argjson resolvedRelayPumpBatchMax "$resolved_relay_pump_batch_max" \
  --argjson resolvedRelayQueueCapacity "$resolved_relay_queue_capacity" \
  --argjson resolvedRelayQueueShardCapacity "$resolved_relay_queue_shard_capacity" \
  --argjson resolvedRelayDispatchShortCircuitTotal "$resolved_relay_dispatch_short_circuit_total" \
  --argjson resolvedRelayDispatchShortCircuitPerSec "$resolved_relay_dispatch_short_circuit_per_sec" \
  --argjson resolvedRelayLiveSenderCount "$resolved_relay_live_sender_count" \
  --arg resolvedDbAdapter "$resolved_db_adapter" \
  --arg resolvedDbPostgresTlsMode "$resolved_db_postgres_tls_mode" \
  --argjson resolvedDbMaxTxHandles "$resolved_db_max_tx_handles" \
  --argjson resolvedDbRecordsMax "$resolved_db_records_max" \
  --argjson resolvedDbPostgresStatementCacheMax "$resolved_db_postgres_statement_cache_max" \
  --argjson resolvedDbPostgresPlaceholderCacheMax "$resolved_db_postgres_placeholder_cache_max" \
  --argjson resolvedDbPostgresStatementTimeoutMs "$resolved_db_postgres_statement_timeout_ms" \
  --argjson resolvedDbPostgresLockTimeoutMs "$resolved_db_postgres_lock_timeout_ms" \
  --argjson resolvedDbPostgresConnectTimeoutMs "$resolved_db_postgres_connect_timeout_ms" \
  --argjson resolvedDbSqliteBusyTimeoutMs "$resolved_db_sqlite_busy_timeout_ms" \
  --arg resolvedDbSqliteJournalMode "$resolved_db_sqlite_journal_mode" \
  --arg resolvedDbSqliteSynchronous "$resolved_db_sqlite_synchronous" \
  --argjson resolvedDbPostgresRetryableConflictRetryMax "$resolved_db_postgres_retryable_conflict_retry_max" \
  --argjson resolvedDbSqliteLockRetryMax "$resolved_db_sqlite_lock_retry_max" \
  --argjson resolvedDbSqliteLockRetryDelayMs "$resolved_db_sqlite_lock_retry_delay_ms" \
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
  --arg relayWorkers "${cluster_relay_workers:-auto}" \
  --arg relayQueue "${cluster_relay_queue:-auto}" \
  --arg acceptWorkers "${cluster_accept_workers:-auto}" \
  --arg relayAcceptBatchMax "${cluster_relay_accept_batch_max:-auto}" \
  --arg relayPumpBatchMax "${cluster_relay_pump_batch_max:-auto}" \
  '{
    impl: $impl,
    profile: $profile,
    projectPath: $projectPath,
    baseUrl: $baseUrl,
    requestPath: $requestPath,
    warmupPath: (if $warmupPath == "" then null else $warmupPath end),
    requestHeader: $requestHeader,
    pass: $pass,
    runExitCode: $runExitCode,
    requestsTargetMet: $requestsTargetMet,
    run: {
      profile: $profile,
      warmupPath: (if $warmupPath == "" then null else $warmupPath end),
      duration: $duration,
      buildProfile: $buildProfile,
      threads: $threads,
      connections: $connections,
      wrkProcesses: $wrkProcesses,
      wrkTotalConnections: $wrkTotalConnections,
      samples: $sampleCount,
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
      fixedReusePortMode: $fixedReusePortMode,
      clusterRelayWorkers: $relayWorkers,
      clusterRelayQueue: $relayQueue,
      clusterAcceptWorkers: $acceptWorkers,
      clusterRelayAcceptBatchMax: $relayAcceptBatchMax,
      clusterRelayPumpBatchMax: $relayPumpBatchMax,
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
      clusterRelayWorkersResolved: $resolvedRelayWorkerCount,
      clusterAcceptWorkersResolved: $resolvedRelayAcceptWorkers,
      clusterRelayAcceptBatchMaxResolved: $resolvedRelayAcceptBatchMax,
      clusterRelayPumpBatchMaxResolved: $resolvedRelayPumpBatchMax,
      clusterRelayQueueCapacityResolved: $resolvedRelayQueueCapacity,
      clusterRelayQueueShardCapacityResolved: $resolvedRelayQueueShardCapacity,
      clusterRelayDispatchSaturationShortCircuitTotal: $resolvedRelayDispatchShortCircuitTotal,
      clusterRelayDispatchSaturationShortCircuitPerSec: $resolvedRelayDispatchShortCircuitPerSec,
      clusterRelayLiveSenderCountResolved: $resolvedRelayLiveSenderCount,
      clusterDbAdapterResolved: (if $resolvedDbAdapter == "" then null else $resolvedDbAdapter end),
      clusterDbPostgresTlsModeResolved: (if $resolvedDbPostgresTlsMode == "" then null else $resolvedDbPostgresTlsMode end),
      clusterDbMaxTxHandlesResolved: $resolvedDbMaxTxHandles,
      clusterDbRecordsMaxResolved: $resolvedDbRecordsMax,
      clusterDbPostgresStatementCacheMaxResolved: $resolvedDbPostgresStatementCacheMax,
      clusterDbPostgresPlaceholderCacheMaxResolved: $resolvedDbPostgresPlaceholderCacheMax,
      clusterDbPostgresStatementTimeoutMsResolved: $resolvedDbPostgresStatementTimeoutMs,
      clusterDbPostgresLockTimeoutMsResolved: $resolvedDbPostgresLockTimeoutMs,
      clusterDbPostgresConnectTimeoutMsResolved: $resolvedDbPostgresConnectTimeoutMs,
      clusterDbSqliteBusyTimeoutMsResolved: $resolvedDbSqliteBusyTimeoutMs,
      clusterDbSqliteJournalModeResolved: (if $resolvedDbSqliteJournalMode == "" then null else $resolvedDbSqliteJournalMode end),
      clusterDbSqliteSynchronousResolved: (if $resolvedDbSqliteSynchronous == "" then null else $resolvedDbSqliteSynchronous end),
      clusterDbPostgresRetryableConflictRetryMaxResolved: $resolvedDbPostgresRetryableConflictRetryMax,
      clusterDbSqliteLockRetryMaxResolved: $resolvedDbSqliteLockRetryMax,
      clusterDbSqliteLockRetryDelayMsResolved: $resolvedDbSqliteLockRetryDelayMs
    },
    observed: {
      requests: $observedRequests,
      requestsPerSec: $observedRequestsPerSec,
      peakRssKb: $peakRssKb,
      p99: $p99,
      selectedSample: $selectedSample,
      sampleSuccessCount: $sampleSuccessCount,
      sampleFailureCount: $sampleFailureCount,
      samples: $sampleRuns
    },
    artifacts: {
      raw: $rawFile,
      serverLog: $serverLog,
      clusterStatusJson: (if $fixedReusePortMode then null else $statusJsonFile end)
    }
  }' >"$out_path"

if [ "$keep_cluster_status_json" != "true" ]; then
  rm -f "$status_json_file"
fi
rm -f "$sample_entries_jsonl"

echo "wrote $out_path"
if [ "$overall_pass" != "true" ]; then
  echo "LASM capacity probe failed: pass=false (runExitCode=$bench_rc, observedRequests=$observed_requests, targetRequests=$target_requests)" >&2
  exit 1
fi
echo "LASM capacity probe passed"
