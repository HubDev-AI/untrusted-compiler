#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--output-json <path>] [--output-markdown <path>] [--format <text|json>]

Builds deterministic M28 priority matrix from the M28 kickoff brief.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m28-kickoff-brief.json"
output_json_path="${root_dir}/build/m28-priority-matrix.json"
output_markdown_path="${root_dir}/build/m28-priority-matrix.md"
output_format="text"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --kickoff-json)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      kickoff_json_path="$2"
      shift 2
      ;;
    --kickoff-json=*)
      kickoff_json_path="${1#--kickoff-json=}"
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

if [ ! -f "${kickoff_json_path}" ]; then
  echo "missing M28 kickoff brief json: ${kickoff_json_path}" >&2
  exit 1
fi

if ! jq -e '.m27Overall and .primaryFocus and (.pendingGates | type == "array")' "${kickoff_json_path}" >/dev/null; then
  echo "invalid M28 kickoff brief contract: ${kickoff_json_path}" >&2
  exit 1
fi

primary_focus="$(jq -r '.primaryFocus' "${kickoff_json_path}")"
m27_overall="$(jq -r '.m27Overall' "${kickoff_json_path}")"
pending_gate_count="$(jq -r '.pendingGates | length' "${kickoff_json_path}")"

runtime_score=60
release_score=60
editor_score=60

case "${primary_focus}" in
  runtime)
    runtime_score=75
    release_score=66
    editor_score=64
    ;;
  release)
    release_score=75
    runtime_score=66
    editor_score=64
    ;;
  editor)
    editor_score=75
    runtime_score=66
    release_score=64
    ;;
  stabilization)
    runtime_score=80
    release_score=65
    editor_score=58
    ;;
  *)
    echo "unsupported M28 primary focus: ${primary_focus}" >&2
    exit 1
    ;;
esac

if [ "${m27_overall}" != "PASS" ]; then
  runtime_score=$((runtime_score + 5))
fi
if [ "${pending_gate_count}" -gt 0 ]; then
  runtime_score=$((runtime_score + 3))
fi

tracks_json="$(
  jq -n \
    --argjson runtimeScore "${runtime_score}" \
    --argjson releaseScore "${release_score}" \
    --argjson editorScore "${editor_score}" \
    --arg primaryFocus "${primary_focus}" \
    '[
      {
        track: "runtime",
        score: $runtimeScore,
        focus: "runtime hardening and smoke reliability",
        rationale: (if $primaryFocus == "runtime" or $primaryFocus == "stabilization"
          then "Runtime remains the top safety axis for M28 kickoff."
          else "Runtime stays high priority for stability guarantees."
        end)
      },
      {
        track: "release",
        score: $releaseScore,
        focus: "release/publish integrity and promotion readiness",
        rationale: (if $primaryFocus == "release"
          then "Release confidence is the declared primary focus."
          else "Release integrity follows once runtime stability remains controlled."
        end)
      },
      {
        track: "editor",
        score: $editorScore,
        focus: "editor/LSP productivity expansion",
        rationale: (if $primaryFocus == "editor"
          then "Editor UX is the declared M28 acceleration focus."
          else "Editor expansion continues after critical runtime/release checks."
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
    --arg kickoffJson "${kickoff_json_path}" \
    --arg primaryFocus "${primary_focus}" \
    --arg m27Overall "${m27_overall}" \
    --argjson pendingGateCount "${pending_gate_count}" \
    --argjson tracks "${tracks_json}" \
    '{
      version: $version,
      kickoffJson: $kickoffJson,
      primaryFocus: $primaryFocus,
      m27Overall: $m27Overall,
      pendingGateCount: $pendingGateCount,
      tracks: $tracks
    }'
)"

mkdir -p "$(dirname "${output_json_path}")"
printf '%s\n' "${matrix_json}" > "${output_json_path}"

mkdir -p "$(dirname "${output_markdown_path}")"
{
  echo "# M28 Priority Matrix"
  echo
  printf -- "- source kickoff: \`%s\`\n" "${kickoff_json_path}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- m27Overall: \`%s\`\n" "${m27_overall}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
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

echo "m28 priority matrix built: ${output_json_path}"
