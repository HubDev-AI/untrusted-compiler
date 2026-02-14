#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rehearsal_script="${root_dir}/scripts/run-m17-operator-clean-clone-rehearsal.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts" "${repo_dir}/examples/hello-api"

make_stub() {
  local name="$1"
  local body="$2"
  cat > "${repo_dir}/scripts/${name}" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail
${body}
SCRIPT
  chmod +x "${repo_dir}/scripts/${name}"
}

make_stub "check-m17-operator-handoff-playbook.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "playbook %s\n" "$*" >> "${REHEARSAL_LOG}"
'

make_stub "run-m17-operator-handoff-quickstart.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "quickstart %s\n" "$*" >> "${REHEARSAL_LOG}"
'

make_stub "run-m17-operator-handoff-ci-smoke.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "ci-smoke %s\n" "$*" >> "${REHEARSAL_LOG}"
mkdir -p "${PWD}/build/operator-handoff-smoke"
'

make_stub "build-m17-operator-release-packet.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "release-packet %s\n" "$*" >> "${REHEARSAL_LOG}"
output_dir=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-dir)
      output_dir="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [ -n "${output_dir}" ]; then
  mkdir -p "${output_dir}"
  echo '{"version":"0.1"}' > "${output_dir}/release-packet.json"
fi
'

make_stub "summarize-m17-operator-handoff-readiness.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "summary %s\n" "$*" >> "${REHEARSAL_LOG}"
cat <<JSON
{"version":"0.1","closure":{"overall":"PASS","pendingCount":0}}
JSON
'

make_stub "check-milestone-closure.sh" '
if [ -z "${REHEARSAL_LOG:-}" ]; then
  echo "missing REHEARSAL_LOG" >&2
  exit 1
fi
printf "closure %s\n" "$*" >> "${REHEARSAL_LOG}"
cat <<JSON
{"overall":"PASS","pendingCount":0}
JSON
'

run_log="${tmp_dir}/rehearsal.log"
output_dir="${tmp_dir}/output"

REHEARSAL_LOG="${run_log}" \
"${rehearsal_script}" \
  --repo-root "${repo_dir}" \
  --skip-clone \
  --output-dir "${output_dir}" \
  >/dev/null

if [ ! -f "${output_dir}/rehearsal-report.json" ]; then
  echo "expected rehearsal script to emit rehearsal-report.json" >&2
  exit 1
fi

if ! jq -e '
  .overall == "PASS"
  and .cloneMode == "in-place"
  and .failedStep == "none"
  and (.steps | length) == 6
  and (.friction | length) == 0
' "${output_dir}/rehearsal-report.json" >/dev/null; then
  echo "expected deterministic PASS rehearsal report contract" >&2
  exit 1
fi

if [ "$(wc -l < "${run_log}" | tr -d ' ')" -ne 6 ]; then
  echo "expected rehearsal flow to execute exactly 6 scripted steps" >&2
  exit 1
fi

if ! sed -n '1p' "${run_log}" | rg -Fq -- 'playbook --repo-root '; then
  echo "expected playbook step to execute first" >&2
  exit 1
fi
if ! sed -n '2p' "${run_log}" | rg -Fq -- 'quickstart --repo-root '; then
  echo "expected quickstart step to execute second" >&2
  exit 1
fi
if ! sed -n '6p' "${run_log}" | rg -Fq -- 'closure --repo-root '; then
  echo "expected closure step to execute last" >&2
  exit 1
fi

cat > "${repo_dir}/scripts/run-m17-operator-handoff-ci-smoke.sh" <<'SHFAIL'
#!/usr/bin/env bash
set -euo pipefail
echo "forced ci smoke failure" >&2
exit 1
SHFAIL
chmod +x "${repo_dir}/scripts/run-m17-operator-handoff-ci-smoke.sh"

failure_output_dir="${tmp_dir}/failure-output"
if REHEARSAL_LOG="${tmp_dir}/failure-rehearsal.log" \
  "${rehearsal_script}" \
    --repo-root "${repo_dir}" \
    --skip-clone \
    --output-dir "${failure_output_dir}" \
    >"${tmp_dir}/failure.log" 2>&1; then
  echo "expected rehearsal script to fail when CI smoke step fails" >&2
  exit 1
fi

if ! jq -e '
  .overall == "FAIL"
  and .failedStep == "ci-smoke"
  and (.friction | length) == 1
  and .friction[0].stepId == "ci-smoke"
' "${failure_output_dir}/rehearsal-report.json" >/dev/null; then
  echo "expected failure rehearsal report to capture ci-smoke friction" >&2
  exit 1
fi
if ! jq -e '.friction[0].message | contains("forced ci smoke failure")' "${failure_output_dir}/rehearsal-report.json" >/dev/null; then
  echo "expected failure rehearsal friction message to include first diagnostic line" >&2
  exit 1
fi

echo "m17 operator clean-clone rehearsal test passed"
