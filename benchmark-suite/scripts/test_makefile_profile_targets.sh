#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
makefile_path="${root_dir}/Makefile"

require_target_uses_profile() {
  local target="$1"
  local expected_endpoint="$2"

  if ! awk -v t="${target}" -v ep="${expected_endpoint}" '
    $0 ~ "^" t ":$" { in_target=1; next }
    in_target == 1 {
      if ($0 ~ /^[^[:space:]]/ && $0 !~ /^#/) { in_target=0 }
      if ($0 ~ "scripts/run_profile.sh[[:space:]]+\\$\\(IMPL\\)[[:space:]]+" ep "[[:space:]]+\\$\\(BASE_URL\\)") {
        found=1
      }
    }
    END { exit(found ? 0 : 1) }
  ' "${makefile_path}"; then
    echo "${target} target must invoke scripts/run_profile.sh for endpoint ${expected_endpoint}" >&2
    exit 1
  fi
}

require_target_uses_profile "bench-ping" "ping"
require_target_uses_profile "bench-decode" "decode"
require_target_uses_profile "bench-users" "users-post"
require_target_uses_profile "bench-users-get" "users-get"

extract_target_block() {
  local target="$1"
  awk -v t="${target}" '
    $0 ~ "^" t ":$" { in_target=1; next }
    in_target == 1 {
      if ($0 ~ /^[^[:space:]]/ && $0 !~ /^#/) {
        in_target=0
      } else {
        print
      }
    }
  ' "${makefile_path}"
}

require_target_contains_token() {
  local target="$1"
  local token="$2"
  local block
  block="$(extract_target_block "${target}")"
  if ! grep -q -- "${token}" <<<"${block}"; then
    echo "${target} target must include token: ${token}" >&2
    exit 1
  fi
}

require_target_contains_token "bench-full-saturation" "--include-lasm-saturation"
require_target_contains_token "bench-full-saturation" "--saturation-boost-steps"
require_target_contains_token "bench-full-saturation" "--saturation-profile"
require_target_contains_token "bench-full-saturation" '$(FULL_SATURATION_WARMUP_PATH_FLAG)'
require_target_contains_token "bench-full-saturation" "--saturation-project-path"
require_target_contains_token "bench-full-saturation" "--saturation-target-requests"
require_target_contains_token "bench-full-saturation" "--saturation-duration"
require_target_contains_token "bench-full-saturation" "--saturation-threads"
require_target_contains_token "bench-full-saturation" "--saturation-connections"
require_target_contains_token "bench-full-saturation" "--saturation-cluster-relay-workers"
require_target_contains_token "bench-full-saturation" "--saturation-cluster-relay-queue"
require_target_contains_token "bench-full-saturation" "--saturation-cluster-accept-workers"
require_target_contains_token "bench-full-saturation" "--saturation-cluster-relay-accept-batch-max"
require_target_contains_token "bench-full-saturation" "--saturation-cluster-relay-pump-batch-max"
require_target_contains_token "bench-full-saturation" '$(SATURATION_FIXED_REUSE_PORT_FLAG)'
require_target_contains_token "bench-full-saturation" '$(LASM_MODE_COMPARE_FLAG)'
require_target_contains_token "bench-full-saturation-dry" "--dry-run"
require_target_contains_token "bench-full-saturation-dry" "--include-lasm-saturation"
require_target_contains_token "bench-full-saturation-dry" "--saturation-profile"
require_target_contains_token "bench-full-saturation-dry" '$(FULL_SATURATION_WARMUP_PATH_FLAG)'
require_target_contains_token "bench-full-saturation-dry" '$(LASM_MODE_COMPARE_FLAG)'
require_target_contains_token "bench-full" '$(LASM_MODE_COMPARE_FLAG)'
require_target_contains_token "bench-full-dry" '$(LASM_MODE_COMPARE_FLAG)'
require_target_contains_token "bench-full-saturation-throughput" '$(MAKE) bench-full-saturation'
require_target_contains_token "bench-full-saturation-throughput" 'FULL_SATURATION_SKIP_VERIFY=true'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_THROUGHPUT_BOOST_STEPS)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_PROFILE="$(LASM_SATURATION_THROUGHPUT_PROFILE)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_THROUGHPUT_WARMUP_PATH)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_THROUGHPUT_RELAY_WORKERS)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_THROUGHPUT_RELAY_QUEUE)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_THROUGHPUT_RELAY_PUMP_BATCH_MAX)"'
require_target_contains_token "bench-full-saturation-throughput" 'LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_THROUGHPUT_FIXED_REUSE_PORT_MODE)"'
require_target_contains_token "bench-full-saturation-throughput-dry" '$(MAKE) bench-full-saturation-dry'
require_target_contains_token "bench-full-saturation-throughput-dry" 'FULL_SATURATION_SKIP_VERIFY=true'
require_target_contains_token "bench-full-saturation-latency" '$(MAKE) bench-full-saturation'
require_target_contains_token "bench-full-saturation-latency" 'FULL_SATURATION_SKIP_VERIFY=true'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_BOOST_STEPS="$(LASM_SATURATION_LATENCY_BOOST_STEPS)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_PROFILE="$(LASM_SATURATION_LATENCY_PROFILE)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_WARMUP_PATH="$(LASM_SATURATION_LATENCY_WARMUP_PATH)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_CLUSTER_RELAY_WORKERS="$(LASM_SATURATION_LATENCY_RELAY_WORKERS)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_CLUSTER_RELAY_QUEUE="$(LASM_SATURATION_LATENCY_RELAY_QUEUE)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX="$(LASM_SATURATION_LATENCY_RELAY_PUMP_BATCH_MAX)"'
require_target_contains_token "bench-full-saturation-latency" 'LASM_CAPACITY_FIXED_REUSE_PORT_MODE="$(LASM_SATURATION_LATENCY_FIXED_REUSE_PORT_MODE)"'
require_target_contains_token "bench-full-saturation-latency-dry" '$(MAKE) bench-full-saturation-dry'
require_target_contains_token "bench-full-saturation-latency-dry" 'FULL_SATURATION_SKIP_VERIFY=true'
require_target_contains_token "bench-full-saturation-presets" '$(MAKE) bench-full-saturation-throughput'
require_target_contains_token "bench-full-saturation-presets" '$(MAKE) bench-full-saturation-latency'
require_target_contains_token "bench-full-saturation-presets-dry" '$(MAKE) bench-full-saturation-throughput-dry'
require_target_contains_token "bench-full-saturation-presets-dry" '$(MAKE) bench-full-saturation-latency-dry'
require_target_contains_token "lasm-cluster-mode-compare" "run_lasm_cluster_mode_compare.sh"
require_target_contains_token "workbench-full-bench" "run_workbench_full_benchmark_suite.sh"
require_target_contains_token "workbench-full-bench" '--impls "$(WORKBENCH_IMPLS)"'
require_target_contains_token "workbench-full-bench" '--endpoints "$(WORKBENCH_ENDPOINTS)"'
require_target_contains_token "workbench-full-bench-dry" "--dry-run"
require_target_contains_token "workbench-full-bench-repeats" "run_workbench_full_benchmark_suite_repeats.sh"
require_target_contains_token "workbench-full-bench-repeats" '--runs "$(WORKBENCH_REPEAT_RUNS)"'
require_target_contains_token "workbench-full-bench-repeats-dry" "--dry-run"
require_target_contains_token "workbench-full-bench-repeats-report" "render_workbench_full_benchmark_suite_repeats_summary.sh"
require_target_contains_token "workbench-full-bench-repeats-report" '"$(WORKBENCH_FULL_REPEATS_SUMMARY)"'
require_target_contains_token "workbench-full-bench-repeats-report" '"$(WORKBENCH_FULL_REPEATS_REPORT)"'
require_target_contains_token "workbench-full-bench-local" "run_workbench_full_benchmark_suite_local.sh"
require_target_contains_token "workbench-full-bench-local" '$(BENCH_LOCAL_POSTGRES_KEEP_UP_FLAG)'
require_target_contains_token "workbench-full-bench-local" '$(BENCH_LOCAL_POSTGRES_RESET_FLAG)'
require_target_contains_token "workbench-full-bench-local-dry" "run_workbench_full_benchmark_suite_local.sh"
require_target_contains_token "workbench-full-bench-local-dry" "--dry-run"
require_target_contains_token "workbench-full-bench-local-repeats" "run_workbench_full_benchmark_suite_local_repeats.sh"
require_target_contains_token "workbench-full-bench-local-repeats" '--runs "$(WORKBENCH_REPEAT_RUNS)"'
require_target_contains_token "workbench-full-bench-local-repeats-dry" "run_workbench_full_benchmark_suite_local_repeats.sh"
require_target_contains_token "workbench-full-bench-local-repeats-dry" "--dry-run"
require_target_contains_token "workbench-full-bench-local-bundle" "run_workbench_full_benchmark_suite_local_bundle.sh"
require_target_contains_token "workbench-full-bench-local-bundle" '--out-summary "$(WORKBENCH_FULL_REPEATS_SUMMARY)"'
require_target_contains_token "workbench-full-bench-local-bundle" '--out-report "$(WORKBENCH_FULL_REPEATS_REPORT)"'
require_target_contains_token "workbench-full-bench-local-bundle-dry" "run_workbench_full_benchmark_suite_local_bundle.sh"
require_target_contains_token "workbench-full-bench-local-bundle-dry" "--dry-run"
require_target_contains_token "lasm-cluster-capacity-probe" '--profile "$(LASM_CAPACITY_PROFILE)"'
require_target_contains_token "lasm-cluster-capacity-probe" '$(SATURATION_WARMUP_PATH_FLAG)'
require_target_contains_token "lasm-cluster-saturation-boost-matrix" '--profile "$(LASM_CAPACITY_PROFILE)"'
require_target_contains_token "lasm-cluster-saturation-boost-matrix" '$(SATURATION_WARMUP_PATH_FLAG)'
require_target_contains_token "lasm-cluster-saturation-boost-verify" '--profile "$(LASM_CAPACITY_PROFILE)"'
require_target_contains_token "lasm-cluster-saturation-boost-verify" '$(SATURATION_WARMUP_PATH_FLAG)'
require_target_contains_token "lasm-cluster-saturation-boost-bundle" '--profile "$(LASM_CAPACITY_PROFILE)"'
require_target_contains_token "lasm-cluster-saturation-boost-bundle" '$(SATURATION_WARMUP_PATH_FLAG)'

echo "makefile profile target test passed"
