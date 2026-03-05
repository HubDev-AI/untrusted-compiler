#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp_json="$(mktemp)"
tmp_md="$(mktemp)"
trap 'rm -f "$tmp_json" "$tmp_md"' EXIT

cat >"$tmp_json" <<'JSON'
{
  "mode": "workbench-full-benchmark-suite-repeats",
  "generatedAt": "2026-02-26T12:00:00Z",
  "dryRun": false,
  "runCount": 2,
  "runs": [
    {
      "run": "run-001",
      "summary": "/tmp/workbench-full-run-001.json",
      "compareMatrix": "/tmp/workbench-compare-matrix-run-001.json",
      "stepMatrix": "/tmp/workbench-step-matrix-run-001.json",
      "report": "/tmp/workbench-full-report-run-001.md"
    }
  ],
  "compareStats": [
    {
      "impl": "sec4",
      "endpoint": "wb-task-get",
      "samples": 2,
      "requestsPerSec": { "mean": 101.5, "min": 100, "max": 103 },
      "p99Ms": { "mean": 1.2, "min": 1.0, "max": 1.5 },
      "rssKb": { "mean": 2048, "min": 2000, "max": 2100 }
    }
  ],
  "stepStats": [
    {
      "impl": "sec4",
      "endpoint": "wb-task-get",
      "samples": 2,
      "kneeDetectedCount": 2,
      "kneeDetectedRatio": 1.0,
      "kneeAtTargetRps": { "mean": 120, "min": 100, "max": 140 },
      "achievedRatioMin": { "mean": 0.95, "min": 0.90, "max": 1.00 },
      "achievedRatioMax": { "mean": 1.05, "min": 1.00, "max": 1.10 },
      "p99MinMs": { "mean": 1.1, "min": 1.0, "max": 1.2 },
      "p99MaxMs": { "mean": 2.3, "min": 2.0, "max": 2.6 }
    }
  ]
}
JSON

"$root_dir/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh" "$tmp_json" "$tmp_md" >/tmp/render-workbench-repeats.out

if ! grep -q 'Workbench Full-Suite Repeated-Run Summary (v0.1)' "$tmp_md"; then
  echo "missing report title in rendered markdown" >&2
  exit 1
fi
if ! grep -q '| run-001 | `/tmp/workbench-full-run-001.json` | `/tmp/workbench-compare-matrix-run-001.json` | `/tmp/workbench-step-matrix-run-001.json` | `/tmp/workbench-full-report-run-001.md` |' "$tmp_md"; then
  echo "missing runs table row in rendered markdown" >&2
  exit 1
fi
if ! grep -q '| sec4 | wb-task-get | 2 | 101.5 | 100 | 103 | 1.2 | 1 | 1.5 | 2048 | 2000 | 2100 |' "$tmp_md"; then
  echo "missing compare stats row in rendered markdown" >&2
  exit 1
fi
if ! grep -q '| sec4 | wb-task-get | 2 | 2 | 100 | 120 | 100 | 140 | 0.95 | 1.05 | 1.1 | 2.3 |' "$tmp_md"; then
  echo "missing step stats row in rendered markdown" >&2
  exit 1
fi

if "$root_dir/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh" /tmp/does-not-exist.json "$tmp_md" >/dev/null 2>&1; then
  echo "expected missing input path to fail" >&2
  exit 1
fi

echo "render_workbench_full_benchmark_suite_repeats_summary test passed"
