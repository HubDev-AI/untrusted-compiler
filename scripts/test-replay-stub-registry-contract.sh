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
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

"${contract_script}" --stubs "${stubs_path}" >/dev/null

cat > "${stubs_path}" <<'JSON'
{
  "version": "0.1",
  "stubs": {},
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
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
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
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
          "url": "https://example.com/users/1"
        },
        "response": {
          "status": 200.5,
          "bodyBase64": "eyJvayI6dHJ1ZX0=",
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when net response.status is not an integer" >&2
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
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when redaction headers are incomplete" >&2
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
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when redaction jsonPaths are incomplete" >&2
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
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure on duplicate request signatures" >&2
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
      }
    ],
    "db": [
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "f00d"
        },
        "response": {
          "rowCount": 1,
          "truncated": false
        }
      }
    ],
    "fs": [
      {
        "request": {
          "op": "read",
          "pathSha256": "beef"
        },
        "response": {
          "ok": true,
          "bytes": 128,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

"${contract_script}" --stubs "${stubs_path}" >/dev/null

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
    "db": [
      {
        "request": {
          "paramsSha256": "f00d"
        },
        "response": {
          "rowCount": 1,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when db.queryTemplateId is missing" >&2
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
      }
    ],
    "fs": [
      {
        "request": {
          "op": "read"
        },
        "response": {
          "ok": true,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure when fs.pathSha256 is missing" >&2
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
      }
    ],
    "db": [
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "x"
        },
        "response": {
          "rowCount": 1,
          "truncated": false
        }
      },
      {
        "request": {
          "queryTemplateId": "users.by_id",
          "paramsSha256": "x"
        },
        "response": {
          "rowCount": 2,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure on duplicate db request signatures" >&2
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
      }
    ],
    "fs": [
      {
        "request": {
          "op": "read",
          "pathSha256": "p"
        },
        "response": {
          "ok": true,
          "truncated": false
        }
      },
      {
        "request": {
          "op": "READ",
          "pathSha256": "p"
        },
        "response": {
          "ok": true,
          "truncated": false
        }
      }
    ]
  },
  "redaction": {
    "headers": ["authorization", "cookie", "set-cookie"],
    "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
  }
}
JSON

if "${contract_script}" --stubs "${stubs_path}" >/dev/null 2>&1; then
  echo "expected replay stub contract failure on duplicate fs request signatures" >&2
  exit 1
fi

echo "replay stub registry contract test passed"
