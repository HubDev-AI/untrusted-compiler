#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--test-file <path>]

Validates that M16 runtime HTTP e2e coverage tests remain present.
USAGE
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
test_file="${repo_root}/compiler/sec4-cli/tests/json_output.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --test-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      test_file="$2"
      shift 2
      ;;
    --test-file=*)
      test_file="${1#--test-file=}"
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

if [ ! -f "${test_file}" ]; then
  echo "missing test file: ${test_file}" >&2
  exit 1
fi

required_tests=(
  "run_command_serves_http_route_in_oneshot_mode_when_clang_available"
  "c_bin_http_runtime_serves_health_route_in_oneshot_mode_when_clang_available"
  "c_bin_http_runtime_serves_users_post_with_json_response_when_clang_available"
  "c_bin_http_runtime_res_ok_honors_custom_status_when_clang_available"
  "c_bin_http_runtime_res_ok_meta_includes_meta_when_clang_available"
  "c_bin_http_runtime_req_json_rejects_invalid_body_when_clang_available"
  "c_bin_http_runtime_req_json_rejects_non_json_content_type_when_clang_available"
  "c_bin_http_runtime_req_json_rejects_oversized_body_when_clang_available"
  "c_bin_http_runtime_req_json_honors_env_body_limit_when_set"
  "c_bin_http_runtime_returns_405_on_method_mismatch_when_clang_available"
  "c_bin_http_runtime_handles_cors_preflight_when_enabled"
  "c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled"
  "c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled"
  "c_bin_http_runtime_applies_cors_origin_header_on_not_found_when_enabled"
  "c_bin_http_runtime_applies_cors_origin_header_on_405_when_enabled"
  "c_bin_http_runtime_applies_cors_origin_header_on_auth_reject_when_enabled"
  "c_bin_http_runtime_applies_cors_origin_header_on_csrf_reject_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_success_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_not_found_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_auth_reject_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_csrf_reject_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_405_when_enabled"
  "c_bin_http_runtime_applies_security_headers_on_cors_preflight_when_enabled"
  "c_bin_http_runtime_rejects_request_without_auth_header_when_enabled"
  "c_bin_http_runtime_allows_request_with_auth_header_when_enabled"
  "c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled"
  "c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled"
)

missing=0
for test_name in "${required_tests[@]}"; do
  if ! rg -Fq "fn ${test_name}(" "${test_file}"; then
    echo "missing required runtime e2e test: ${test_name}" >&2
    missing=1
  fi
done

if [ "${missing}" -ne 0 ]; then
  exit 1
fi

echo "m16 runtime http coverage contract passed"
