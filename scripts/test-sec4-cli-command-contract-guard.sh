#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-sec4-cli-command-contract.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

cli_path="${tmp}/main.rs"

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
    Replay {
        capture: String,
        effects: ReplayEffectsMode,
        format: ReplayOutputFormat,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=HIGH"));
    None
}
RS

"${contract_script}" --cli "${cli_path}" >/dev/null

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
    Replay {
        capture: String,
        effects: ReplayEffectsMode,
        format: ReplayOutputFormat,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=MEDIUM"));
    None
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected command contract failure when gate threshold default changes" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
    Replay {
        capture: String,
        effects: ReplayEffectsMode,
        format: ReplayOutputFormat,
    },
    Sec {
        cmd: String,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=HIGH"));
    None
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected command contract failure when legacy sec alias returns" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=HIGH"));
    None
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected command contract failure when replay subcommand is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
    Replay {
        capture: String,
        format: ReplayOutputFormat,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=HIGH"));
    None
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected command contract failure when replay effects mode field is missing" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

enum ReplayOutputFormat {
    Text,
    Json,
}

enum Commands {
    Audit(AuditArgs),
    Gate {
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
    Replay {
        capture: String,
        effects: ReplayEffectsMode,
    },
}

fn gate_default(fail_on: Option<String>) -> Option<String> {
    let _x = Some(fail_on.as_deref().unwrap_or("risk>=HIGH"));
    None
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected command contract failure when replay output format field is missing" >&2
  exit 1
fi

echo "sec4 CLI command contract guard test passed"
