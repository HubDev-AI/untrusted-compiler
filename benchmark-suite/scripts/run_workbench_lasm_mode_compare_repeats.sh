#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [options] [-- <args for run_workbench_lasm_mode_compare.sh>]

Runs the LASM mode compare sequentially multiple times and emits one aggregate
JSON artifact with median per-mode metrics.

Options:
  --repeats <n>           Number of sequential repeats (default: 3)
  --out <path>            Aggregate output JSON
  --keep-repeat-files     Keep per-repeat JSON artifacts
  --dry-run               Print planned commands only
  -h, --help              Show this help

All remaining arguments are passed through to:
  benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh
USAGE
}

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"
base_script="${suite_dir}/scripts/run_workbench_lasm_mode_compare.sh"
repeats=3
out_path="${suite_dir}/results/summaries/workbench-lasm-mode-compare-repeats.json"
keep_repeat_files="false"
dry_run="false"
pass_args=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repeats)
      repeats="${2:-}"
      shift 2
      ;;
    --repeats=*)
      repeats="${1#--repeats=}"
      shift
      ;;
    --out)
      out_path="${2:-}"
      shift 2
      ;;
    --out=*)
      out_path="${1#--out=}"
      shift
      ;;
    --keep-repeat-files)
      keep_repeat_files="true"
      shift
      ;;
    --dry-run)
      dry_run="true"
      shift
      ;;
    --)
      shift
      while [ "$#" -gt 0 ]; do
        pass_args+=("$1")
        shift
      done
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      pass_args+=("$1")
      shift
      ;;
  esac
done

if ! [[ "$repeats" =~ ^[0-9]+$ ]] || [ "$repeats" -lt 1 ]; then
  echo "--repeats must be an integer >= 1" >&2
  exit 2
fi

mkdir -p "$(dirname "$out_path")"
repeat_dir="$(mktemp -d "${suite_dir}/results/tmp-mode-compare-repeats.XXXXXX")"
cleanup() {
  if [ "$keep_repeat_files" != "true" ] && [ -d "$repeat_dir" ]; then
    rm -rf "$repeat_dir"
  fi
}
trap cleanup EXIT

repeat_outputs=()
repeat_index=1
while [ "$repeat_index" -le "$repeats" ]; do
  repeat_json="${repeat_dir}/repeat-${repeat_index}.json"
  repeat_outputs+=("$repeat_json")

  repeat_single_runs="${repeat_dir}/repeat-${repeat_index}-single-runs.json"
  repeat_fixed_runs="${repeat_dir}/repeat-${repeat_index}-fixed-runs.json"
  repeat_proxy_runs="${repeat_dir}/repeat-${repeat_index}-proxy-runs.json"
  repeat_single_report="${repeat_dir}/repeat-${repeat_index}-single-report.json"
  repeat_fixed_report="${repeat_dir}/repeat-${repeat_index}-fixed-report.json"
  repeat_proxy_report="${repeat_dir}/repeat-${repeat_index}-proxy-report.json"

  cmd=(
    "$base_script"
    "${pass_args[@]}"
    --single-runs-out "$repeat_single_runs"
    --fixed-runs-out "$repeat_fixed_runs"
    --proxy-runs-out "$repeat_proxy_runs"
    --single-report-out "$repeat_single_report"
    --fixed-report-out "$repeat_fixed_report"
    --proxy-report-out "$repeat_proxy_report"
    --out "$repeat_json"
  )

  if [ "$dry_run" = "true" ]; then
    printf 'repeat %s:' "$repeat_index"
    printf ' %q' "${cmd[@]}"
    printf '\n'
    repeat_index=$((repeat_index + 1))
    continue
  fi

  "${cmd[@]}"
  repeat_index=$((repeat_index + 1))
done

if [ "$dry_run" = "true" ]; then
  echo "aggregate artifact: ${out_path}"
  exit 0
fi

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -s \
  --arg generatedAt "$generated_at" \
  --argjson repeats "$repeats" \
  --arg repeatDir "$repeat_dir" '
  def median($values):
    ($values | map(select(. != null)) | sort) as $sorted
    | if ($sorted | length) == 0 then null
      elif ($sorted | length) % 2 == 1 then $sorted[($sorted | length / 2 | floor)]
      else (($sorted[($sorted | length / 2 | floor) - 1] + $sorted[($sorted | length / 2 | floor)]) / 2)
      end;

  def mode_stats($runs; $mode):
    {
      mode: $mode,
      repeatCount: ($runs | length),
      medianRequestsPerSec: median([$runs[] | .[$mode].observed.requestsPerSec]),
      minRequestsPerSec: ([$runs[] | .[$mode].observed.requestsPerSec] | min),
      maxRequestsPerSec: ([$runs[] | .[$mode].observed.requestsPerSec] | max),
      medianP99Ms: median([$runs[] | .[$mode].observed.p99Ms]),
      medianPeakRssKb: median([$runs[] | .[$mode].observed.peakRssKb]),
      lastObserved: ($runs[-1][$mode].observed)
    };

  . as $runs
  | [
      mode_stats($runs; "single"),
      mode_stats($runs; "fixed"),
      mode_stats($runs; "proxy")
    ] as $modeStats
  | (
      $modeStats
      | sort_by([-(.medianRequestsPerSec // 0), (.medianP99Ms // 1e18), (.medianPeakRssKb // 1e18), .mode])
      | .[0]
    ) as $recommended
  | {
      version: "0.1",
      generatedAt: $generatedAt,
      repeatCount: $repeats,
      repeatDir: $repeatDir,
      config: ($runs[-1].config // null),
      repeats: [
        range(0; $runs | length) as $index
        | {
            repeat: ($index + 1),
            comparison: ($runs[$index].comparison // null),
            single: ($runs[$index].single.observed // null),
            fixed: ($runs[$index].fixed.observed // null),
            proxy: ($runs[$index].proxy.observed // null)
          }
      ],
      modes: {
        single: ($modeStats[] | select(.mode == "single")),
        fixed: ($modeStats[] | select(.mode == "fixed")),
        proxy: ($modeStats[] | select(.mode == "proxy"))
      },
      recommendation: {
        mode: $recommended.mode,
        reason: ("medianRequestsPerSec=" + (($recommended.medianRequestsPerSec // 0) | tostring))
      }
    }
  ' "${repeat_outputs[@]}" >"$out_path"

echo "wrote ${out_path}"
