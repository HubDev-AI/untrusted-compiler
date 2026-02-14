#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-replay-cli-json-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

cli_path="${tmp}/main.rs"

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "deny",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []},
                "mockDependencyStubSummaries": {"db": [], "fs": []},
                "mockDependencyTraces": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

"${contract_script}" --cli "${cli_path}" >/dev/null

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Json)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "deny",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []},
                "mockDependencyStubSummaries": {"db": [], "fs": []},
                "mockDependencyTraces": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when format default drifts" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "deny",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty"
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when policyHashMatched key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "effectsMode": "deny",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when allowPolicyMismatch key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "deny",
                "warnings": []
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when stubCounts key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []},
                "mockDependencyStubSummaries": {"db": [], "fs": []},
                "mockDependencyTraces": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when stubDetails key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []},
                "mockDependencyStubSummaries": {"db": [], "fs": []},
                "mockDependencyTraces": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when mockMatchedStub key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when mockDependencyMatches key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when mockDependencySignatures key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when mockDependencyStubSummaries key is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Replay {
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
    },
}

fn render(output_format: ReplayOutputFormat) {
    match output_format {
        ReplayOutputFormat::Text => {
            println!("ok");
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": "capture.json",
                "stubs": "stubs.json",
                "policyHashMatched": true,
                "compilerHashMatched": true,
                "runtimeHashMatched": true,
                "allowPolicyMismatch": false,
                "effectsMode": "mock",
                "warnings": [],
                "stubCounts": {"net": 1, "db": 0, "fs": 0},
                "stubDetails": {"db": {"entries": 0, "uniqueQueryTemplateIds": 0}, "fs": {"entries": 0, "readOps": 0, "writeOps": 0, "otherOps": 0}},
                "mockRequestSignature": "GET|https://example.com/ping|empty",
                "mockMatchedStub": {"status": 200, "truncated": false, "bodyKind": "base64"},
                "mockDependencyMatches": {"db": 0, "fs": 0},
                "mockDependencySignatures": {"db": [], "fs": []},
                "mockDependencyStubSummaries": {"db": [], "fs": []}
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("payload should serialize")
            );
        }
    }
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected replay json contract failure when mockDependencyTraces key is missing" >&2
  exit 1
fi

echo "replay cli json contract guard test passed"
