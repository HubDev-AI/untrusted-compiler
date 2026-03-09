#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options]

Runs the canonical LASM workbench benchmark in three modes on the same workload:
  1) single-instance
  2) fixed reuse-port cluster
  3) proxy cluster

Then emits one comparison JSON artifact with per-endpoint leaders and an
overall recommended mode.

Options:
  --endpoints <csv>                               Benchmark endpoints (default: all workbench endpoints, including optional wb-tasks-with-comment-tx when requested)
  --port <n>                                      Service port (default: 18093)
  --lasm-db-adapter <sqlite|postgres>             LASM DB adapter (default: sqlite)
  --lasm-db-base <path>                           Optional LASM sqlite DB base
  --lasm-postgres-dsn-file <path>                 Optional LASM Postgres DSN file
  --lasm-db-records-capture-enabled <0|1>         Optional LASM runtime DB-record capture switch
  --lasm-db-postgres-shared-client-max-active-per-key <n>
                                                  Optional LASM Postgres active-pool per-key limit
  --lasm-db-postgres-shared-client-max-active-total <n>
                                                  Optional LASM Postgres active-pool total limit
  --instances <n>                                 Cluster worker count for fixed/proxy modes (default: 2)
  --autoscale-max-instances <n>                   Proxy-cluster max instances (default: 4)
  --autoscale-target-connections <n>              Proxy autoscale target (default: 256)
  --autoscale-check-ms <n>                        Proxy autoscale check interval (default: 1000)
  --cluster-relay-workers <n>                     Optional proxy relay worker override
  --cluster-relay-queue <n>                       Optional proxy relay queue override
  --cluster-accept-workers <n>                    Optional proxy accept-worker override
  --cluster-relay-accept-batch-max <n>            Optional proxy accept batch max override
  --cluster-relay-pump-batch-max <n>              Optional proxy relay pump batch max override
  --single-runs-out <path>                        Output runs JSON for single mode
  --fixed-runs-out <path>                         Output runs JSON for fixed-cluster mode
  --proxy-runs-out <path>                         Output runs JSON for proxy-cluster mode
  --single-report-out <path>                      Copied report JSON for single mode
  --fixed-report-out <path>                       Copied report JSON for fixed-cluster mode
  --proxy-report-out <path>                       Copied report JSON for proxy-cluster mode
  --out <path>                                    Comparison output JSON
  --dry-run                                       Print execution plan only
  -h, --help                                      Show this help
USAGE
}

endpoints_csv="wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list"
bench_port="${BENCH_WORKBENCH_PORT:-18093}"
lasm_db_adapter="${BENCH_WORKBENCH_LASM_DB_ADAPTER:-sqlite}"
lasm_db_base="${BENCH_WORKBENCH_LASM_DB_BASE:-}"
lasm_postgres_dsn_file="${BENCH_WORKBENCH_LASM_POSTGRES_DSN_FILE:-}"
lasm_db_records_capture_enabled="${BENCH_WORKBENCH_LASM_DB_RECORDS_CAPTURE_ENABLED:-}"
lasm_db_postgres_shared_client_max_active_per_key="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_PER_KEY:-}"
lasm_db_postgres_shared_client_max_active_total="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_TOTAL:-}"
instances="${BENCH_WORKBENCH_LASM_INSTANCES:-2}"
autoscale_max_instances="${BENCH_WORKBENCH_LASM_AUTOSCALE_MAX_INSTANCES:-4}"
autoscale_target_connections="${BENCH_WORKBENCH_LASM_AUTOSCALE_TARGET_CONNECTIONS:-256}"
autoscale_check_ms="${BENCH_WORKBENCH_LASM_AUTOSCALE_CHECK_MS:-1000}"
cluster_relay_workers="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_WORKERS:-}"
cluster_relay_queue="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_QUEUE:-}"
cluster_accept_workers="${BENCH_WORKBENCH_LASM_CLUSTER_ACCEPT_WORKERS:-}"
cluster_relay_accept_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX:-}"
cluster_relay_pump_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX:-}"
single_runs_out=""
fixed_runs_out=""
proxy_runs_out=""
single_report_out=""
fixed_report_out=""
proxy_report_out=""
out_path=""
dry_run="false"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --endpoints)
      endpoints_csv="${2:-}"
      shift 2
      ;;
    --endpoints=*)
      endpoints_csv="${1#--endpoints=}"
      shift
      ;;
    --port)
      bench_port="${2:-}"
      shift 2
      ;;
    --port=*)
      bench_port="${1#--port=}"
      shift
      ;;
    --lasm-db-adapter)
      lasm_db_adapter="${2:-}"
      shift 2
      ;;
    --lasm-db-adapter=*)
      lasm_db_adapter="${1#--lasm-db-adapter=}"
      shift
      ;;
    --lasm-db-base)
      lasm_db_base="${2:-}"
      shift 2
      ;;
    --lasm-db-base=*)
      lasm_db_base="${1#--lasm-db-base=}"
      shift
      ;;
    --lasm-postgres-dsn-file)
      lasm_postgres_dsn_file="${2:-}"
      shift 2
      ;;
    --lasm-postgres-dsn-file=*)
      lasm_postgres_dsn_file="${1#--lasm-postgres-dsn-file=}"
      shift
      ;;
    --lasm-db-records-capture-enabled)
      lasm_db_records_capture_enabled="${2:-}"
      shift 2
      ;;
    --lasm-db-records-capture-enabled=*)
      lasm_db_records_capture_enabled="${1#--lasm-db-records-capture-enabled=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key)
      lasm_db_postgres_shared_client_max_active_per_key="${2:-}"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key=*)
      lasm_db_postgres_shared_client_max_active_per_key="${1#--lasm-db-postgres-shared-client-max-active-per-key=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-total)
      lasm_db_postgres_shared_client_max_active_total="${2:-}"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-total=*)
      lasm_db_postgres_shared_client_max_active_total="${1#--lasm-db-postgres-shared-client-max-active-total=}"
      shift
      ;;
    --instances)
      instances="${2:-}"
      shift 2
      ;;
    --instances=*)
      instances="${1#--instances=}"
      shift
      ;;
    --autoscale-max-instances)
      autoscale_max_instances="${2:-}"
      shift 2
      ;;
    --autoscale-max-instances=*)
      autoscale_max_instances="${1#--autoscale-max-instances=}"
      shift
      ;;
    --autoscale-target-connections)
      autoscale_target_connections="${2:-}"
      shift 2
      ;;
    --autoscale-target-connections=*)
      autoscale_target_connections="${1#--autoscale-target-connections=}"
      shift
      ;;
    --autoscale-check-ms)
      autoscale_check_ms="${2:-}"
      shift 2
      ;;
    --autoscale-check-ms=*)
      autoscale_check_ms="${1#--autoscale-check-ms=}"
      shift
      ;;
    --cluster-relay-workers)
      cluster_relay_workers="${2:-}"
      shift 2
      ;;
    --cluster-relay-workers=*)
      cluster_relay_workers="${1#--cluster-relay-workers=}"
      shift
      ;;
    --cluster-relay-queue)
      cluster_relay_queue="${2:-}"
      shift 2
      ;;
    --cluster-relay-queue=*)
      cluster_relay_queue="${1#--cluster-relay-queue=}"
      shift
      ;;
    --cluster-accept-workers)
      cluster_accept_workers="${2:-}"
      shift 2
      ;;
    --cluster-accept-workers=*)
      cluster_accept_workers="${1#--cluster-accept-workers=}"
      shift
      ;;
    --cluster-relay-accept-batch-max)
      cluster_relay_accept_batch_max="${2:-}"
      shift 2
      ;;
    --cluster-relay-accept-batch-max=*)
      cluster_relay_accept_batch_max="${1#--cluster-relay-accept-batch-max=}"
      shift
      ;;
    --cluster-relay-pump-batch-max)
      cluster_relay_pump_batch_max="${2:-}"
      shift 2
      ;;
    --cluster-relay-pump-batch-max=*)
      cluster_relay_pump_batch_max="${1#--cluster-relay-pump-batch-max=}"
      shift
      ;;
    --single-runs-out)
      single_runs_out="${2:-}"
      shift 2
      ;;
    --single-runs-out=*)
      single_runs_out="${1#--single-runs-out=}"
      shift
      ;;
    --fixed-runs-out)
      fixed_runs_out="${2:-}"
      shift 2
      ;;
    --fixed-runs-out=*)
      fixed_runs_out="${1#--fixed-runs-out=}"
      shift
      ;;
    --proxy-runs-out)
      proxy_runs_out="${2:-}"
      shift 2
      ;;
    --proxy-runs-out=*)
      proxy_runs_out="${1#--proxy-runs-out=}"
      shift
      ;;
    --single-report-out)
      single_report_out="${2:-}"
      shift 2
      ;;
    --single-report-out=*)
      single_report_out="${1#--single-report-out=}"
      shift
      ;;
    --fixed-report-out)
      fixed_report_out="${2:-}"
      shift 2
      ;;
    --fixed-report-out=*)
      fixed_report_out="${1#--fixed-report-out=}"
      shift
      ;;
    --proxy-report-out)
      proxy_report_out="${2:-}"
      shift 2
      ;;
    --proxy-report-out=*)
      proxy_report_out="${1#--proxy-report-out=}"
      shift
      ;;
    --out)
      out_path="${2:-}"
      shift 2
      ;;
    --out=*)
      out_path="${1#--out=}"
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

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"

if [ -z "$single_runs_out" ]; then
  single_runs_out="${suite_dir}/results/summaries/workbench-lasm-mode-single-runs.json"
fi
if [ -z "$fixed_runs_out" ]; then
  fixed_runs_out="${suite_dir}/results/summaries/workbench-lasm-mode-fixed-runs.json"
fi
if [ -z "$proxy_runs_out" ]; then
  proxy_runs_out="${suite_dir}/results/summaries/workbench-lasm-mode-proxy-runs.json"
fi
if [ -z "$single_report_out" ]; then
  single_report_out="${suite_dir}/results/summaries/workbench-lasm-mode-single-report.json"
fi
if [ -z "$fixed_report_out" ]; then
  fixed_report_out="${suite_dir}/results/summaries/workbench-lasm-mode-fixed-report.json"
fi
if [ -z "$proxy_report_out" ]; then
  proxy_report_out="${suite_dir}/results/summaries/workbench-lasm-mode-proxy-report.json"
fi
if [ -z "$out_path" ]; then
  out_path="${suite_dir}/results/summaries/workbench-lasm-mode-compare.json"
fi

mkdir -p "$(dirname "$single_runs_out")" "$(dirname "$fixed_runs_out")" "$(dirname "$proxy_runs_out")" \
  "$(dirname "$single_report_out")" "$(dirname "$fixed_report_out")" "$(dirname "$proxy_report_out")" \
  "$(dirname "$out_path")"

if [ "$instances" -lt 2 ] 2>/dev/null; then
  echo "--instances must be >= 2 for fixed/proxy comparison modes" >&2
  exit 2
fi
if [ "$autoscale_max_instances" -le "$instances" ] 2>/dev/null; then
  echo "--autoscale-max-instances must be > --instances for proxy mode comparison" >&2
  exit 2
fi
if [ -n "$lasm_db_records_capture_enabled" ]; then
  case "$lasm_db_records_capture_enabled" in
    0|1) ;;
    *)
      echo "--lasm-db-records-capture-enabled must be 0 or 1" >&2
      exit 2
      ;;
  esac
fi

benchmark_matrix_script="${suite_dir}/scripts/run_workbench_benchmark_matrix.sh"
current_report_path="${suite_dir}/results/summaries/sec4-lasm-report.json"

copy_current_report() {
  local out_file="$1"
  if [ ! -f "$current_report_path" ]; then
    echo "missing sec4-lasm report after mode run: ${current_report_path}" >&2
    return 2
  fi
  cp "$current_report_path" "$out_file"
}

run_mode() {
  local mode="$1"
  local runs_out="$2"
  local report_out="$3"
  shift 3
  local -a cmd=(
    "$benchmark_matrix_script"
    --impls "sec4-lasm"
    --endpoints "$endpoints_csv"
    --port "$bench_port"
    --lasm-db-adapter "$lasm_db_adapter"
    --out-runs "$runs_out"
  )

  if [ -n "$lasm_db_base" ]; then
    cmd+=(--lasm-db-base "$lasm_db_base")
  fi
  if [ -n "$lasm_postgres_dsn_file" ]; then
    cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
  fi
  if [ -n "$lasm_db_records_capture_enabled" ]; then
    cmd+=(--lasm-db-records-capture-enabled "$lasm_db_records_capture_enabled")
  fi
  if [ -n "$lasm_db_postgres_shared_client_max_active_per_key" ]; then
    cmd+=(--lasm-db-postgres-shared-client-max-active-per-key "$lasm_db_postgres_shared_client_max_active_per_key")
  fi
  if [ -n "$lasm_db_postgres_shared_client_max_active_total" ]; then
    cmd+=(--lasm-db-postgres-shared-client-max-active-total "$lasm_db_postgres_shared_client_max_active_total")
  fi

  case "$mode" in
    single)
      cmd+=(--lasm-instances 1 --lasm-autoscale-max-instances 1)
      ;;
    fixed)
      cmd+=(--lasm-instances "$instances" --lasm-autoscale-max-instances "$instances")
      ;;
    proxy)
      cmd+=(--lasm-instances "$instances")
      cmd+=(--lasm-autoscale-max-instances "$autoscale_max_instances")
      cmd+=(--lasm-autoscale-target-connections "$autoscale_target_connections")
      cmd+=(--lasm-autoscale-check-ms "$autoscale_check_ms")
      if [ -n "$cluster_relay_workers" ]; then
        cmd+=(--lasm-cluster-relay-workers "$cluster_relay_workers")
      fi
      if [ -n "$cluster_relay_queue" ]; then
        cmd+=(--lasm-cluster-relay-queue "$cluster_relay_queue")
      fi
      if [ -n "$cluster_accept_workers" ]; then
        cmd+=(--lasm-cluster-accept-workers "$cluster_accept_workers")
      fi
      if [ -n "$cluster_relay_accept_batch_max" ]; then
        cmd+=(--lasm-cluster-relay-accept-batch-max "$cluster_relay_accept_batch_max")
      fi
      if [ -n "$cluster_relay_pump_batch_max" ]; then
        cmd+=(--lasm-cluster-relay-pump-batch-max "$cluster_relay_pump_batch_max")
      fi
      ;;
    *)
      echo "unsupported mode: $mode" >&2
      return 2
      ;;
  esac

  if [ "$dry_run" = "true" ]; then
    printf 'run (%s):' "$mode"
    printf ' %q' "${cmd[@]}"
    printf '\n'
    echo "copy report (${mode}): ${current_report_path} -> ${report_out}"
    return 0
  fi

  "${cmd[@]}"
  copy_current_report "$report_out"
}

run_mode single "$single_runs_out" "$single_report_out"
run_mode fixed "$fixed_runs_out" "$fixed_report_out"
run_mode proxy "$proxy_runs_out" "$proxy_report_out"

if [ "$dry_run" = "true" ]; then
  echo "write compare artifact: ${out_path}"
  exit 0
fi

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
  --arg generatedAt "$generated_at" \
  --arg endpoints "$endpoints_csv" \
  --arg lasmDbAdapter "$lasm_db_adapter" \
  --arg lasmDbRecordsCaptureEnabled "$lasm_db_records_capture_enabled" \
  --argjson instances "$instances" \
  --argjson autoscaleMaxInstances "$autoscale_max_instances" \
  --argjson autoscaleTargetConnections "$autoscale_target_connections" \
  --argjson autoscaleCheckMs "$autoscale_check_ms" \
  --arg singleRunsPath "$single_runs_out" \
  --arg fixedRunsPath "$fixed_runs_out" \
  --arg proxyRunsPath "$proxy_runs_out" \
  --arg singleReportPath "$single_report_out" \
  --arg fixedReportPath "$fixed_report_out" \
  --arg proxyReportPath "$proxy_report_out" \
  --slurpfile single "$single_report_out" \
  --slurpfile fixed "$fixed_report_out" \
  --slurpfile proxy "$proxy_report_out" \
  '
  def p99num($v):
    if ($v == null or ($v | tostring) == "") then null
    elif (($v | tostring) | endswith("us")) then (($v | tostring | sub("us$";"") | tonumber) / 1000)
    elif (($v | tostring) | endswith("ms")) then ($v | tostring | sub("ms$";"") | tonumber)
    elif (($v | tostring) | endswith("s")) then (($v | tostring | sub("s$";"") | tonumber) * 1000)
    elif (($v | tostring) | endswith("m")) then (($v | tostring | sub("m$";"") | tonumber) * 60000)
    else null end;

  def mode_rows($report; $mode):
    [($report[0].summaries // [])[] | {
      mode: $mode,
      endpoint: .endpoint,
      targetRps: (.targetRps // 0),
      completedRequests: (.completedRequests // 0),
      requestsPerSec: (.requestsPerSec // 0),
      loadGenerator: (.loadGenerator // "wrk2"),
      constantRate: (.constantRate // true),
      p99: (.latency.p99 // ""),
      p99Ms: p99num(.latency.p99),
      rssKb: (.memory.rssKb // null),
      non2xxOr3xxResponses: (.http.non2xxOr3xxResponses // 0),
      socketErrorsTotal: (
        (.http.socketErrors.connect // 0)
        + (.http.socketErrors.read // 0)
        + (.http.socketErrors.write // 0)
        + (.http.socketErrors.timeout // 0)
      )
    }];

  def mode_summary($rows; $mode; $runsPath; $reportPath):
    ($rows | map(select(.mode == $mode))) as $filtered
    | {
        mode: $mode,
        pass: true,
        runsPath: $runsPath,
        reportPath: $reportPath,
        endpointCount: ($filtered | length),
        observed: {
          requestsPerSec: ($filtered | map(.requestsPerSec) | add),
          avgRequestsPerSec: (
            if ($filtered | length) == 0 then 0
            else (($filtered | map(.requestsPerSec) | add) / ($filtered | length))
            end
          ),
          p99Ms: (
            ($filtered | map(.p99Ms) | map(select(. != null))) as $p99s
            | if ($p99s | length) == 0 then null else (($p99s | add) / ($p99s | length)) end
          ),
          peakRssKb: (
            ($filtered | map(.rssKb) | map(select(. != null))) as $rss
            | if ($rss | length) == 0 then null else ($rss | max) end
          )
        }
      };

  (mode_rows($single; "single") + mode_rows($fixed; "fixed") + mode_rows($proxy; "proxy")) as $rows
  | [
      ($rows | map(.endpoint) | unique[]) as $endpoint
      | {
          endpoint: $endpoint,
          compared: (
            $rows
            | map(select(.endpoint == $endpoint))
            | sort_by([-(.requestsPerSec // 0), (.p99Ms // 1e18), (.rssKb // 1e18), .mode])
          )
        }
        | .leader = (.compared[0] // null)
    ] as $endpointRows
  | (
      $endpointRows
      | reduce .[] as $row ({single: 0, fixed: 0, proxy: 0};
          if ($row.leader.mode // "") == "" then .
          else .[$row.leader.mode] += 1
          end
        )
    ) as $modeWins
  | [
      mode_summary($rows; "single"; $singleRunsPath; $singleReportPath),
      mode_summary($rows; "fixed"; $fixedRunsPath; $fixedReportPath),
      mode_summary($rows; "proxy"; $proxyRunsPath; $proxyReportPath)
    ] as $modeSummaries
  | (
      $modeSummaries
      | map(. + {winCount: ($modeWins[.mode] // 0)})
      | sort_by([-(.winCount), -(.observed.requestsPerSec // 0), (.observed.p99Ms // 1e18), (.observed.peakRssKb // 1e18), .mode])
      | .[0]
    ) as $recommended
  | {
      version: "0.1",
      generatedAt: $generatedAt,
      config: {
        endpoints: ($endpoints | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
        lasmDbAdapter: $lasmDbAdapter,
        lasmDbRecordsCaptureEnabled: $lasmDbRecordsCaptureEnabled,
        instances: $instances,
        autoscaleMaxInstances: $autoscaleMaxInstances,
        autoscaleTargetConnections: $autoscaleTargetConnections,
        autoscaleCheckMs: $autoscaleCheckMs
      },
      single: ($modeSummaries[] | select(.mode == "single")),
      fixed: ($modeSummaries[] | select(.mode == "fixed")),
      proxy: ($modeSummaries[] | select(.mode == "proxy")),
      endpoints: $endpointRows,
      comparison: {
        recommendedMode: $recommended.mode,
        recommendedReason: ("wins=" + (($recommended.winCount // 0) | tostring) + ", totalRps=" + (($recommended.observed.requestsPerSec // 0) | tostring)),
        modeWins: $modeWins,
        totalRequestsPerSec: {
          single: (($modeSummaries[] | select(.mode == "single") | .observed.requestsPerSec) // 0),
          fixed: (($modeSummaries[] | select(.mode == "fixed") | .observed.requestsPerSec) // 0),
          proxy: (($modeSummaries[] | select(.mode == "proxy") | .observed.requestsPerSec) // 0)
        },
        averageP99Ms: {
          single: (($modeSummaries[] | select(.mode == "single") | .observed.p99Ms) // null),
          fixed: (($modeSummaries[] | select(.mode == "fixed") | .observed.p99Ms) // null),
          proxy: (($modeSummaries[] | select(.mode == "proxy") | .observed.p99Ms) // null)
        },
        requestsPerSecDelta: (
          (($modeSummaries[] | select(.mode == "fixed") | .observed.requestsPerSec) // 0)
          - (($modeSummaries[] | select(.mode == "proxy") | .observed.requestsPerSec) // 0)
        ),
        requestsPerSecGainPctVsProxy: (
          (($modeSummaries[] | select(.mode == "proxy") | .observed.requestsPerSec) // 0) as $proxyRps
          | if $proxyRps == 0 then null
            else (
              ((($modeSummaries[] | select(.mode == "fixed") | .observed.requestsPerSec) // 0) - $proxyRps) / $proxyRps * 100
            )
            end
        ),
        p99ProxyMs: (($modeSummaries[] | select(.mode == "proxy") | .observed.p99Ms) // null),
        p99FixedMs: (($modeSummaries[] | select(.mode == "fixed") | .observed.p99Ms) // null),
        peakRssDeltaKb: (
          (($modeSummaries[] | select(.mode == "fixed") | .observed.peakRssKb) // null) as $fixedRss
          | (($modeSummaries[] | select(.mode == "proxy") | .observed.peakRssKb) // null) as $proxyRss
          | if ($fixedRss == null or $proxyRss == null) then null else ($fixedRss - $proxyRss) end
        ),
        relayDispatchShortCircuitTotalProxy: null,
        relayDispatchShortCircuitTotalFixed: null,
        relayDispatchShortCircuitTotalDelta: null,
        relayLiveSenderCountProxy: null,
        relayLiveSenderCountFixed: null,
        relayLiveSenderCountDelta: null
      }
    }
  ' >"$out_path"

echo "wrote ${out_path}"
