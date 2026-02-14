#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bootstrap_script="${root_dir}/scripts/run-m17-operator-bootstrap.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

repo_dir="${tmp_dir}/repo"
mkdir -p "${repo_dir}/scripts" "${repo_dir}/examples/hello-api"

cat > "${repo_dir}/scripts/smoke-sec4-run-hello-api.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [ -z "${SEC4_SMOKE_LOG:-}" ]; then
  echo "missing SEC4_SMOKE_LOG" >&2
  exit 1
fi

args="$*"
printf '%s\n' "${args}" >> "${SEC4_SMOKE_LOG}"

artifacts_dir=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --artifacts-dir)
      artifacts_dir="$2"
      shift 2
      ;;
    --artifacts-dir=*)
      artifacts_dir="${1#--artifacts-dir=}"
      shift
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "${artifacts_dir}" ]; then
  echo "missing --artifacts-dir in smoke stub" >&2
  exit 1
fi

mkdir -p "${artifacts_dir}"
printf 'ok\n' > "${artifacts_dir}/run-metadata.txt"
SH
chmod +x "${repo_dir}/scripts/smoke-sec4-run-hello-api.sh"

cat > "${repo_dir}/scripts/check-runtime-smoke-bundle.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [ -z "${SEC4_BUNDLE_LOG:-}" ]; then
  echo "missing SEC4_BUNDLE_LOG" >&2
  exit 1
fi

args="$*"
printf '%s\n' "${args}" >> "${SEC4_BUNDLE_LOG}"

artifacts_root=""
index_path=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --artifacts-root)
      artifacts_root="$2"
      shift 2
      ;;
    --artifacts-root=*)
      artifacts_root="${1#--artifacts-root=}"
      shift
      ;;
    --index-path)
      index_path="$2"
      shift 2
      ;;
    --index-path=*)
      index_path="${1#--index-path=}"
      shift
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "${artifacts_root}" ] || [ -z "${index_path}" ]; then
  echo "missing bundle args in bundle stub" >&2
  exit 1
fi

if [ ! -d "${artifacts_root}/default" ] || [ ! -d "${artifacts_root}/max-body" ]; then
  echo "missing branch dirs in bundle stub" >&2
  exit 1
fi

mkdir -p "$(dirname "${index_path}")"
cat > "${index_path}" <<'JSON'
{"branchOrder":["default","max-body"],"branches":[]}
JSON
SH
chmod +x "${repo_dir}/scripts/check-runtime-smoke-bundle.sh"

smoke_log="${tmp_dir}/smoke.log"
bundle_log="${tmp_dir}/bundle.log"
artifacts_root="${tmp_dir}/runtime-smoke"

SEC4_SMOKE_LOG="${smoke_log}" \
SEC4_BUNDLE_LOG="${bundle_log}" \
"${bootstrap_script}" \
  --repo-root "${repo_dir}" \
  --project "${repo_dir}/examples/hello-api" \
  --artifacts-root "${artifacts_root}" \
  --serve-timeout-ms 9000 \
  --max-body-bytes 4096 \
  >/dev/null

if [ "$(wc -l < "${smoke_log}" | tr -d ' ')" -ne 2 ]; then
  echo "expected bootstrap helper to invoke smoke script twice" >&2
  exit 1
fi

smoke_default_line="$(sed -n '1p' "${smoke_log}")"
smoke_max_body_line="$(sed -n '2p' "${smoke_log}")"

if ! printf '%s\n' "${smoke_default_line}" | rg -Fq -- "--artifacts-dir ${artifacts_root}/default"; then
  echo "expected first smoke invocation to target default artifacts dir" >&2
  exit 1
fi
if ! printf '%s\n' "${smoke_max_body_line}" | rg -Fq -- "--artifacts-dir ${artifacts_root}/max-body"; then
  echo "expected second smoke invocation to target max-body artifacts dir" >&2
  exit 1
fi
if ! printf '%s\n' "${smoke_default_line}" | rg -Fq -- "--serve-timeout-ms 9000"; then
  echo "expected default smoke invocation to propagate timeout override" >&2
  exit 1
fi
if ! printf '%s\n' "${smoke_max_body_line}" | rg -Fq -- "--serve-timeout-ms 9000"; then
  echo "expected max-body smoke invocation to propagate timeout override" >&2
  exit 1
fi
if printf '%s\n' "${smoke_default_line}" | rg -Fq -- "--max-body-bytes"; then
  echo "expected default smoke invocation to omit max-body flag" >&2
  exit 1
fi
if ! printf '%s\n' "${smoke_max_body_line}" | rg -Fq -- "--max-body-bytes 4096"; then
  echo "expected max-body smoke invocation to include max-body override" >&2
  exit 1
fi

if ! rg -Fq -- "--artifacts-root ${artifacts_root}" "${bundle_log}"; then
  echo "expected bundle checker invocation to include artifacts root" >&2
  exit 1
fi
if ! rg -Fq -- "--index-path ${artifacts_root}/runtime-smoke-branch-index.json" "${bundle_log}"; then
  echo "expected bundle checker invocation to include deterministic branch index path" >&2
  exit 1
fi

if ! jq -e '.branchOrder == ["default","max-body"]' "${artifacts_root}/runtime-smoke-branch-index.json" >/dev/null; then
  echo "expected bootstrap helper to produce branch index output" >&2
  exit 1
fi

rm -f "${repo_dir}/scripts/check-runtime-smoke-bundle.sh"
if SEC4_SMOKE_LOG="${smoke_log}" SEC4_BUNDLE_LOG="${bundle_log}" "${bootstrap_script}" --repo-root "${repo_dir}" --project "${repo_dir}/examples/hello-api" --artifacts-root "${artifacts_root}" >"${tmp_dir}/missing-bundle.log" 2>&1; then
  echo "expected bootstrap helper to fail when bundle checker script is missing" >&2
  exit 1
fi
if ! rg -Fq 'missing runtime smoke bundle checker:' "${tmp_dir}/missing-bundle.log"; then
  echo "expected missing-bundle diagnostic from bootstrap helper" >&2
  exit 1
fi

echo "m17 operator bootstrap profile helper test passed"
