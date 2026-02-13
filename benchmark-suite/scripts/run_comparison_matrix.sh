#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls node,go,rust,c] [--sec-audit path]

Runs benchmark profiles for each implementation, builds per-impl reports,
then emits compare-matrix and markdown report artifacts.
USAGE
}

dry_run="false"
impls_csv="node,go,rust"
sec_audit_path=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
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
    --sec-audit)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      sec_audit_path="$2"
      shift 2
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
base_url="http://127.0.0.1:8080"
results_dir="${root_dir}/results"
summaries_dir="${results_dir}/summaries"
mkdir -p "$summaries_dir"

if [ -z "$sec_audit_path" ]; then
  candidate="${root_dir}/../baselines/sec-audit/default-secure-prod.hello.json"
  if [ -f "$candidate" ]; then
    sec_audit_path="$candidate"
  fi
fi

IFS=',' read -r -a impls <<< "$impls_csv"
if [ "${#impls[@]}" -eq 0 ]; then
  echo "no implementations provided" >&2
  exit 2
fi

is_supported_impl() {
  case "$1" in
    node|go|rust|c)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

for raw_impl in "${impls[@]}"; do
  normalized="${raw_impl// /}"
  [ -z "$normalized" ] && continue
  if ! is_supported_impl "$normalized"; then
    echo "unsupported implementation for orchestrator: ${normalized}" >&2
    exit 2
  fi
done

start_service() {
  local impl="$1"
  local service_dir="${root_dir}/services/${impl}"
  if [ ! -d "$service_dir" ]; then
    echo "service directory not found for impl=${impl}: ${service_dir}" >&2
    return 2
  fi

  case "$impl" in
    node)
      (
        cd "$service_dir"
        PORT=8080 npm run start
      ) &
      ;;
    go)
      (
        cd "$service_dir"
        PORT=8080 go run .
      ) &
      ;;
    rust)
      (
        cd "$service_dir"
        PORT=8080 cargo run --quiet
      ) &
      ;;
    c)
      (
        cd "$service_dir"
        cc -O2 -std=c11 server.c -o c-bench-server
        PORT=8080 ./c-bench-server
      ) &
      ;;
  esac

  echo $!
}

wait_for_ready() {
  local tries=120
  while [ "$tries" -gt 0 ]; do
    if curl -fsS "${base_url}/ping" >/dev/null 2>&1; then
      return 0
    fi
    tries=$((tries - 1))
    sleep 0.25
  done
  return 1
}

for impl in "${impls[@]}"; do
  impl="${impl// /}"
  [ -z "$impl" ] && continue

  echo "=== impl=${impl} ==="

  if [ "$dry_run" = "true" ]; then
    echo "start: ${impl} service on :8080"
    echo "run: ${root_dir}/scripts/run_profile.sh --dry-run ${impl} ping ${base_url}"
    echo "run: ${root_dir}/scripts/run_profile.sh --dry-run ${impl} decode ${base_url}"
    echo "run: ${root_dir}/scripts/run_profile.sh --dry-run ${impl} users-post ${base_url}"
    echo "run: ${root_dir}/scripts/build_report.sh ${impl} ${results_dir} ${summaries_dir}/${impl}-report.json"
    continue
  fi

  pid="$(start_service "$impl")"
  cleanup_impl() {
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  }
  trap cleanup_impl EXIT

  if ! wait_for_ready; then
    echo "service failed readiness check for impl=${impl}" >&2
    cleanup_impl
    trap - EXIT
    exit 1
  fi

  "${root_dir}/scripts/run_profile.sh" "$impl" ping "$base_url"
  "${root_dir}/scripts/run_profile.sh" "$impl" decode "$base_url"
  "${root_dir}/scripts/run_profile.sh" "$impl" users-post "$base_url"
  "${root_dir}/scripts/build_report.sh" "$impl" "$results_dir" "${summaries_dir}/${impl}-report.json"

  cleanup_impl
  trap - EXIT
done

matrix_path="${summaries_dir}/compare-matrix.json"
analysis_path="${summaries_dir}/analysis.json"
report_md_path="${results_dir}/benchmark-report.md"

if [ "$dry_run" = "true" ]; then
  echo "run: ${root_dir}/scripts/compare_matrix.sh ${summaries_dir} ${matrix_path}"
  echo "run: ${root_dir}/scripts/analyze_matrix.sh ${matrix_path} ${analysis_path}"
  if [ -n "$sec_audit_path" ]; then
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} ${sec_audit_path} ${analysis_path}"
  else
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} \"\" ${analysis_path}"
  fi
  exit 0
fi

"${root_dir}/scripts/compare_matrix.sh" "$summaries_dir" "$matrix_path"
"${root_dir}/scripts/analyze_matrix.sh" "$matrix_path" "$analysis_path"

if [ -n "$sec_audit_path" ]; then
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "$sec_audit_path" "$analysis_path"
else
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "" "$analysis_path"
fi

echo "comparison matrix written to ${matrix_path}"
echo "analysis written to ${analysis_path}"
echo "benchmark report written to ${report_md_path}"
