#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script_path="${root_dir}/scripts/build-m37-alpha-publish-checklist-delta.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "missing required command: jq" >&2
  exit 1
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sec4-m37-checklist-test.XXXXXX")"
trap 'rm -rf "${tmp_dir}"' EXIT

release_dir="${tmp_dir}/release-alpha-gate"
mkdir -p "${release_dir}"

cat > "${release_dir}/summary.txt" <<'EOF_SUMMARY'
sec4 alpha release gate: PASS
naming lock: PASS
milestone closure: PASS
EOF_SUMMARY

cat > "${release_dir}/checksums.txt" <<'EOF_CHECKSUMS'
policy_identity_hash pol_hash_123
compiler_identity_hash cpl_hash_456
runtime_identity_hash rt_hash_789
EOF_CHECKSUMS

cat > "${release_dir}/hello-build_metadata.json" <<'EOF_HELLO_META'
{"policyHash":"pol_hash_123","compilerHash":"cpl_hash_456","runtimeHash":"rt_hash_789"}
EOF_HELLO_META

cat > "${release_dir}/hello-api-build_metadata.json" <<'EOF_HELLO_API_META'
{"policyHash":"pol_hash_123","compilerHash":"cpl_hash_456","runtimeHash":"rt_hash_789"}
EOF_HELLO_API_META

cat > "${release_dir}/hello-audit.json" <<'EOF_HELLO_AUDIT'
{
  "policy":{"hash":"pol_hash_123"},
  "build":{"compilerHash":"cpl_hash_456","runtimeHash":"rt_hash_789"},
  "summary":{"highestSeverity":"LOW","riskScore":0}
}
EOF_HELLO_AUDIT

cat > "${release_dir}/hello-api-audit.json" <<'EOF_HELLO_API_AUDIT'
{
  "policy":{"hash":"pol_hash_123"},
  "build":{"compilerHash":"cpl_hash_456","runtimeHash":"rt_hash_789"},
  "summary":{"highestSeverity":"LOW","riskScore":0}
}
EOF_HELLO_API_AUDIT

output_json="${tmp_dir}/delta.json"
output_md="${tmp_dir}/delta.md"

"${script_path}" \
  --release-gate-dir "${release_dir}" \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > "${tmp_dir}/stdout.json"

jq -e '.marker == "M37-S6" and .overall == "PASS" and .tagDecision == "GO"' "${output_json}" >/dev/null
rg -Fq "M37-S6" "${output_md}"
rg -Fq "Proceed with alpha tag decision and release-note publication." "${output_md}"

# Pending state when sample metadata hash mismatches.
cat > "${release_dir}/hello-build_metadata.json" <<'EOF_HELLO_META_MISMATCH'
{"policyHash":"different","compilerHash":"cpl_hash_456","runtimeHash":"rt_hash_789"}
EOF_HELLO_META_MISMATCH

"${script_path}" \
  --release-gate-dir "${release_dir}" \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --format json > /dev/null

jq -e '.overall == "PENDING" and .tagDecision == "HOLD"' "${output_json}" >/dev/null
jq -e '.samples[] | select(.sample == "hello") | .status == "PENDING"' "${output_json}" >/dev/null

# Contract failure when required artifact is missing.
rm -f "${release_dir}/summary.txt"
if "${script_path}" --release-gate-dir "${release_dir}" --output-json "${output_json}" --output-md "${output_md}" --format json > /dev/null 2>&1; then
  echo "expected missing summary contract failure" >&2
  exit 1
fi

echo "m37 alpha publish checklist delta test passed"
