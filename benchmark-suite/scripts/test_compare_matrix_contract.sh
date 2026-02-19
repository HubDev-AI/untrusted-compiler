#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$root_dir/testdata/sample-sec4-report.json" "$tmp/sec4-report.json"
cp "$root_dir/testdata/sample-go-report.json" "$tmp/go-report.json"
cp "$root_dir/testdata/sample-node-report.json" "$tmp/node-report.json"

out="$tmp/compare-matrix.json"
"$root_dir/compare_matrix.sh" "$tmp" "$out" "sec4,go,node" >/dev/null

if ! jq -e '
  def row_ok:
    (.impl | type == "string")
    and (.endpoint | type == "string")
    and (.targetRps | type == "number")
    and (.requestsPerSec | type == "number")
    and (.p99 | type == "string")
    and (.loadGenerator | type == "string")
    and (.constantRate | type == "boolean")
    and has("rssKb")
    and ((.rssKb == null) or (.rssKb | type == "number"));
  def row_eq(a; b):
    a.impl == b.impl
    and a.endpoint == b.endpoint
    and a.targetRps == b.targetRps
    and a.requestsPerSec == b.requestsPerSec
    and a.p99 == b.p99
    and a.loadGenerator == b.loadGenerator
    and a.constantRate == b.constantRate
    and a.rssKb == b.rssKb;

  .version == "0.1"
  and (.endpoints | type == "array" and length > 0)
  and all(.endpoints[];
    . as $entry
    | ($entry.endpoint | type == "string")
    and ($entry.compared | type == "array" and length > 0)
    and (all($entry.compared[]; row_ok and (.endpoint == $entry.endpoint)))
    and ($entry.leader | row_ok)
    and ($entry.leader.endpoint == $entry.endpoint)
    and (any($entry.compared[]; row_eq(.; $entry.leader)))
  )
' "$out" >/dev/null; then
  echo "compare-matrix output failed contract checks" >&2
  exit 1
fi

echo "compare_matrix contract test passed"
