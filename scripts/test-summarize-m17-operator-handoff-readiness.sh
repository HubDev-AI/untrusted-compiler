#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
summary_script="${root_dir}/scripts/summarize-m17-operator-handoff-readiness.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts"

cat > "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "m17 operator handoff readiness check passed"
SH
chmod +x "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh"

cat > "${repo_dir}/scripts/run-m17-operator-handoff-ci-smoke.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -z "${SUMMARY_LOG:-}" ]; then
  echo "missing SUMMARY_LOG" >&2
  exit 1
fi
printf 'ci-smoke %s\n' "$*" >> "${SUMMARY_LOG}"
SH
chmod +x "${repo_dir}/scripts/run-m17-operator-handoff-ci-smoke.sh"

cat > "${repo_dir}/scripts/inspect-m17-operator-handoff-artifacts.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
cat <<'JSON'
{
  "version": "0.1",
  "artifactsRoot": "tmp",
  "indexPath": "tmp/runtime-smoke-branch-index.json",
  "branchCount": 2,
  "branches": [
    {"name":"default","maxBodyBytes":"unset","runFlags":"--port,--oneshot,--serve-timeout-ms","port":"8080","serveTimeoutMs":"12000","healthStatus":"HTTP/1.1 200 OK","usersStatus":"HTTP/1.1 201 Created"},
    {"name":"max-body","maxBodyBytes":"2048","runFlags":"--port,--oneshot,--serve-timeout-ms,--max-body-bytes","port":"8081","serveTimeoutMs":"12000","healthStatus":"HTTP/1.1 200 OK","usersStatus":"HTTP/1.1 201 Created"}
  ]
}
JSON
SH
chmod +x "${repo_dir}/scripts/inspect-m17-operator-handoff-artifacts.sh"

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

summary_log="${tmp_dir}/summary.log"
artifacts_root="${tmp_dir}/artifacts"

json_output="$(
  SUMMARY_LOG="${summary_log}" \
  "${summary_script}" \
    --repo-root "${repo_dir}" \
    --artifacts-root "${artifacts_root}" \
    --format json
)"

if ! printf '%s\n' "${json_output}" | jq -e '
  .version == "0.1"
  and .steps.readiness == "PASS"
  and .steps.ciSmoke == "PASS"
  and .steps.artifactInspector == "PASS"
  and .steps.closure == "PASS"
  and .inspector.branchCount == 2
  and .closure.overall == "PASS"
  and .closure.pendingCount == 0
' >/dev/null; then
  echo "expected deterministic readiness summary json output contract" >&2
  exit 1
fi

if ! rg -Fq -- "ci-smoke --repo-root ${repo_dir} --artifacts-root ${artifacts_root}" "${summary_log}"; then
  echo "expected summary script to invoke ci smoke wrapper with deterministic args" >&2
  exit 1
fi

text_output="$(
  SUMMARY_LOG="${summary_log}" \
  "${summary_script}" \
    --repo-root "${repo_dir}" \
    --artifacts-root "${artifacts_root}" \
    --format text
)"
if ! printf '%s\n' "${text_output}" | rg -Fq 'M17 Operator Handoff Readiness Summary'; then
  echo "expected readiness summary text heading" >&2
  exit 1
fi
if ! printf '%s\n' "${text_output}" | rg -Fq 'closureOverall: PASS'; then
  echo "expected readiness summary text to include closure overall status" >&2
  exit 1
fi

cat > "${repo_dir}/scripts/check-milestone-closure.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "closure failure" >&2
exit 1
SH
chmod +x "${repo_dir}/scripts/check-milestone-closure.sh"

if SUMMARY_LOG="${summary_log}" "${summary_script}" --repo-root "${repo_dir}" --artifacts-root "${artifacts_root}" --format json >"${tmp_dir}/closure-failure.log" 2>&1; then
  echo "expected readiness summary script to fail when closure checker fails" >&2
  exit 1
fi
if ! rg -Fq 'closure failure' "${tmp_dir}/closure-failure.log"; then
  echo "expected closure failure diagnostic to bubble through readiness summary script" >&2
  exit 1
fi

echo "m17 operator handoff readiness summary test passed"
