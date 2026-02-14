#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--report <path>] [--output <path>] [--format <markdown|json>]

Generates an M18 kickoff brief from the latest clean-clone rehearsal report.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_path="${root_dir}/build/operator-clean-clone-rehearsal-live/rehearsal-report.json"
output_path="${root_dir}/build/m18-kickoff-brief.md"
output_format="markdown"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --report)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      report_path="$2"
      shift 2
      ;;
    --report=*)
      report_path="${1#--report=}"
      shift
      ;;
    --output)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_path="$2"
      shift 2
      ;;
    --output=*)
      output_path="${1#--output=}"
      shift
      ;;
    --format)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_format="$2"
      shift 2
      ;;
    --format=*)
      output_format="${1#--format=}"
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

case "${output_format}" in
  markdown|json)
    ;;
  *)
    echo "unknown format: ${output_format}" >&2
    usage
    exit 2
    ;;
esac

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

if [ ! -f "${report_path}" ]; then
  echo "missing rehearsal report: ${report_path}" >&2
  exit 1
fi

if ! jq -e '.overall and .failedStep and (.steps | type == "array") and (.friction | type == "array")' "${report_path}" >/dev/null; then
  echo "invalid rehearsal report contract: ${report_path}" >&2
  exit 1
fi

overall="$(jq -r '.overall' "${report_path}")"
failed_step="$(jq -r '.failedStep' "${report_path}")"
step_count="$(jq -r '.steps | length' "${report_path}")"
friction_count="$(jq -r '.friction | length' "${report_path}")"

recommendations_json='[]'
if [ "${overall}" = "PASS" ] && [ "${friction_count}" = "0" ]; then
  recommendations_json="$(jq -n '[
    "Start M18 with runtime/editor/release priority matrix and closure-gated first slice.",
    "Keep clean-clone rehearsal in operator smoke cadence for regression detection."
  ]')"
else
  recommendations_json="$(jq -n --arg failedStep "${failed_step}" '[
    "Address failing rehearsal step before expanding M18 scope.",
    ("Use step log evidence to resolve failure: " + $failedStep)
  ]')"
fi

summary_json="$(
  jq -n \
    --arg version "0.1" \
    --arg reportPath "${report_path}" \
    --arg overall "${overall}" \
    --arg failedStep "${failed_step}" \
    --argjson stepCount "${step_count}" \
    --argjson frictionCount "${friction_count}" \
    --argjson recommendations "${recommendations_json}" \
    '{
      version: $version,
      reportPath: $reportPath,
      overall: $overall,
      failedStep: $failedStep,
      stepCount: $stepCount,
      frictionCount: $frictionCount,
      recommendations: $recommendations
    }'
)"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${summary_json}"
  exit 0
fi

mkdir -p "$(dirname "${output_path}")"

{
  echo "# M18 Kickoff Brief"
  echo
  printf -- "- source report: \`%s\`\\n" "${report_path}"
  printf -- "- overall: \`%s\`\\n" "${overall}"
  printf -- "- failedStep: \`%s\`\\n" "${failed_step}"
  printf -- "- stepCount: \`%s\`\\n" "${step_count}"
  printf -- "- frictionCount: \`%s\`\\n" "${friction_count}"
  echo
  echo "## Recommendations"
  jq -r '.recommendations[] | "- " + .' <<<"${summary_json}"
} > "${output_path}"

echo "m18 kickoff brief generated: ${output_path}"
