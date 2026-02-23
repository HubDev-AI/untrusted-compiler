#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

matrix="$tmp/matrix.json"
cat >"${matrix}" <<'JSON'
{
  "impl": "sec4-lasm-cluster",
  "run": {
    "projectPath": "examples/lasm-alpha-full",
    "requestPath": "/health",
    "duration": "40s",
    "threads": 8,
    "connections": 256,
    "targetRequests": 1000000,
    "clusterRelayPumpBatchMax": "256"
  },
  "boostSteps": [2, 4, 6],
  "runs": [
    {
      "saturationBoostStep": 2,
      "pass": true,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 12000
    },
    {
      "saturationBoostStep": 4,
      "pass": true,
      "requests": 1260000,
      "requestsPerSec": 63000,
      "peakRssKb": 12500,
      "p99": "4.20ms",
      "clusterRelayWorkersResolved": 2,
      "clusterAcceptWorkersResolved": 2,
      "clusterRelayAcceptBatchMaxResolved": 64,
      "clusterRelayPumpBatchMaxResolved": 256,
      "clusterRelayLiveSenderCountResolved": 2,
      "clusterRelayDispatchSaturationShortCircuitTotal": 18,
      "clusterRelayDispatchSaturationShortCircuitPerSec": 3.1,
      "clusterDbAdapterResolved": "sqlite",
      "clusterDbSqliteBusyTimeoutMsResolved": 2500,
      "clusterDbSqliteJournalModeResolved": "WAL",
      "clusterDbSqliteSynchronousResolved": "NORMAL",
      "clusterDbSqliteLockRetryMaxResolved": 7,
      "clusterDbSqliteLockRetryDelayMsResolved": 9
    },
    {
      "saturationBoostStep": 6,
      "pass": false,
      "requests": 1400000,
      "requestsPerSec": 70000,
      "peakRssKb": 11000
    }
  ]
}
JSON

analysis="$tmp/analysis.json"
cat >"${analysis}" <<'JSON'
{
  "summary": {
    "runCount": 3,
    "passCount": 2,
    "selectionMode": "pass-first",
    "recommendedBoostStep": 4
  },
  "rankedRuns": [
    {
      "saturationBoostStep": 4,
      "pass": true,
      "requests": 1260000,
      "requestsPerSec": 63000,
      "peakRssKb": 12500,
      "p99": "4.20ms",
      "clusterRelayWorkersResolved": 2,
      "clusterAcceptWorkersResolved": 2,
      "clusterRelayAcceptBatchMaxResolved": 64,
      "clusterRelayPumpBatchMaxResolved": 256,
      "clusterRelayLiveSenderCountResolved": 2,
      "clusterRelayDispatchSaturationShortCircuitTotal": 18,
      "clusterRelayDispatchSaturationShortCircuitPerSec": 3.1
    },
    {
      "saturationBoostStep": 2,
      "pass": true,
      "requests": 1200000,
      "requestsPerSec": 60000,
      "peakRssKb": 12000
    },
    {
      "saturationBoostStep": 6,
      "pass": false,
      "requests": 1400000,
      "requestsPerSec": 70000,
      "peakRssKb": 11000
    }
  ]
}
JSON

verify="$tmp/verify.json"
cat >"${verify}" <<'JSON'
{
  "pass": true,
  "requestsTargetMet": true,
  "run": {
    "autoscaleSaturationBoostStep": 4,
    "clusterRelayWorkersResolved": 2,
    "clusterAcceptWorkersResolved": 2,
    "clusterRelayAcceptBatchMaxResolved": 64,
    "clusterRelayPumpBatchMaxResolved": 256,
    "clusterRelayQueueCapacityResolved": 2048,
    "clusterRelayQueueShardCapacityResolved": 1024,
    "clusterRelayLiveSenderCountResolved": 2,
    "clusterRelayDispatchSaturationShortCircuitTotal": 22,
    "clusterRelayDispatchSaturationShortCircuitPerSec": 2.8,
    "clusterDbAdapterResolved": "sqlite",
    "clusterDbSqliteBusyTimeoutMsResolved": 2500,
    "clusterDbSqliteJournalModeResolved": "WAL",
    "clusterDbSqliteSynchronousResolved": "NORMAL",
    "clusterDbSqliteLockRetryMaxResolved": 7,
    "clusterDbSqliteLockRetryDelayMsResolved": 9
  },
  "observed": {
    "requests": 1280000,
    "requestsPerSec": 64000,
    "peakRssKb": 12400,
    "p99": "3.90ms"
  }
}
JSON

out="$tmp/summary.md"
"${root_dir}/scripts/render_lasm_cluster_saturation_boost_summary.sh" "${matrix}" "${analysis}" "${out}" "${verify}" >/dev/null

if ! grep -q '^# LASM Saturation Boost Summary (v0.1)$' "${out}"; then
  echo "summary missing title" >&2
  exit 1
fi
if ! grep -q '^- Recommended boost step: 4$' "${out}"; then
  echo "summary missing recommended boost step line" >&2
  exit 1
fi
if ! grep -q '^## Ranked Runs$' "${out}"; then
  echo "summary missing ranked runs section" >&2
  exit 1
fi
if ! grep -q '^- Relay pump batch max: 256$' "${out}"; then
  echo "summary missing probe profile relay pump batch line" >&2
  exit 1
fi
if ! grep -q '| 1 | 4 | true | 63000 | 4.20ms | 1260000 | 12500 | 2 | 2 | 64 | 256 | 2 | 18 | 3.1 |' "${out}"; then
  echo "summary missing ranked run row for recommended step" >&2
  exit 1
fi
if ! grep -q '^## Recommended Step Verification$' "${out}"; then
  echo "summary missing verification section" >&2
  exit 1
fi
if ! grep -q '^## Resolved DB Runtime (Recommended Step)$' "${out}"; then
  echo "summary missing resolved DB runtime section" >&2
  exit 1
fi
if ! grep -q '^- DB adapter (resolved): sqlite$' "${out}"; then
  echo "summary missing resolved DB adapter line for recommended row" >&2
  exit 1
fi
if ! grep -q '^- DB sqlite lock retry max (resolved): 7$' "${out}"; then
  echo "summary missing resolved DB sqlite lock retry max line for recommended row" >&2
  exit 1
fi
if ! grep -q '^- Requests/sec: 64000$' "${out}"; then
  echo "summary missing verification throughput line" >&2
  exit 1
fi
if ! grep -q '^- Relay workers (resolved): 2$' "${out}"; then
  echo "summary missing verification resolved relay worker line" >&2
  exit 1
fi
if ! grep -q '^- Relay pump batch max (resolved): 256$' "${out}"; then
  echo "summary missing verification relay pump batch line" >&2
  exit 1
fi
if ! grep -q '^- Relay live sender count (resolved): 2$' "${out}"; then
  echo "summary missing verification relay live sender count line" >&2
  exit 1
fi
if ! grep -q '^- Relay dispatch short-circuit total (resolved): 22$' "${out}"; then
  echo "summary missing verification short-circuit total line" >&2
  exit 1
fi
if ! grep -q '^- Relay dispatch short-circuit per sec (resolved): 2.8$' "${out}"; then
  echo "summary missing verification short-circuit per-sec line" >&2
  exit 1
fi
if ! grep -q '^- DB sqlite journal mode (resolved): WAL$' "${out}"; then
  echo "summary missing verification db sqlite journal mode line" >&2
  exit 1
fi
if ! grep -q '^- DB sqlite lock retry delay ms (resolved): 9$' "${out}"; then
  echo "summary missing verification db sqlite lock retry delay line" >&2
  exit 1
fi

verify_bad="$tmp/verify-bad.json"
cat >"${verify_bad}" <<'JSON'
{
  "run": {
    "autoscaleSaturationBoostStep": 2
  }
}
JSON
if "${root_dir}/scripts/render_lasm_cluster_saturation_boost_summary.sh" "${matrix}" "${analysis}" "${tmp}/bad.md" "${verify_bad}" >/tmp/lasm-sat-boost-summary-mismatch.log 2>&1; then
  echo "summary renderer accepted mismatched verification boost step" >&2
  exit 1
fi
if ! grep -q 'recommended verification boost step mismatch: expected 4, got 2' /tmp/lasm-sat-boost-summary-mismatch.log; then
  echo "summary renderer missing mismatch diagnostic" >&2
  exit 1
fi

echo "render_lasm_cluster_saturation_boost_summary test passed"
