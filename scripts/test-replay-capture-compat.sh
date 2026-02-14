#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check_script="${root_dir}/scripts/check-replay-capture-compat.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

capture_path="${tmp}/capture.json"

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_A",
  "compilerHash": "cpl_A",
  "runtimeHash": "rt_A",
  "request": {
    "method": "GET",
    "scheme": "https",
    "host": "example.com",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
      "sha256": "body_sha256",
      "truncated": false
    }
  },
  "env": {"timezone": "UTC", "locale": "en-US"},
  "determinism": {
    "seed": 1,
    "time": {"mode": "frozen", "nowMs": 1760000000000},
    "uuid": {"mode": "seeded"},
    "budget": {
      "maxBodyBytes": 1,
      "maxJsonBytes": 1,
      "maxJsonDepth": 1,
      "deadlineMs": 1
    }
  },
  "redaction": {"headers": [], "jsonPaths": []}
}
JSON

"${check_script}" --capture "${capture_path}" --policy-hash pol_A --compiler-hash cpl_A --runtime-hash rt_A >/dev/null

if "${check_script}" --capture "${capture_path}" --policy-hash pol_B --compiler-hash cpl_A --runtime-hash rt_A >/dev/null 2>&1; then
  echo "expected policy mismatch failure without allow flag" >&2
  exit 1
fi

"${check_script}" --capture "${capture_path}" --policy-hash pol_B --compiler-hash cpl_A --runtime-hash rt_A --allow-policy-mismatch >/dev/null

if "${check_script}" --capture "${capture_path}" --policy-hash pol_A --compiler-hash cpl_B --runtime-hash rt_A --allow-policy-mismatch >/dev/null 2>&1; then
  echo "expected compiler hash mismatch failure" >&2
  exit 1
fi

if "${check_script}" --capture "${capture_path}" --policy-hash pol_A --compiler-hash cpl_A --runtime-hash rt_B --allow-policy-mismatch >/dev/null 2>&1; then
  echo "expected runtime hash mismatch failure" >&2
  exit 1
fi

echo "replay capture compatibility test passed"
