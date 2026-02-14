#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--report <path>] [--output-json <path>] [--output-markdown <path>] [--format <text|json>]

Builds a deterministic M18 priority matrix from clean-clone rehearsal evidence.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
report_path="${root_dir}/build/operator-clean-clone-rehearsal-live/rehearsal-report.json"
output_json_path="${root_dir}/build/m18-priority-matrix.json"
output_markdown_path="${root_dir}/build/m18-priority-matrix.md"
output_format="text"

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
    --output-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_json_path="$2"
      shift 2
      ;;
    --output-json=*)
      output_json_path="${1#--output-json=}"
      shift
      ;;
    --output-markdown)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      output_markdown_path="$2"
      shift 2
      ;;
    --output-markdown=*)
      output_markdown_path="${1#--output-markdown=}"
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
  text|json)
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

if ! jq -e '.overall and (.friction | type == "array")' "${report_path}" >/dev/null; then
  echo "invalid rehearsal report contract: ${report_path}" >&2
  exit 1
fi

overall="$(jq -r '.overall' "${report_path}")"
friction_count="$(jq -r '.friction | length' "${report_path}")"

runtime_score=0
editor_score=0
release_score=0

if [ "${overall}" = "PASS" ]; then
  runtime_score=$((60 + friction_count * 2))
  editor_score=$((65 + friction_count))
  release_score=$((62 + friction_count * 2))
else
  runtime_score=$((80 + friction_count * 2))
  editor_score=$((55 + friction_count))
  release_score=$((70 + friction_count * 2))
fi

tracks_json="$(
  jq -n \
    --arg overall "${overall}" \
    --argjson frictionCount "${friction_count}" \
    --argjson runtimeScore "${runtime_score}" \
    --argjson editorScore "${editor_score}" \
    --argjson releaseScore "${release_score}" \
    '[
      {
        track: "runtime",
        score: $runtimeScore,
        focus: "runtime hardening and smoke contract coverage",
        rationale: (if $overall == "PASS"
          then "Runtime remains stable; keep preventive hardening active."
          else "Runtime instability blocks expansion; prioritize remediation first."
        end)
      },
      {
        track: "editor",
        score: $editorScore,
        focus: "LSP/Zed tooling contract expansion",
        rationale: (if $overall == "PASS"
          then "Stable runtime allows increasing editor/productivity surface."
          else "Editor expansion remains secondary until runtime path recovers."
        end)
      },
      {
        track: "release",
        score: $releaseScore,
        focus: "release-gate and operator rollout integrity",
        rationale: (if $overall == "PASS"
          then "Strengthen publish/release confidence while handoff evidence is fresh."
          else "Release confidence depends on resolving failing rehearsal stages first."
        end)
      }
    ]
    | sort_by(-.score, .track)
    | to_entries
    | map(.value + {priority: (.key + 1)})'
)"

matrix_json="$(
  jq -n \
    --arg version "0.1" \
    --arg reportPath "${report_path}" \
    --arg overall "${overall}" \
    --argjson frictionCount "${friction_count}" \
    --argjson tracks "${tracks_json}" \
    '{
      version: $version,
      reportPath: $reportPath,
      overall: $overall,
      frictionCount: $frictionCount,
      tracks: $tracks
    }'
)"

mkdir -p "$(dirname "${output_json_path}")"
printf '%s\n' "${matrix_json}" > "${output_json_path}"

mkdir -p "$(dirname "${output_markdown_path}")"
{
  echo "# M18 Priority Matrix"
  echo
  printf -- "- source report: \`%s\`\n" "${report_path}"
  printf -- "- overall: \`%s\`\n" "${overall}"
  printf -- "- frictionCount: \`%s\`\n" "${friction_count}"
  echo
  echo "| Priority | Track | Score | Focus |"
  echo "| --- | --- | --- | --- |"
  jq -r '.tracks[] | "| \(.priority) | \(.track) | \(.score) | \(.focus) |"' <<<"${matrix_json}"
  echo
  echo "## Rationales"
  jq -r '.tracks[] | "- [P" + (.priority|tostring) + "] " + .track + ": " + .rationale' <<<"${matrix_json}"
} > "${output_markdown_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${matrix_json}"
  exit 0
fi

echo "m18 priority matrix built: ${output_json_path}"
