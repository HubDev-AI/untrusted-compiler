#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
capture_path="${root_dir}/captures/sample-capture.json"

usage() {
  cat >&2 <<USAGE
usage: $0 [--capture <path>]

Validates replay capture JSON contract shape.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --capture)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --capture" >&2
        usage
        exit 2
      fi
      capture_path="$2"
      shift 2
      ;;
    --capture=*)
      capture_path="${1#--capture=}"
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

if [[ ! -f "${capture_path}" ]]; then
  echo "missing capture file: ${capture_path}" >&2
  exit 1
fi

jq -e '
  .version == "0.1"
  and (.captureId | type == "string" and length > 0)
  and (.traceId | type == "string" and length > 0)
  and (.timeMs | type == "number")
  and (.policyHash | type == "string" and length > 0)
  and (.compilerHash | type == "string" and length > 0)
  and (.runtimeHash | type == "string" and length > 0)
  and (.request | type == "object")
  and (.request.method | type == "string" and length > 0)
  and (.request.path | type == "string")
  and (.request.headers | type == "object")
  and (.request.body | type == "object")
  and (.request.body.encoding | type == "string")
  and ((.request.body.truncated | type) == "boolean")
  and (
    if .request.body.encoding == "base64" then
      (.request.body.bytes | type == "string" and length > 0)
    elif .request.body.encoding == "none" then
      (.request.body.sha256 | type == "string" and length > 0)
    else
      false
    end
  )
  and (.env | type == "object")
  and (.determinism | type == "object")
  and (.determinism.seed | type == "number")
  and (.determinism.time.mode | type == "string")
  and (.determinism.time.nowMs | type == "number")
  and (.determinism.uuid.mode | type == "string")
  and (.determinism.budget.maxBodyBytes | type == "number")
  and (.determinism.budget.maxJsonBytes | type == "number")
  and (.determinism.budget.maxJsonDepth | type == "number")
  and (.determinism.budget.deadlineMs | type == "number")
  and (.redaction | type == "object")
  and (.redaction.headers | type == "array")
  and (.redaction.jsonPaths | type == "array")
  and (
    if has("dependencies") then
      (.dependencies | type == "object")
      and (
        if (.dependencies | has("db")) then
          (.dependencies.db | type == "array")
          and all(.dependencies.db[];
            (.request | type == "object")
            and (.request.queryTemplateId | type == "string" and length > 0)
            and (
              if (.request | has("paramsSha256"))
              then (.request.paramsSha256 | type == "string" and length > 0)
              else true
              end
            )
          )
        else true end
      )
      and (
        if (.dependencies | has("fs")) then
          (.dependencies.fs | type == "array")
          and all(.dependencies.fs[];
            (.request | type == "object")
            and (.request.op | type == "string" and length > 0)
            and (.request.pathSha256 | type == "string" and length > 0)
          )
        else true end
      )
    else true end
  )
' "${capture_path}" >/dev/null

if ! jq -e '
  if has("dependencies") and (.dependencies | has("db")) then
    (.dependencies.db | map([.request.queryTemplateId, (.request.paramsSha256 // "-")] | join("|"))) as $keys
    | ($keys | length) == ($keys | unique | length)
  else
    true
  end
' "${capture_path}" >/dev/null; then
  echo "replay capture contract failed: duplicate db dependency request signatures in ${capture_path}" >&2
  exit 1
fi

if ! jq -e '
  if has("dependencies") and (.dependencies | has("fs")) then
    (.dependencies.fs | map([(.request.op | ascii_downcase), .request.pathSha256] | join("|"))) as $keys
    | ($keys | length) == ($keys | unique | length)
  else
    true
  end
' "${capture_path}" >/dev/null; then
  echo "replay capture contract failed: duplicate fs dependency request signatures in ${capture_path}" >&2
  exit 1
fi

echo "replay capture contract check passed"
