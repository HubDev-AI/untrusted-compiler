#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="${root_dir}/scripts/test-m16-runtime-http-coverage.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

fixture="${tmp}/json_output_fixture.rs"

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

: > "${fixture}"
for test_name in "${required_tests[@]}"; do
  printf 'fn %s() {}\n' "${test_name}" >> "${fixture}"
done

"${checker}" --test-file "${fixture}" >/dev/null

missing_target="c_bin_http_runtime_req_json_rejects_invalid_body_when_clang_available"
missing_fixture="${tmp}/json_output_missing.rs"
: > "${missing_fixture}"
for test_name in "${required_tests[@]}"; do
  if [ "${test_name}" = "${missing_target}" ]; then
    continue
  fi
  printf 'fn %s() {}\n' "${test_name}" >> "${missing_fixture}"
done

if "${checker}" --test-file "${missing_fixture}" >"${tmp}/missing.log" 2>&1; then
  echo "expected runtime http coverage checker to fail when a required test is missing" >&2
  exit 1
fi

if ! rg -Fq "missing required runtime e2e test: ${missing_target}" "${tmp}/missing.log"; then
  echo "expected missing-test error output for ${missing_target}" >&2
  exit 1
fi

echo "m16 runtime http coverage guard test passed"
