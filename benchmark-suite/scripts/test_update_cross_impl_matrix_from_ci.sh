#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

matrix_ok="${root_dir}/testdata/sample-cross-impl-compare-matrix.json"
matrix_bad="${root_dir}/testdata/sample-compare-matrix.json"
target="${tmp}/compare-matrix.json"

dry_out="$("$root_dir/update_cross_impl_matrix_from_ci.sh" --dry-run --repo HubDev-AI/untrusted-compiler --out-dir "$tmp")"
if ! grep -q 'gh run list --repo HubDev-AI/untrusted-compiler --workflow benchmark-cross-impl-evidence.yml' <<<"$dry_out"; then
  echo "dry-run missing gh run list command" >&2
  exit 1
fi
if ! grep -q 'validate matrix includes impls sec4,go,node,rust' <<<"$dry_out"; then
  echo "dry-run missing validation step" >&2
  exit 1
fi
if ! grep -q 'check-benchmark-evidence-quality.sh --matrix <resolved-matrix-path> --fail-on-warning' <<<"$dry_out"; then
  echo "dry-run missing strict quality validation step" >&2
  exit 1
fi

dry_allow_out="$("$root_dir/update_cross_impl_matrix_from_ci.sh" --dry-run --matrix "$matrix_ok" --quality-allow-warning --target "$target")"
if grep -q 'check-benchmark-evidence-quality.sh --matrix .* --fail-on-warning' <<<"$dry_allow_out"; then
  echo "dry-run should omit strict quality flag when quality-allow-warning is set" >&2
  exit 1
fi

"$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$matrix_ok" --target "$target" >/dev/null
if [ ! -f "$target" ]; then
  echo "target matrix not written" >&2
  exit 1
fi
if ! jq -e '.endpoints[0].compared[] | select(.impl == "node")' "$target" >/dev/null; then
  echo "expected node impl in imported matrix" >&2
  exit 1
fi

if "$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$matrix_bad" --target "$target" >/dev/null 2>&1; then
  echo "expected failure for matrix missing required impls" >&2
  exit 1
fi

matrix_partial="$tmp/compare-matrix-partial-endpoint.json"
cat > "$matrix_partial" <<'EOF'
{
  "version": "0.1",
  "endpoints": [
    {
      "endpoint": "ping",
      "compared": [
        {"impl":"sec4","endpoint":"ping","targetRps":10000,"requestsPerSec":9800,"p99":"4.20ms","loadGenerator":"wrk2","constantRate":true},
        {"impl":"go","endpoint":"ping","targetRps":10000,"requestsPerSec":9700,"p99":"4.80ms","loadGenerator":"wrk2","constantRate":true},
        {"impl":"node","endpoint":"ping","targetRps":10000,"requestsPerSec":9000,"p99":"7.50ms","loadGenerator":"wrk2","constantRate":true},
        {"impl":"rust","endpoint":"ping","targetRps":10000,"requestsPerSec":9750,"p99":"4.60ms","loadGenerator":"wrk2","constantRate":true}
      ],
      "leader": {"impl":"sec4","endpoint":"ping","targetRps":10000,"requestsPerSec":9800,"p99":"4.20ms","loadGenerator":"wrk2","constantRate":true}
    },
    {
      "endpoint": "decode",
      "compared": [
        {"impl":"sec4","endpoint":"decode","targetRps":2000,"requestsPerSec":1800,"p99":"14.20ms","loadGenerator":"wrk2","constantRate":true}
      ],
      "leader": {"impl":"sec4","endpoint":"decode","targetRps":2000,"requestsPerSec":1800,"p99":"14.20ms","loadGenerator":"wrk2","constantRate":true}
    }
  ]
}
EOF

if "$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$matrix_partial" --target "$target" >/dev/null 2>&1; then
  echo "expected failure when an endpoint is missing required impl coverage" >&2
  exit 1
fi

warn_matrix="$tmp/warn-matrix.json"
jq '
  .endpoints[0].compared[0].constantRate = false
  | .endpoints[0].compared[0].loadGenerator = "wrk"
  | .endpoints[0].leader.constantRate = false
  | .endpoints[0].leader.loadGenerator = "wrk"
' "$matrix_ok" > "$warn_matrix"

if "$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$warn_matrix" --target "$target" >/dev/null 2>&1; then
  echo "expected strict quality mode to fail for non-constant-rate cross-impl matrix" >&2
  exit 1
fi

if ! "$root_dir/update_cross_impl_matrix_from_ci.sh" --matrix "$warn_matrix" --target "$target" --quality-allow-warning >/dev/null 2>&1; then
  echo "expected quality-allow-warning mode to permit non-constant-rate cross-impl matrix" >&2
  exit 1
fi

echo "update_cross_impl_matrix_from_ci test passed"
