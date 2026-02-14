#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check_script="${root_dir}/scripts/check-replay-capture-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

capture_path="${tmp}/capture.json"

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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

"${check_script}" --capture "${capture_path}" >/dev/null

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure when traceId is missing" >&2
  exit 1
fi

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "none",
      "truncated": true
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

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure when encoding=none lacks sha256" >&2
  exit 1
fi

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "raw",
      "bytes": "e30=",
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

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure for unsupported body encoding" >&2
  exit 1
fi

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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
  "redaction": {"headers": [], "jsonPaths": []},
  "dependencies": {
    "db": [
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "x"
        }
      }
    ],
    "fs": [
      {
        "request": {
          "op": "read",
          "pathSha256": "p"
        }
      }
    ]
  }
}
JSON

"${check_script}" --capture "${capture_path}" >/dev/null

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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
  "redaction": {"headers": [], "jsonPaths": []},
  "dependencies": {
    "db": [
      {
        "request": {
          "paramsSha256": "x"
        }
      }
    ]
  }
}
JSON

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure when capture dependencies.db request.queryTemplateId is missing" >&2
  exit 1
fi

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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
  "redaction": {"headers": [], "jsonPaths": []},
  "dependencies": {
    "db": [
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "x"
        }
      },
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "x"
        }
      }
    ]
  }
}
JSON

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure when capture dependencies.db request signatures are duplicated" >&2
  exit 1
fi

cat > "${capture_path}" <<'JSON'
{
  "version": "0.1",
  "captureId": "cap_01",
  "traceId": "tr_01",
  "timeMs": 1760000000000,
  "policyHash": "pol_abc",
  "compilerHash": "cpl_abc",
  "runtimeHash": "rt_abc",
  "request": {
    "method": "GET",
    "path": "/ping",
    "headers": {},
    "body": {
      "encoding": "base64",
      "bytes": "e30=",
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
  "redaction": {"headers": [], "jsonPaths": []},
  "dependencies": {
    "fs": [
      {
        "request": {
          "op": "read",
          "pathSha256": "p"
        }
      },
      {
        "request": {
          "op": "READ",
          "pathSha256": "p"
        }
      }
    ]
  }
}
JSON

if "${check_script}" --capture "${capture_path}" >/dev/null 2>&1; then
  echo "expected failure when capture dependencies.fs request signatures are duplicated" >&2
  exit 1
fi

echo "replay capture contract test passed"
