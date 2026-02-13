#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/testdata/sample-node-report.json" "$tmp/node-report.json"

out="$tmp/compare-ping.json"
"$root_dir/compare_reports.sh" "$tmp" ping "$out" >/dev/null

if ! jq -e '
  .version == "0.1"
  and (.endpoint == "ping")
  and (.compared | type == "array" and length > 0)
  and (.leader | type == "object")
  and (all(.compared[];
    (.impl | type == "string")
    and (.targetRps | type == "number")
    and (.requestsPerSec | type == "number")
    and (.p99 | type == "string")
    and (.loadGenerator | type == "string")
    and (.constantRate | type == "boolean")
  ))
  and (
    (.leader.impl | type == "string")
    and (.leader.targetRps | type == "number")
    and (.leader.requestsPerSec | type == "number")
    and (.leader.p99 | type == "string")
    and (.leader.loadGenerator | type == "string")
    and (.leader.constantRate | type == "boolean")
  )
' "$out" >/dev/null; then
  echo "compare-reports output failed contract checks" >&2
  exit 1
fi

echo "compare_reports contract test passed"
