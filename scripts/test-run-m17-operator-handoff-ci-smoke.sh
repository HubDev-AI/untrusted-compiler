#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ci_smoke_script="${root_dir}/scripts/run-m17-operator-handoff-ci-smoke.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts" "${repo_dir}/examples/hello-api"

cat > "${repo_dir}/scripts/run-m17-operator-handoff-quickstart.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [ -z "${CI_SMOKE_LOG:-}" ]; then
  echo "missing CI_SMOKE_LOG" >&2
  exit 1
fi

printf '%s\n' "$*" >> "${CI_SMOKE_LOG}"
SH
chmod +x "${repo_dir}/scripts/run-m17-operator-handoff-quickstart.sh"

log_path="${tmp_dir}/ci-smoke.log"
artifacts_root="${tmp_dir}/runtime-smoke"

CI_SMOKE_LOG="${log_path}" \
"${ci_smoke_script}" \
  --repo-root "${repo_dir}" \
  --project "${repo_dir}/examples/hello-api" \
  --artifacts-root "${artifacts_root}" \
  --serve-timeout-ms 8888 \
  --max-body-bytes 4444 \
  >/dev/null

if [ "$(wc -l < "${log_path}" | tr -d ' ')" -ne 1 ]; then
  echo "expected CI smoke wrapper to invoke quickstart exactly once" >&2
  exit 1
fi

line="$(cat "${log_path}")"
if ! printf '%s\n' "${line}" | rg -Fq -- "--repo-root ${repo_dir} --project ${repo_dir}/examples/hello-api --artifacts-root ${artifacts_root} --no-matrix --serve-timeout-ms 8888 --max-body-bytes 4444"; then
  echo "expected CI smoke wrapper to forward deterministic quickstart args and --no-matrix flag" >&2
  exit 1
fi

log_default_root="${tmp_dir}/ci-smoke-default-root.log"
CI_SMOKE_LOG="${log_default_root}" \
"${ci_smoke_script}" \
  --repo-root "${repo_dir}" \
  --project "${repo_dir}/examples/hello-api" \
  >/dev/null

if ! rg -q -- '--artifacts-root /' "${log_default_root}"; then
  echo "expected CI smoke wrapper to provide generated artifacts root when not passed explicitly" >&2
  exit 1
fi
if ! rg -Fq -- '--no-matrix' "${log_default_root}"; then
  echo "expected CI smoke wrapper to force --no-matrix in generated-root mode" >&2
  exit 1
fi

rm -f "${repo_dir}/scripts/run-m17-operator-handoff-quickstart.sh"
if CI_SMOKE_LOG="${tmp_dir}/ci-smoke-missing.log" "${ci_smoke_script}" --repo-root "${repo_dir}" --project "${repo_dir}/examples/hello-api" >"${tmp_dir}/missing.log" 2>&1; then
  echo "expected CI smoke wrapper to fail when quickstart script is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing M17 handoff quickstart script:' "${tmp_dir}/missing.log"; then
  echo "expected missing-quickstart diagnostic from CI smoke wrapper" >&2
  exit 1
fi

echo "m17 operator handoff ci smoke test passed"
