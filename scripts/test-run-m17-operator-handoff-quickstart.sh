#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
quickstart_script="${root_dir}/scripts/run-m17-operator-handoff-quickstart.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts" "${repo_dir}/examples/hello-api"

cat > "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -z "${QUICKSTART_LOG:-}" ]; then
  echo "missing QUICKSTART_LOG" >&2
  exit 1
fi
printf 'readiness %s\n' "$*" >> "${QUICKSTART_LOG}"
SH
chmod +x "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh"

cat > "${repo_dir}/scripts/run-m17-operator-bootstrap.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -z "${QUICKSTART_LOG:-}" ]; then
  echo "missing QUICKSTART_LOG" >&2
  exit 1
fi
printf 'bootstrap %s\n' "$*" >> "${QUICKSTART_LOG}"
SH
chmod +x "${repo_dir}/scripts/run-m17-operator-bootstrap.sh"

cat > "${repo_dir}/scripts/check-milestone-closure.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -z "${QUICKSTART_LOG:-}" ]; then
  echo "missing QUICKSTART_LOG" >&2
  exit 1
fi
printf 'closure %s\n' "$*" >> "${QUICKSTART_LOG}"
SH
chmod +x "${repo_dir}/scripts/check-milestone-closure.sh"

cat > "${repo_dir}/scripts/print-m17-operator-troubleshooting-matrix.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ -z "${QUICKSTART_LOG:-}" ]; then
  echo "missing QUICKSTART_LOG" >&2
  exit 1
fi
printf 'matrix %s\n' "$*" >> "${QUICKSTART_LOG}"
SH
chmod +x "${repo_dir}/scripts/print-m17-operator-troubleshooting-matrix.sh"

artifacts_root="${tmp_dir}/runtime-smoke"
run_log="${tmp_dir}/quickstart.log"

QUICKSTART_LOG="${run_log}" \
"${quickstart_script}" \
  --repo-root "${repo_dir}" \
  --project "${repo_dir}/examples/hello-api" \
  --artifacts-root "${artifacts_root}" \
  --serve-timeout-ms 7777 \
  --max-body-bytes 3333 \
  >/dev/null

if [ "$(wc -l < "${run_log}" | tr -d ' ')" -ne 4 ]; then
  echo "expected quickstart flow to invoke readiness/bootstrap/closure/matrix in full mode" >&2
  exit 1
fi

line1="$(sed -n '1p' "${run_log}")"
line2="$(sed -n '2p' "${run_log}")"
line3="$(sed -n '3p' "${run_log}")"
line4="$(sed -n '4p' "${run_log}")"

if ! printf '%s\n' "${line1}" | rg -Fq -- 'readiness --repo-root '; then
  echo "expected quickstart to call readiness verifier first" >&2
  exit 1
fi
if ! printf '%s\n' "${line2}" | rg -Fq -- "bootstrap --repo-root ${repo_dir} --project ${repo_dir}/examples/hello-api --artifacts-root ${artifacts_root} --serve-timeout-ms 7777 --max-body-bytes 3333"; then
  echo "expected quickstart to call bootstrap helper with forwarded overrides" >&2
  exit 1
fi
if ! printf '%s\n' "${line3}" | rg -Fq -- "closure --repo-root ${repo_dir} --fail-on-pending"; then
  echo "expected quickstart to call closure checker after bootstrap helper" >&2
  exit 1
fi
if ! printf '%s\n' "${line4}" | rg -Fq -- 'matrix '; then
  echo "expected quickstart to print troubleshooting matrix by default" >&2
  exit 1
fi

run_no_matrix_log="${tmp_dir}/quickstart-no-matrix.log"
QUICKSTART_LOG="${run_no_matrix_log}" \
"${quickstart_script}" \
  --repo-root "${repo_dir}" \
  --project "${repo_dir}/examples/hello-api" \
  --artifacts-root "${artifacts_root}" \
  --no-matrix \
  >/dev/null

if [ "$(wc -l < "${run_no_matrix_log}" | tr -d ' ')" -ne 3 ]; then
  echo "expected quickstart --no-matrix flow to skip troubleshooting matrix invocation" >&2
  exit 1
fi
if rg -Fq 'matrix ' "${run_no_matrix_log}"; then
  echo "expected no matrix invocation when --no-matrix is provided" >&2
  exit 1
fi

cat > "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "forced readiness failure" >&2
exit 1
SH
chmod +x "${repo_dir}/scripts/check-m17-operator-handoff-readiness.sh"

failure_log="${tmp_dir}/quickstart-failure.log"
if QUICKSTART_LOG="${tmp_dir}/quickstart-failure-order.log" "${quickstart_script}" --repo-root "${repo_dir}" --project "${repo_dir}/examples/hello-api" >"${failure_log}" 2>&1; then
  echo "expected quickstart to fail when readiness verifier fails" >&2
  exit 1
fi
if ! rg -Fq 'forced readiness failure' "${failure_log}"; then
  echo "expected readiness failure diagnostic to bubble through quickstart" >&2
  exit 1
fi

echo "m17 operator handoff quickstart test passed"
