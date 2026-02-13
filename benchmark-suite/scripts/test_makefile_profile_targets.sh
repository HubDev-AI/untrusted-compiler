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

echo "makefile profile target test passed"
