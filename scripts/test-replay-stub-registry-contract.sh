#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/check-replay-stub-registry-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

stubs_path="${tmp}/stubs.json"

cat > "${stubs_path}" <<'JSON'
{
  "version": "0.1",
  "stubs": {
    "net": [
      {
        "request": {
          "method": "GET",
          "url": "https://example.com/users/1",
          "bodySha256": "abc123"
        },
        "response": {
          "status": 200,
          "bodyBase64": "eyJvayI6dHJ1ZX0=",
          "truncated": false
        }
      }
    ],
    "db": [],
    "fs": []
  },
  "redaction": {
    "headers": ["authorization", "cookie"],
    "jsonPaths": ["$.password"]
  }
}
JSON

"${contract_script}" --stubs "${stubs_path}" >/dev/null

cat > "${stubs_path}" <<'JSON'
{
  "version": "0.1",
  "stubs": {},
  "redaction": {
    "headers": ["authorization", "cookie"],
    "jsonPaths": ["$.password"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when stubs.net is missing" >&2
  exit 1
fi

cat > "${stubs_path}" <<'JSON'
{
  "version": "0.1",
  "stubs": {
    "net": [
      {
        "request": {
          "method": "GET",
          "url": "https://example.com/users/1"
        },
        "response": {
          "status": 200,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie"],
    "jsonPaths": ["$.password"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when response body payload is missing" >&2
  exit 1
fi

cat > "${stubs_path}" <<'JSON'
{
  "version": "0.1",
  "stubs": {
    "net": [
      {
        "request": {
          "method": "GET",
          "url": "https://example.com/users/1",
          "bodySha256": "abc123"
        },
        "response": {
          "status": 200,
          "bodyBase64": "eyJvayI6dHJ1ZX0=",
          "truncated": false
        }
      },
      {
        "request": {
          "method": "GET",
          "url": "https://example.com/users/1",
          "bodySha256": "abc123"
        },
        "response": {
          "status": 200,
          "bodySha256": "def456",
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie"],
    "jsonPaths": ["$.password"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure on duplicate request signatures" >&2
  exit 1
fi

echo "replay stub registry contract test passed"
