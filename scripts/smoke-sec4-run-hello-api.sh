#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--project <path>] [--artifacts-dir <path>]

Builds and runs a temporary hello-api service via `sec4 run` in oneshot mode,
then validates GET /health and POST /users end-to-end responses.
USAGE
}

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_project="${root_dir}/examples/hello-api"
artifacts_dir=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --project)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      source_project="$2"
      shift 2
      ;;
    --project=*)
      source_project="${1#--project=}"
      shift
      ;;
    --artifacts-dir)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      artifacts_dir="$2"
      shift 2
      ;;
    --artifacts-dir=*)
      artifacts_dir="${1#--artifacts-dir=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "missing required command: ${name}" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd curl
require_cmd jq
require_cmd python3

if [ ! -d "${source_project}" ]; then
  echo "missing project directory: ${source_project}" >&2
  exit 1
fi

pick_free_port() {
  python3 - <<'PY'
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()
PY
}

tmp_dir="$(mktemp -d)"
cleanup() {
  if [ -n "${service_pid:-}" ] && kill -0 "${service_pid}" >/dev/null 2>&1; then
    kill "${service_pid}" >/dev/null 2>&1 || true
    wait "${service_pid}" >/dev/null 2>&1 || true
  fi
  if [ -n "${artifacts_dir}" ]; then
    mkdir -p "${artifacts_dir}"
    for artifact in run-metadata.txt health.headers health.body health.run.log users.headers users.body users.run.log; do
      if [ -f "${tmp_dir}/${artifact}" ]; then
        cp "${tmp_dir}/${artifact}" "${artifacts_dir}/${artifact}"
      fi
    done
  fi
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

work_project="${tmp_dir}/hello-api"
cp -R "${source_project}" "${work_project}"

port="$(pick_free_port)"
main_file="${work_project}/src/main.ut"
perl -0pi -e "s/http\\.serve\\(8080, router\\);/http.serve(${port}, router);/" "${main_file}"

if ! rg -q "http\.serve\(${port}, router\);" "${main_file}"; then
  echo "failed to patch runtime port in ${main_file}" >&2
  exit 1
fi

cat > "${tmp_dir}/run-metadata.txt" <<META
sourceProject=${source_project}
workProject=${work_project}
port=${port}
META

request_once() {
  local name="$1"
  local method="$2"
  local path="$3"
  local expected_status="$4"
  local headers_file="${tmp_dir}/${name}.headers"
  local body_file="${tmp_dir}/${name}.body"
  local log_file="${tmp_dir}/${name}.run.log"
  shift 4

  local -a curl_args=(
    --silent --show-error
    --output "${body_file}"
    --dump-header "${headers_file}"
    --write-out "%{http_code}"
    --request "${method}"
    "$@"
    "http://127.0.0.1:${port}${path}"
  )

  cargo run -p sec4 -- run --path "${work_project}" --oneshot --serve-timeout-ms 12000 >"${log_file}" 2>&1 &
  service_pid=$!

  local status_code=""
  local got_response=0
  for _ in $(seq 1 320); do
    if ! kill -0 "${service_pid}" >/dev/null 2>&1; then
      echo "sec4 run exited before ${name} request completed" >&2
      cat "${log_file}" >&2
      exit 1
    fi

    status_code="$(curl "${curl_args[@]}" 2>/dev/null || true)"
    if [ -n "${status_code}" ] && [ "${status_code}" != "000" ]; then
      got_response=1
      break
    fi
    sleep 0.03
  done

  if [ "${got_response}" -ne 1 ]; then
    echo "did not receive HTTP response for ${name}" >&2
    cat "${log_file}" >&2
    exit 1
  fi

  local wait_status=""
  for _ in $(seq 1 240); do
    if wait "${service_pid}" >/dev/null 2>&1; then
      wait_status="ok"
      break
    fi
    sleep 0.02
  done
  service_pid=""

  if [ -z "${wait_status}" ]; then
    echo "sec4 run did not exit after oneshot request for ${name}" >&2
    cat "${log_file}" >&2
    exit 1
  fi

  if [ "${status_code}" != "${expected_status}" ]; then
    echo "unexpected HTTP status for ${name}: got ${status_code}, expected ${expected_status}" >&2
    cat "${headers_file}" >&2
    cat "${body_file}" >&2
    exit 1
  fi

  echo "${body_file}"
}

health_body="$(request_once \
  "health" \
  "GET" \
  "/health" \
  "200" \
  --header "Authorization: Bearer smoke-token" \
  --header "Connection: close"
)"

if [ "$(cat "${health_body}")" != "ok" ]; then
  echo "unexpected /health body" >&2
  cat "${health_body}" >&2
  exit 1
fi

users_body="$(request_once \
  "users" \
  "POST" \
  "/users" \
  "201" \
  --header "Authorization: Bearer smoke-token" \
  --header "Content-Type: application/json" \
  --header "X-CSRF-Token: token123" \
  --header "Cookie: csrf=token123" \
  --header "Connection: close" \
  --data "{}"
)"

if ! jq -e '.ok == true and .status == 201 and (.traceId | type == "string" and length > 0) and has("data")' "${users_body}" >/dev/null; then
  echo "unexpected /users JSON envelope" >&2
  cat "${users_body}" >&2
  exit 1
fi

echo "sec4 run hello-api smoke passed (port ${port})"
