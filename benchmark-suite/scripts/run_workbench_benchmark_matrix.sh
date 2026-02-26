#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--matrix path] [--impls sec4,sec4-lasm,node,go,rust]
          [--endpoints wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list]
          [--port <n>] [--out-runs path] [--out-compare path] [--out-analysis path] [--out-report path]

Runs workbench benchmark profiles across implemented matrix lanes and emits:
1) per-impl endpoint summaries
2) per-impl report bundles
3) cross-impl compare matrix + analysis + markdown report
USAGE
}

dry_run="false"
matrix_path=""
impls_csv=""
endpoints_csv="wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list"
bench_port="${BENCH_WORKBENCH_PORT:-18093}"
out_runs=""
out_compare=""
out_analysis=""
out_report=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    --matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      impls_csv="$2"
      shift 2
      ;;
    --impls=*)
      impls_csv="${1#--impls=}"
      shift
      ;;
    --endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      endpoints_csv="$2"
      shift 2
      ;;
    --endpoints=*)
      endpoints_csv="${1#--endpoints=}"
      shift
      ;;
    --port)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      bench_port="$2"
      shift 2
      ;;
    --port=*)
      bench_port="${1#--port=}"
      shift
      ;;
    --out-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_runs="$2"
      shift 2
      ;;
    --out-runs=*)
      out_runs="${1#--out-runs=}"
      shift
      ;;
    --out-compare)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_compare="$2"
      shift 2
      ;;
    --out-compare=*)
      out_compare="${1#--out-compare=}"
      shift
      ;;
    --out-analysis)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_analysis="$2"
      shift 2
      ;;
    --out-analysis=*)
      out_analysis="${1#--out-analysis=}"
      shift
      ;;
    --out-report)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_report="$2"
      shift 2
      ;;
    --out-report=*)
      out_report="${1#--out-report=}"
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
repo_root="$(cd "$suite_dir/.." && pwd)"

if [ -z "$matrix_path" ]; then
  matrix_path="${suite_dir}/workbench/matrix.backends.json"
fi
if [ -z "$out_runs" ]; then
  out_runs="${suite_dir}/results/summaries/workbench-benchmark-runs.json"
fi
if [ -z "$out_compare" ]; then
  out_compare="${suite_dir}/results/summaries/workbench-benchmark-compare-matrix.json"
fi
if [ -z "$out_analysis" ]; then
  out_analysis="${suite_dir}/results/summaries/workbench-benchmark-analysis.json"
fi
if [ -z "$out_report" ]; then
  out_report="${suite_dir}/results/workbench-benchmark-report.md"
fi

if [ ! -f "$matrix_path" ]; then
  echo "workbench matrix missing: $matrix_path" >&2
  exit 2
fi

mkdir -p "${suite_dir}/results/raw" "${suite_dir}/results/summaries" "$(dirname "$out_report")"

if [ -n "$impls_csv" ]; then
  selected_impls_json="$(
    IFS=',' read -r -a impls <<<"$impls_csv"
    json='[]'
    for raw_impl in "${impls[@]}"; do
      impl="$(echo "$raw_impl" | tr -d '[:space:]')"
      [ -z "$impl" ] && continue
      json="$(jq -c --arg impl "$impl" '. + [$impl]' <<<"$json")"
    done
    printf '%s' "$json"
  )"
else
  selected_impls_json='[]'
fi

is_selected_impl() {
  local impl="$1"
  if [ "$selected_impls_json" = "[]" ]; then
    return 0
  fi
  jq -e --arg impl "$impl" 'index($impl) != null' <<<"$selected_impls_json" >/dev/null
}

supported_endpoint() {
  case "$1" in
    wb-tasks-post|wb-tasks-with-comment|wb-task-comment-post|wb-task-get|wb-tasks-list)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

IFS=',' read -r -a endpoints <<<"$endpoints_csv"
if [ "${#endpoints[@]}" -eq 0 ]; then
  echo "no workbench endpoints provided" >&2
  exit 2
fi
for raw_endpoint in "${endpoints[@]}"; do
  endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
  [ -z "$endpoint" ] && continue
  if ! supported_endpoint "$endpoint"; then
    echo "unsupported workbench endpoint: $endpoint" >&2
    exit 2
  fi
done

runnable_rows_json='[]'
while IFS= read -r impl_row; do
  impl="$(jq -r '.impl' <<<"$impl_row")"
  status="$(jq -r '.status' <<<"$impl_row")"
  service_rel="$(jq -r '.servicePath' <<<"$impl_row")"
  if [ "$status" != "implemented-alpha" ] && [ "$status" != "implemented" ]; then
    continue
  fi
  if ! is_selected_impl "$impl"; then
    continue
  fi
  runnable_rows_json="$(jq -c --argjson row "$impl_row" '. + [$row]' <<<"$runnable_rows_json")"
done < <(jq -c '.implementations[]' "$matrix_path")

if [ "$(jq 'length' <<<"$runnable_rows_json")" -eq 0 ]; then
  echo "no runnable workbench implementations selected from matrix" >&2
  exit 2
fi

runnable_impls_csv="$(
  jq -r '.[].impl' <<<"$runnable_rows_json" | paste -sd, -
)"

if [ "$dry_run" = "true" ]; then
  "${suite_dir}/scripts/preflight.sh" --impls "$runnable_impls_csv" --dry-run-only
else
  "${suite_dir}/scripts/preflight.sh" --impls "$runnable_impls_csv"
fi

base_url="http://127.0.0.1:${bench_port}"
pg_dsn="${BENCH_WORKBENCH_PG_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-postgresql://127.0.0.1:5432/postgres?sslmode=disable}}"

service_pid=""
service_log=""
service_temp_dir=""
cleanup_temp_dir="false"

cleanup_impl() {
  if [ -n "$service_pid" ] && kill -0 "$service_pid" >/dev/null 2>&1; then
    kill "$service_pid" >/dev/null 2>&1 || true
    wait "$service_pid" >/dev/null 2>&1 || true
  fi
  service_pid=""
  if [ "$cleanup_temp_dir" = "true" ] && [ -n "$service_temp_dir" ] && [ -d "$service_temp_dir" ]; then
    rm -rf "$service_temp_dir"
  fi
  service_temp_dir=""
  cleanup_temp_dir="false"
}

start_impl_service() {
  local impl="$1"
  local service_abs="$2"
  local log_file="$3"

  service_temp_dir=""
  cleanup_temp_dir="false"
  service_log="$log_file"

  case "$impl" in
    sec4)
      service_temp_dir="$(mktemp -d "/tmp/sec4-workbench-bench-db.XXXXXX")"
      cleanup_temp_dir="true"
      (
        cd "$repo_root"
        SEC4_RT_DB_BASE="$service_temp_dir" cargo run -q -p sec4 -- run \
          --path "$service_abs" \
          --backend c \
          --port "$bench_port" \
          --serve-timeout-ms 20000
      ) >"$log_file" 2>&1 &
      ;;
    sec4-lasm)
      service_temp_dir="$(mktemp -d "/tmp/sec4-lasm-workbench-bench-db.XXXXXX")"
      cleanup_temp_dir="true"
      (
        cd "$repo_root"
        cargo run -q -p sec4 -- run \
          --path "$service_abs" \
          --backend lasm \
          --db-adapter sqlite \
          --db-base "$service_temp_dir" \
          --port "$bench_port" \
          --serve-timeout-ms 20000
      ) >"$log_file" 2>&1 &
      ;;
    node)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$bench_port" node server.mjs
      ) >"$log_file" 2>&1 &
      ;;
    go)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$bench_port" go run .
      ) >"$log_file" 2>&1 &
      ;;
    rust)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$bench_port" cargo run --quiet
      ) >"$log_file" 2>&1 &
      ;;
    *)
      echo "unsupported workbench impl runtime: $impl" >&2
      return 2
      ;;
  esac

  service_pid="$!"
}

wait_ready() {
  local pid="$1"
  local tries=180
  while [ "$tries" -gt 0 ]; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      return 2
    fi
    if curl -fsS "${base_url}/health" >/tmp/workbench-bench-health.txt 2>/dev/null; then
      if [ "$(cat /tmp/workbench-bench-health.txt 2>/dev/null || true)" = "ok" ]; then
        return 0
      fi
    fi
    tries=$((tries - 1))
    sleep 0.1
  done
  return 1
}

seed_impl_state() {
  local impl="$1"
  local auth_header='Authorization: Bearer token123'
  local run_id
  run_id="$(date +%s%N)"

  local setup_out="/tmp/workbench-bench-${impl}-setup.json"
  local setup_status
  setup_status="$(curl -sS -o "$setup_out" -w '%{http_code}' -X POST -H "$auth_header" "${base_url}/wb/setup")"
  if [ "$setup_status" != "200" ]; then
    echo "workbench setup failed impl=${impl} status=${setup_status}" >&2
    return 1
  fi

  local seed_task_id="${impl}-wb-seed-task-${run_id}"
  local seed_comment_id="${impl}-wb-seed-comment-${run_id}"
  local seed_task_params
  local seed_task_params_uri
  local seed_task_status
  local seed_task_out="/tmp/workbench-bench-${impl}-seed-task.json"
  seed_task_params="$(jq -nc \
    --arg id "$seed_task_id" \
    --arg title "SeedTask" \
    --arg description "workbench benchmark seed" \
    --arg status "open" \
    --argjson priority 3 \
    --argjson created 1700000000000 \
    '[ $id, $title, $description, $status, $priority, $created ]')"
  seed_task_params_uri="$(printf '%s' "$seed_task_params" | jq -sRr @uri)"
  seed_task_status="$(curl -sS -o "$seed_task_out" -w '%{http_code}' \
    -X POST -H "$auth_header" \
    "${base_url}/wb/tasks?params=${seed_task_params_uri}")"
  case "$seed_task_status" in
    200|201) ;;
    *)
      echo "workbench seed task failed impl=${impl} status=${seed_task_status}" >&2
      return 1
      ;;
  esac

  local seed_comment_params
  local seed_comment_params_uri
  local seed_comment_status
  local seed_comment_out="/tmp/workbench-bench-${impl}-seed-comment.json"
  seed_comment_params="$(jq -nc \
    --arg id "$seed_comment_id" \
    --arg task_id "$seed_task_id" \
    --arg body "seed comment" \
    --argjson created 1700000000001 \
    '[ $id, $task_id, $body, $created ]')"
  seed_comment_params_uri="$(printf '%s' "$seed_comment_params" | jq -sRr @uri)"
  seed_comment_status="$(curl -sS -o "$seed_comment_out" -w '%{http_code}' \
    -X POST -H "$auth_header" \
    "${base_url}/wb/tasks/${seed_task_id}/comments?params=${seed_comment_params_uri}")"
  case "$seed_comment_status" in
    200|201) ;;
    *)
      echo "workbench seed comment failed impl=${impl} status=${seed_comment_status}" >&2
      return 1
      ;;
  esac

  printf '%s' "$seed_task_id"
}

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
runs_json='[]'
total_passed=0
total_failed=0
total_skipped=0
passed_impls_csv=""

while IFS= read -r impl_row; do
  impl="$(jq -r '.impl' <<<"$impl_row")"
  status="$(jq -r '.status' <<<"$impl_row")"
  service_rel="$(jq -r '.servicePath' <<<"$impl_row")"
  service_abs="${repo_root}/${service_rel}"
  log_file="${suite_dir}/results/raw/${impl}-workbench-service.log"

  if [ "$dry_run" = "true" ]; then
    echo "start: impl=${impl} servicePath=${service_rel} port=${bench_port}"
    echo "run: ${suite_dir}/scripts/run_workbench_profile.sh --dry-run ${impl} <endpoint> ${base_url}"
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
      [ -z "$endpoint" ] && continue
      echo "run: ${suite_dir}/scripts/run_workbench_profile.sh --dry-run ${impl} ${endpoint} ${base_url}"
    done
    echo "run: ${suite_dir}/scripts/build_report.sh ${impl} ${suite_dir}/results ${suite_dir}/results/summaries/${impl}-report.json \"\" ${endpoints_csv}"
    continue
  fi

  result="passed"
  reason=""
  exit_code=0
  seed_task_id=""

  start_impl_service "$impl" "$service_abs" "$log_file" || {
    result="failed"
    reason="failed to start service process"
    exit_code=1
  }

  if [ "$result" = "passed" ]; then
    if ! wait_ready "$service_pid"; then
      result="failed"
      reason="service failed readiness on ${base_url}/health"
      exit_code=1
    fi
  fi

  if [ "$result" = "passed" ]; then
    seed_task_id="$(seed_impl_state "$impl" || true)"
    if [ -z "$seed_task_id" ]; then
      result="failed"
      reason="failed to setup/seed workbench state"
      exit_code=1
    fi
  fi

  if [ "$result" = "passed" ]; then
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
      [ -z "$endpoint" ] && continue
      run_tag="${impl}-${endpoint}-$(date +%s%N)"
      if ! BENCH_SERVER_PID="$service_pid" BENCH_WB_TASK_ID="$seed_task_id" BENCH_WB_RUN_TAG="$run_tag" \
        "${suite_dir}/scripts/run_workbench_profile.sh" "$impl" "$endpoint" "$base_url"; then
        result="failed"
        reason="profile failed endpoint=${endpoint}"
        exit_code=1
        break
      fi
    done
  fi

  if [ "$result" = "passed" ]; then
    if ! "${suite_dir}/scripts/build_report.sh" "$impl" "${suite_dir}/results" "${suite_dir}/results/summaries/${impl}-report.json" "" "$endpoints_csv"; then
      result="failed"
      reason="failed to build report bundle"
      exit_code=1
    fi
  fi

  if [ "$result" = "passed" ]; then
    total_passed=$((total_passed + 1))
    if [ -z "$passed_impls_csv" ]; then
      passed_impls_csv="$impl"
    else
      passed_impls_csv="${passed_impls_csv},${impl}"
    fi
  else
    total_failed=$((total_failed + 1))
    if [ -f "$log_file" ]; then
      echo "--- ${impl} workbench log tail ---" >&2
      tail -n 30 "$log_file" >&2 || true
      echo "--- end log tail ---" >&2
    fi
  fi

  run_row="$(jq -nc \
    --arg impl "$impl" \
    --arg status "$status" \
    --arg servicePath "$service_rel" \
    --arg result "$result" \
    --arg reason "$reason" \
    --argjson exitCode "$exit_code" \
    --arg endpoints "$endpoints_csv" \
    '{
      impl: $impl,
      status: $status,
      servicePath: $servicePath,
      benchmarkResult: $result,
      reason: (if $reason == "" then null else $reason end),
      exitCode: $exitCode,
      endpoints: ($endpoints | split(","))
    }')"
  runs_json="$(jq -c --argjson row "$run_row" '. + [$row]' <<<"$runs_json")"

  cleanup_impl
done < <(jq -c '.[]' <<<"$runnable_rows_json")

if [ "$dry_run" = "true" ]; then
  echo "run: ${suite_dir}/scripts/compare_matrix.sh ${suite_dir}/results/summaries ${out_compare} ${runnable_impls_csv}"
  echo "run: ${suite_dir}/scripts/analyze_matrix.sh ${out_compare} ${out_analysis}"
  echo "run: ${suite_dir}/scripts/publish_report.sh ${out_compare} ${out_report} \"\" ${out_analysis}"
  exit 0
fi

if [ -z "$passed_impls_csv" ]; then
  total_skipped=$((total_skipped + 0))
fi

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
mkdir -p "$(dirname "$out_runs")"
jq -n \
  --arg version "0.1" \
  --arg startedAt "$started_at" \
  --arg finishedAt "$finished_at" \
  --arg matrixPath "$matrix_path" \
  --arg endpoints "$endpoints_csv" \
  --arg comparePath "$out_compare" \
  --arg analysisPath "$out_analysis" \
  --arg reportPath "$out_report" \
  --argjson totals "$(jq -nc --argjson passed "$total_passed" --argjson failed "$total_failed" --argjson skipped "$total_skipped" '{passed:$passed,failed:$failed,skipped:$skipped}')" \
  --argjson runs "$runs_json" \
  '{
    version: $version,
    startedAt: $startedAt,
    finishedAt: $finishedAt,
    matrixPath: $matrixPath,
    endpoints: ($endpoints | split(",")),
    comparePath: $comparePath,
    analysisPath: $analysisPath,
    reportPath: $reportPath,
    totals: $totals,
    runs: $runs
  }' >"$out_runs"

if [ -n "$passed_impls_csv" ]; then
  "${suite_dir}/scripts/compare_matrix.sh" "${suite_dir}/results/summaries" "$out_compare" "$passed_impls_csv"
  "${suite_dir}/scripts/analyze_matrix.sh" "$out_compare" "$out_analysis"
  "${suite_dir}/scripts/publish_report.sh" "$out_compare" "$out_report" "" "$out_analysis"
fi

echo "wrote workbench benchmark run summary: $out_runs"
if [ -n "$passed_impls_csv" ]; then
  echo "wrote workbench compare matrix: $out_compare"
  echo "wrote workbench analysis: $out_analysis"
  echo "wrote workbench report: $out_report"
fi
echo "totals: passed=${total_passed} failed=${total_failed} skipped=${total_skipped}"

if [ "$total_failed" -gt 0 ]; then
  exit 1
fi
