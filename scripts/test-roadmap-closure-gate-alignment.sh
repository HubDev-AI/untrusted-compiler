#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
roadmap_path="${root_dir}/docs/05-sec4-master-roadmap.md"
closure_script_path="${root_dir}/scripts/check-milestone-closure.sh"

if [[ ! -f "${roadmap_path}" ]]; then
  echo "missing roadmap file: ${roadmap_path}" >&2
  exit 1
fi
if [[ ! -f "${closure_script_path}" ]]; then
  echo "missing closure script: ${closure_script_path}" >&2
  exit 1
fi

mapfile -t script_gates < <(
  rg -o '^emit_check "[^"]+"' "${closure_script_path}" \
    | sed -E 's/^emit_check "([^"]+)"$/\1/' \
    | sort -u
)

if [[ "${#script_gates[@]}" -eq 0 ]]; then
  echo "no emit_check gates found in ${closure_script_path}" >&2
  exit 1
fi

closure_block="$(awk '
  /^Current strict closure result:$/ { in_block=1; next }
  /^Strict closure interpretation:$/ { in_block=0 }
  in_block { print }
' "${roadmap_path}")"

mapfile -t roadmap_gates < <(
  printf '%s\n' "${closure_block}" \
    | rg -o '`M[0-9]+-[A-Z0-9]+`' \
    | tr -d '`' \
    | sort -u
)

if [[ "${#roadmap_gates[@]}" -eq 0 ]]; then
  echo "no closure gates found in roadmap closure table" >&2
  exit 1
fi

for gate in "${script_gates[@]}"; do
  if ! printf '%s\n' "${roadmap_gates[@]}" | grep -qx "${gate}"; then
    echo "missing gate '${gate}' in roadmap closure table" >&2
    exit 1
  fi
done

for gate in "${roadmap_gates[@]}"; do
  if ! printf '%s\n' "${script_gates[@]}" | grep -qx "${gate}"; then
    echo "stale roadmap gate '${gate}' not present in closure script" >&2
    exit 1
  fi
done

echo "roadmap closure gate alignment test passed"
