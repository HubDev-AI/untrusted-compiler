#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script_path="${root_dir}/scripts/build-m37-alpha-release-notes.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sec4-m37-release-notes-test.XXXXXX")"
trap 'rm -rf "${tmp_dir}"' EXIT

checklist_path="${tmp_dir}/m37-checklist.json"
cat > "${checklist_path}" <<'EOF_CHECKLIST_PASS'
{
  "version": "0.1",
  "marker": "M37-S6",
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  },
  "overall": "PASS",
  "tagDecision": "GO",
  "samples": [
    {"sample":"hello","status":"PASS","evidence":"ok","highestSeverity":"LOW"},
    {"sample":"hello-api","status":"PASS","evidence":"ok","highestSeverity":"LOW"}
  ]
}
EOF_CHECKLIST_PASS

output_json="${tmp_dir}/release-notes.json"
output_md="${tmp_dir}/release-notes.md"

"${script_path}" \
  --checklist-json "${checklist_path}" \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > "${tmp_dir}/stdout.json"

jq -e '.marker == "M37-S7" and .releaseState == "alpha-ready" and .checklist.overall == "PASS" and .checklist.tagDecision == "GO"' "${output_json}" >/dev/null
rg -Fq "M37-S7" "${output_md}"
rg -Fq "No-stub alpha baseline is verified and ready for candidate tagging." "${output_md}"

# HOLD path
cat > "${checklist_path}" <<'EOF_CHECKLIST_HOLD'
{
  "version": "0.1",
  "marker": "M37-S6",
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  },
  "overall": "PENDING",
  "tagDecision": "HOLD",
  "samples": [
    {"sample":"hello","status":"PENDING","evidence":"mismatch","highestSeverity":"LOW"}
  ]
}
EOF_CHECKLIST_HOLD

"${script_path}" \
  --checklist-json "${checklist_path}" \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > /dev/null

jq -e '.releaseState == "alpha-hold" and .checklist.tagDecision == "HOLD" and .samples.pending == 1' "${output_json}" >/dev/null

# Contract failure on invalid checklist marker.
cat > "${checklist_path}" <<'EOF_CHECKLIST_INVALID'
{"marker":"WRONG"}
EOF_CHECKLIST_INVALID

if "${script_path}" --checklist-json "${checklist_path}" --output-json "${output_json}" --output-md "${output_md}" --format json > /dev/null 2>&1; then
  echo "expected invalid checklist contract failure" >&2
  exit 1
fi

echo "m37 alpha release notes test passed"
