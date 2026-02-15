#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--kickoff-json <path>] [--output-json <path>] [--output-markdown <path>] [--format <text|json>]

Builds deterministic M36 priority matrix from the M36 kickoff brief.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
kickoff_json_path="${root_dir}/build/m36-kickoff-brief.json"
output_json_path="${root_dir}/build/m36-priority-matrix.json"
output_markdown_path="${root_dir}/build/m36-priority-matrix.md"
output_format="text"
closure_marker="M36-B"

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
  echo "missing M36 kickoff brief json: ${kickoff_json_path}" >&2
  exit 1
fi

if ! jq -e '
  .version
  and .kickoffGate
  and .m35Overall
  and .convergenceOverall
  and .primaryFocus
  and .selectedTrack
  and .runtimeStatus
  and (.pendingGates | type == "array")
  and (.pendingGateCount | type == "number")
  and (.pendingGateCount == (.pendingGates | length))
' "${kickoff_json_path}" >/dev/null; then
  echo "invalid M36 kickoff brief contract: ${kickoff_json_path}" >&2
  exit 1
fi

kickoff_gate="$(jq -r '.kickoffGate' "${kickoff_json_path}")"
primary_focus="$(jq -r '.primaryFocus' "${kickoff_json_path}")"
selected_track="$(jq -r '.selectedTrack' "${kickoff_json_path}")"
m35_overall="$(jq -r '.m35Overall' "${kickoff_json_path}")"
convergence_overall="$(jq -r '.convergenceOverall' "${kickoff_json_path}")"
runtime_status="$(jq -r '.runtimeStatus' "${kickoff_json_path}")"
pending_gate_count="$(jq -r '.pendingGateCount' "${kickoff_json_path}")"

if [ "${kickoff_gate}" != "M36-A" ]; then
  echo "unexpected M36 kickoff gate marker: ${kickoff_gate}" >&2
  exit 1
fi

case "${primary_focus}" in
  runtime|release|editor|stabilization)
    ;;
  *)
    echo "unsupported M36 primary focus: ${primary_focus}" >&2
    exit 1
    ;;
esac

case "${selected_track}" in
  runtime|release|editor)
    ;;
  *)
    echo "unsupported M36 selected track: ${selected_track}" >&2
    exit 1
    ;;
esac

case "${m35_overall}" in
  PASS|PENDING|FAIL)
    ;;
  *)
    echo "unsupported M36 m35Overall status: ${m35_overall}" >&2
    exit 1
    ;;
esac

case "${convergence_overall}" in
  PASS|PENDING|FAIL)
    ;;
  *)
    echo "unsupported M36 convergenceOverall status: ${convergence_overall}" >&2
    exit 1
    ;;
esac

case "${runtime_status}" in
  PASS|DRY_RUN|PENDING|FAIL)
    ;;
  *)
    echo "unsupported M36 runtimeStatus: ${runtime_status}" >&2
    exit 1
    ;;
esac

runtime_score=60
release_score=60
editor_score=60

case "${primary_focus}" in
  runtime)
    runtime_score=80
    release_score=68
    editor_score=66
    ;;
  release)
    release_score=80
    runtime_score=68
    editor_score=66
    ;;
  editor)
    editor_score=80
    runtime_score=68
    release_score=66
    ;;
  stabilization)
    runtime_score=84
    release_score=62
    editor_score=58
    ;;
esac

case "${selected_track}" in
  runtime)
    runtime_score=$((runtime_score + 2))
    ;;
  release)
    release_score=$((release_score + 2))
    ;;
  editor)
    editor_score=$((editor_score + 2))
    ;;
esac

if [ "${m35_overall}" != "PASS" ]; then
  runtime_score=$((runtime_score + 4))
fi
if [ "${convergence_overall}" != "PASS" ]; then
  runtime_score=$((runtime_score + 4))
fi
if [ "${pending_gate_count}" -gt 0 ]; then
  runtime_score=$((runtime_score + 3))
fi

case "${runtime_status}" in
  FAIL)
    runtime_score=$((runtime_score + 5))
    ;;
  PENDING)
    runtime_score=$((runtime_score + 3))
    ;;
  DRY_RUN)
    runtime_score=$((runtime_score + 2))
    ;;
esac

tracks_json="$(
  jq -n \
    --argjson runtimeScore "${runtime_score}" \
    --argjson releaseScore "${release_score}" \
    --argjson editorScore "${editor_score}" \
    --arg primaryFocus "${primary_focus}" \
    --arg selectedTrack "${selected_track}" \
    '[
      {
        track: "runtime",
        score: $runtimeScore,
        focus: "runtime de-stubbing completion and typed sink closure readiness",
        rationale: (
          if $primaryFocus == "runtime" or $primaryFocus == "stabilization"
          then "Runtime stays first to lock closure-critical implementation risk."
          elif $selectedTrack == "runtime"
          then "M35 handoff selected runtime, so runtime keeps elevated execution priority."
          else "Runtime remains high to preserve deterministic security guarantees."
          end
        )
      },
      {
        track: "release",
        score: $releaseScore,
        focus: "release artifact integrity and closure evidence packaging",
        rationale: (
          if $primaryFocus == "release"
          then "Release is the declared M36 focus and gets first delivery priority."
          elif $selectedTrack == "release"
          then "M35 handoff selected release, so release readiness remains elevated."
          else "Release hardening proceeds once runtime risk is sufficiently controlled."
          end
        )
      },
      {
        track: "editor",
        score: $editorScore,
        focus: "editor/LSP diagnostics throughput and UX polish",
        rationale: (
          if $primaryFocus == "editor"
          then "Editor is the declared M36 acceleration lane and gets first priority."
          elif $selectedTrack == "editor"
          then "M35 handoff selected editor, so diagnostics productivity remains elevated."
          else "Editor improvements continue after runtime/release stabilization pressure reduces."
          end
        )
      }
    ]
    | sort_by(-.score, .track)
    | to_entries
    | map(.value + {priority: (.key + 1)})'
)"

matrix_json="$(
  jq -n \
    --arg version "0.1" \
    --arg closureMarker "${closure_marker}" \
    --arg kickoffJson "${kickoff_json_path}" \
    --arg kickoffGate "${kickoff_gate}" \
    --arg primaryFocus "${primary_focus}" \
    --arg selectedTrack "${selected_track}" \
    --arg m35Overall "${m35_overall}" \
    --arg convergenceOverall "${convergence_overall}" \
    --arg runtimeStatus "${runtime_status}" \
    --argjson pendingGateCount "${pending_gate_count}" \
    --argjson tracks "${tracks_json}" \
    '{
      version: $version,
      closureMarker: $closureMarker,
      kickoffJson: $kickoffJson,
      kickoffGate: $kickoffGate,
      primaryFocus: $primaryFocus,
      selectedTrack: $selectedTrack,
      m35Overall: $m35Overall,
      convergenceOverall: $convergenceOverall,
      runtimeStatus: $runtimeStatus,
      pendingGateCount: $pendingGateCount,
      tracks: $tracks
    }'
)"

mkdir -p "$(dirname "${output_json_path}")"
printf '%s\n' "${matrix_json}" > "${output_json_path}"

mkdir -p "$(dirname "${output_markdown_path}")"
{
  echo "# M36 Priority Matrix"
  echo
  printf -- "- source kickoff: \`%s\`\n" "${kickoff_json_path}"
  printf -- "- kickoffGate: \`%s\`\n" "${kickoff_gate}"
  printf -- "- closureMarker: \`%s\`\n" "${closure_marker}"
  printf -- "- primaryFocus: \`%s\`\n" "${primary_focus}"
  printf -- "- selectedTrack: \`%s\`\n" "${selected_track}"
  printf -- "- m35Overall: \`%s\`\n" "${m35_overall}"
  printf -- "- convergenceOverall: \`%s\`\n" "${convergence_overall}"
  printf -- "- runtimeStatus: \`%s\`\n" "${runtime_status}"
  printf -- "- pendingGateCount: \`%s\`\n" "${pending_gate_count}"
  echo
  echo "| Priority | Track | Score | Focus |"
  echo "| --- | --- | --- | --- |"
  jq -r '.tracks[] | "| \(.priority) | \(.track) | \(.score) | \(.focus) |"' <<<"${matrix_json}"
  echo
  echo "## Rationales"
  jq -r '.tracks[] | "- [P" + (.priority | tostring) + "] " + .track + ": " + .rationale' <<<"${matrix_json}"
} > "${output_markdown_path}"

if [ "${output_format}" = "json" ]; then
  printf '%s\n' "${matrix_json}"
  exit 0
fi

echo "m36 priority matrix built: ${output_json_path}"
