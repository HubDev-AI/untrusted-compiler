#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script_path="${root_dir}/scripts/build-m37-alpha-tag-decision-record.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sec4-m37-tag-decision-test.XXXXXX")"
trap 'rm -rf "${tmp_dir}"' EXIT

checklist_json="${tmp_dir}/checklist.json"
release_notes_json="${tmp_dir}/release-notes.json"
output_json="${tmp_dir}/decision.json"
output_md="${tmp_dir}/decision.md"

cat > "${checklist_json}" <<'EOF_CHECKLIST_PASS'
{
  "marker": "M37-S6",
  "overall": "PASS",
  "tagDecision": "GO",
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  }
}
EOF_CHECKLIST_PASS

cat > "${release_notes_json}" <<'EOF_NOTES_PASS'
{
  "marker": "M37-S7",
  "releaseState": "alpha-ready",
  "checklist": {
    "overall": "PASS",
    "tagDecision": "GO"
  },
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  }
}
EOF_NOTES_PASS

"${script_path}" \
  --checklist-json "${checklist_json}" \
  --release-notes-json "${release_notes_json}" \
  --decision go \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > "${tmp_dir}/stdout.json"

jq -e '.marker == "M37-S8" and .decision.executed == "GO" and .closure.outcome == "PASS"' "${output_json}" >/dev/null
rg -Fq "M37-S8" "${output_md}"
rg -Fq 'outcome: `PASS`' "${output_md}"

# HOLD path should remain valid and produce pending closure.
cat > "${checklist_json}" <<'EOF_CHECKLIST_HOLD'
{
  "marker": "M37-S6",
  "overall": "PENDING",
  "tagDecision": "HOLD",
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  }
}
EOF_CHECKLIST_HOLD

cat > "${release_notes_json}" <<'EOF_NOTES_HOLD'
{
  "marker": "M37-S7",
  "releaseState": "alpha-hold",
  "checklist": {
    "overall": "PENDING",
    "tagDecision": "HOLD"
  },
  "identity": {
    "policyHash": "pol_hash_123",
    "compilerHash": "cpl_hash_456",
    "runtimeHash": "rt_hash_789"
  }
}
EOF_NOTES_HOLD

"${script_path}" \
  --checklist-json "${checklist_json}" \
  --release-notes-json "${release_notes_json}" \
  --decision hold \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > /dev/null

jq -e '.decision.executed == "HOLD" and .closure.outcome == "PENDING"' "${output_json}" >/dev/null

# GO must be blocked when prerequisites are not PASS/GO.
if "${script_path}" \
  --checklist-json "${checklist_json}" \
  --release-notes-json "${release_notes_json}" \
  --decision go \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > /dev/null 2>&1; then
  echo "expected GO prerequisite failure for hold checklist" >&2
  exit 1
fi

echo "m37 alpha tag decision record test passed"
