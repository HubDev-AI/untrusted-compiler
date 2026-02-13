#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
  echo "usage: $0 <impl> <results_dir> <out_report.json> [sec_audit_json]" >&2
  exit 2
fi

impl="$1"
results_dir="$2"
out="$3"
sec_audit_path="${4:-}"

summaries_dir="${results_dir}/summaries"
env_path="${results_dir}/env.json"

mapfile -t summary_files < <(find "$summaries_dir" -maxdepth 1 -type f -name "${impl}-*.json" | sort)
if [ "${#summary_files[@]}" -eq 0 ]; then
  echo "no summary files found for impl=${impl} under ${summaries_dir}" >&2
  exit 2
fi

summaries_json="$(jq -s '.' "${summary_files[@]}")"

if [ -f "$env_path" ]; then
  env_json="$(cat "$env_path")"
else
  env_json='null'
fi

if [ -n "$sec_audit_path" ]; then
  if [ ! -f "$sec_audit_path" ]; then
    echo "sec audit file not found: ${sec_audit_path}" >&2
    exit 2
  fi
  sec_audit_json="$(cat "$sec_audit_path")"
else
  sec_audit_json='null'
fi

mkdir -p "$(dirname "$out")"

jq -n \
  --arg impl "$impl" \
  --argjson env "$env_json" \
  --argjson summaries "$summaries_json" \
  --argjson secAudit "$sec_audit_json" \
  '{
    version: "0.1",
    impl: $impl,
    env: $env,
    summaries: $summaries,
    secAudit: $secAudit
  }' > "$out"

echo "wrote $out"
