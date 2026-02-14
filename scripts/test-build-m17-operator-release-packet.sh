#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
builder_script="${root_dir}/scripts/build-m17-operator-release-packet.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts"

cat > "${repo_dir}/scripts/summarize-m17-operator-handoff-readiness.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -n "${SUMMARY_CALL_LOG:-}" ]; then
  printf '%s\n' "$*" >> "${SUMMARY_CALL_LOG}"
fi
cat <<'JSON'
{
  "version": "0.1",
  "artifactsRoot": "tmp",
  "steps": {
    "readiness": "PASS",
    "ciSmoke": "PASS",
    "artifactInspector": "PASS",
    "closure": "PASS"
  },
  "inspector": {
    "branchCount": 2,
    "branches": []
  },
  "closure": {
    "overall": "PASS",
    "pendingCount": 0
  }
}
JSON
SH
chmod +x "${repo_dir}/scripts/summarize-m17-operator-handoff-readiness.sh"

cat > "${repo_dir}/scripts/check-milestone-closure.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
cat <<'JSON'
{
  "repo": ".",
  "overall": "PASS",
  "message": "all tracked closure checks satisfied",
  "pendingCount": 0,
  "gates": []
}
JSON
SH
chmod +x "${repo_dir}/scripts/check-milestone-closure.sh"

artifacts_root="${tmp_dir}/artifacts"
mkdir -p "${artifacts_root}/default" "${artifacts_root}/max-body"
cat > "${artifacts_root}/runtime-smoke-branch-index.json" <<'JSON'
{"branchOrder":["default","max-body"],"branches":[]}
JSON
cat > "${artifacts_root}/default/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
TXT
cat > "${artifacts_root}/max-body/run-metadata.txt" <<'TXT'
sourceProject=examples/hello-api
TXT

output_dir="${tmp_dir}/packet"
summary_call_log="${tmp_dir}/summary-call.log"

SUMMARY_CALL_LOG="${summary_call_log}" \
"${builder_script}" \
  --repo-root "${repo_dir}" \
  --artifacts-root "${artifacts_root}" \
  --output-dir "${output_dir}" \
  >/dev/null

if [ ! -f "${output_dir}/readiness-summary.json" ]; then
  echo "expected release packet builder to emit readiness-summary.json" >&2
  exit 1
fi
if [ ! -f "${output_dir}/closure-gates.json" ]; then
  echo "expected release packet builder to emit closure-gates.json" >&2
  exit 1
fi
if [ ! -f "${output_dir}/artifact-manifest.txt" ]; then
  echo "expected release packet builder to emit artifact-manifest.txt" >&2
  exit 1
fi
if [ ! -f "${output_dir}/release-packet.json" ]; then
  echo "expected release packet builder to emit release-packet.json" >&2
  exit 1
fi

if ! rg -Fq -- '--artifacts-root '"${artifacts_root}" "${summary_call_log}"; then
  echo "expected release packet builder to forward artifacts root to readiness summary script" >&2
  exit 1
fi
if ! rg -Fq -- 'runtime-smoke-branch-index.json' "${output_dir}/artifact-manifest.txt"; then
  echo "expected artifact manifest to include runtime-smoke branch index" >&2
  exit 1
fi

if ! jq -e '
  .version == "0.1"
  and .summaryOverall == "PASS"
  and .closureOverall == "PASS"
  and (.artifactCount | tonumber) >= 3
' "${output_dir}/release-packet.json" >/dev/null; then
  echo "expected deterministic release packet metadata json contract" >&2
  exit 1
fi

if "${builder_script}" --repo-root "${repo_dir}" --artifacts-root "${tmp_dir}/missing-artifacts" --output-dir "${output_dir}" >"${tmp_dir}/missing-artifacts.log" 2>&1; then
  echo "expected release packet builder to fail on missing artifacts root" >&2
  exit 1
fi
if ! rg -Fq 'missing artifacts root:' "${tmp_dir}/missing-artifacts.log"; then
  echo "expected missing-artifacts diagnostic from release packet builder" >&2
  exit 1
fi

echo "m17 operator release packet builder test passed"
