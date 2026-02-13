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

echo "replay cli json contract guard test passed"
