#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stubs_path="${root_dir}/captures/sample-replay-stubs.json"

usage() {
  cat >&2 <<USAGE
usage: $0 [--stubs <path>]

Validates replay stub registry contract for deterministic mock-mode bootstrap.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --stubs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      stubs_path="$2"
      shift 2
      ;;
    --stubs=*)
      stubs_path="${1#--stubs=}"
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

if [[ ! -f "${stubs_path}" ]]; then
  echo "missing replay stubs file: ${stubs_path}" >&2
  exit 1
fi

if ! jq -e '
  .version == "0.1"
  and (.stubs | type == "object")
  and (.stubs.net | type == "array")
  and (
    if (.stubs | has("db")) then (.stubs.db | type == "array") else true end
  )
  and (
    if (.stubs | has("fs")) then (.stubs.fs | type == "array") else true end
  )
  and (.redaction | type == "object")
  and (.redaction.headers | type == "array")
  and (.redaction.jsonPaths | type == "array")
  and all(.stubs.net[];
    (.request | type == "object")
    and (.request.method | type == "string" and length > 0)
    and (.request.url | type == "string" and length > 0)
    and (
      if (.request | has("bodySha256"))
      then (.request.bodySha256 | type == "string" and length > 0)
      else true
      end
    )
    and (.response | type == "object")
    and (.response.status | type == "number")
    and (.response.status >= 100 and .response.status <= 599)
    and (.response.truncated | type == "boolean")
    and (
      ((.response | has("bodyBase64")) and (.response.bodyBase64 | type == "string" and length > 0))
      or
      ((.response | has("bodySha256")) and (.response.bodySha256 | type == "string" and length > 0))
    )
  )
' "${stubs_path}" >/dev/null; then
  echo "replay stub registry contract failed for ${stubs_path}" >&2
  exit 1
fi

if ! jq -e '
  .stubs.net as $entries
  | (
      $entries
      | map([.request.method, .request.url, (.request.bodySha256 // "-")] | join("|"))
    ) as $keys
  | ($keys | length) == ($keys | unique | length)
' "${stubs_path}" >/dev/null; then
  echo "replay stub registry contract failed: duplicate net stub request signatures in ${stubs_path}" >&2
  exit 1
fi

echo "replay stub registry contract check passed"
