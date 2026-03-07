#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--impl <name>] [--task-id <id>] [--run-tag <tag>] <endpoint> <base_url>

Probes one workbench benchmark endpoint with the same request shape used by the
load scripts and emits one JSON result row to stdout.
USAGE
}

impl=""
seed_task_id="${BENCH_WB_TASK_ID:-}"
run_tag="${BENCH_WB_RUN_TAG:-}"
endpoint=""
base_url=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --impl)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      impl="$2"
      shift 2
      ;;
    --impl=*)
      impl="${1#--impl=}"
      shift
      ;;
    --task-id)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      seed_task_id="$2"
      shift 2
      ;;
    --task-id=*)
      seed_task_id="${1#--task-id=}"
      shift
      ;;
    --run-tag)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      run_tag="$2"
      shift 2
      ;;
    --run-tag=*)
      run_tag="${1#--run-tag=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [ -z "$endpoint" ]; then
        endpoint="$1"
      elif [ -z "$base_url" ]; then
        base_url="$1"
      else
        echo "unknown argument: $1" >&2
        usage
        exit 2
      fi
      shift
      ;;
  esac
done

if [ -z "$endpoint" ] || [ -z "$base_url" ]; then
  usage
  exit 2
fi

auth_header='Authorization: Bearer token123'
body_file="$(mktemp "/tmp/workbench-preflight-body.XXXXXX")"
trap 'rm -f "$body_file"' EXIT

trim_preview() {
  local path="$1"
  if [ ! -f "$path" ]; then
    printf '%s' ""
    return
  fi
  tr '\r\n' '  ' <"$path" | sed 's/[[:space:]]\+/ /g' | cut -c1-240
}

emit_result() {
  local result="$1"
  local reason="$2"
  local observed_status_json="$3"
  local body_preview="$4"
  jq -nc \
    --arg impl "$impl" \
    --arg endpoint "$endpoint" \
    --arg method "$method" \
    --arg path "$path" \
    --arg requestUrl "${base_url}${path}" \
    --argjson expectedStatuses "$expected_statuses_json" \
    --argjson observedStatus "$observed_status_json" \
    --arg result "$result" \
    --arg reason "$reason" \
    --arg bodyPreview "$body_preview" \
    '{
      impl: (if $impl == "" then null else $impl end),
      endpoint: $endpoint,
      method: $method,
      path: $path,
      requestUrl: $requestUrl,
      expectedStatuses: $expectedStatuses,
      observedStatus: $observedStatus,
      result: $result,
      reason: (if $reason == "" then null else $reason end),
      bodyPreview: (if $bodyPreview == "" then null else $bodyPreview end)
    }'
}

method=""
path=""
expected_statuses_json='[]'
request_body=""
declare -a curl_headers
curl_headers=()
validation_status_expr='.ok == true'
validation_detail_expr='""'

case "$endpoint" in
  wb-tasks-post)
    if [ -z "$run_tag" ]; then
      echo "run tag is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    probe_task_id="wb-${run_tag}-task-1"
    params="$(jq -nc \
      --arg id "$probe_task_id" \
      --arg title "Task1" \
      --arg description "wrk" \
      --arg status "open" \
      --argjson priority 3 \
      --argjson created 1700000000001 \
      '[ $id, $title, $description, $status, $priority, $created ]')"
    label_params="$(jq -nc --arg id "$probe_task_id" '[ $id, "[]" ]')"
    params_uri="$(printf '%s' "$params" | jq -sRr @uri)"
    label_params_uri="$(printf '%s' "$label_params" | jq -sRr @uri)"
    method="POST"
    path="/wb/tasks?params=${params_uri}&label_params=${label_params_uri}"
    expected_statuses_json='[200,201]'
    request_body='{}'
    curl_headers=(-H "$auth_header" -H 'Content-Type: application/json')
    validation_status_expr='.ok == true and .data.id == $taskId'
    validation_detail_expr='".data.id=" + ((.data.id // "null") | tostring)'
    ;;
  wb-tasks-with-comment)
    if [ -z "$run_tag" ]; then
      echo "run tag is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    probe_task_id="wb-${run_tag}-task-tx-1"
    probe_comment_id="wb-${run_tag}-comment-tx-1"
    task_params="$(jq -nc \
      --arg id "$probe_task_id" \
      --arg title "TaskTx1" \
      --arg description "wrk-tx" \
      --arg status "in_progress" \
      --argjson priority 4 \
      --argjson created 1700000100001 \
      '[ $id, $title, $description, $status, $priority, $created ]')"
    comment_params="$(jq -nc \
      --arg id "$probe_comment_id" \
      --arg task_id "$probe_task_id" \
      --arg body "bench-comment" \
      --argjson created 1700000200001 \
      '[ $id, $task_id, $body, $created ]')"
    task_params_uri="$(printf '%s' "$task_params" | jq -sRr @uri)"
    comment_params_uri="$(printf '%s' "$comment_params" | jq -sRr @uri)"
    method="POST"
    path="/wb/tasks/with-comment?task_params=${task_params_uri}&comment_params=${comment_params_uri}"
    expected_statuses_json='[200,201]'
    request_body='{}'
    curl_headers=(-H "$auth_header" -H 'Content-Type: application/json')
    validation_status_expr='.ok == true and .data.taskId == $taskId and .data.commentId == $commentId'
    validation_detail_expr='".data.taskId=" + ((.data.taskId // "null") | tostring) + " data.commentId=" + ((.data.commentId // "null") | tostring)'
    ;;
  wb-tasks-with-comment-tx)
    if [ -z "$run_tag" ]; then
      echo "run tag is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    probe_task_id="wb-${run_tag}-task-tx-chain-1"
    probe_comment_id="wb-${run_tag}-comment-tx-chain-1"
    task_params="$(jq -nc \
      --arg id "$probe_task_id" \
      --arg title "TaskTxChain1" \
      --arg description "wrk-tx-chain" \
      --arg status "in_progress" \
      --argjson priority 4 \
      --argjson created 1700000300001 \
      '[ $id, $title, $description, $status, $priority, $created ]')"
    comment_params="$(jq -nc \
      --arg id "$probe_comment_id" \
      --arg task_id "$probe_task_id" \
      --arg body "bench-comment-chain" \
      --argjson created 1700000400001 \
      '[ $id, $task_id, $body, $created ]')"
    task_params_uri="$(printf '%s' "$task_params" | jq -sRr @uri)"
    comment_params_uri="$(printf '%s' "$comment_params" | jq -sRr @uri)"
    method="POST"
    path="/wb/tasks/with-comment-tx?task_params=${task_params_uri}&comment_params=${comment_params_uri}"
    expected_statuses_json='[200,201]'
    request_body='{}'
    curl_headers=(-H "$auth_header" -H 'Content-Type: application/json')
    validation_status_expr='.ok == true and .data.taskId == $taskId and .data.commentId == $commentId'
    validation_detail_expr='".data.taskId=" + ((.data.taskId // "null") | tostring) + " data.commentId=" + ((.data.commentId // "null") | tostring)'
    ;;
  wb-task-comment-post)
    if [ -z "$seed_task_id" ]; then
      echo "task id is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    if [ -z "$run_tag" ]; then
      echo "run tag is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    probe_comment_id="wb-${run_tag}-comment-1"
    params="$(jq -nc \
      --arg id "$probe_comment_id" \
      --arg task_id "$seed_task_id" \
      --arg body "bench-comment" \
      --argjson created 1700000300001 \
      '[ $id, $task_id, $body, $created ]')"
    params_uri="$(printf '%s' "$params" | jq -sRr @uri)"
    method="POST"
    path="/wb/tasks/${seed_task_id}/comments?params=${params_uri}"
    expected_statuses_json='[200,201]'
    request_body='{}'
    curl_headers=(-H "$auth_header" -H 'Content-Type: application/json')
    validation_status_expr='.ok == true and .data.id == $commentId'
    validation_detail_expr='".data.id=" + ((.data.id // "null") | tostring)'
    ;;
  wb-task-get)
    if [ -z "$seed_task_id" ]; then
      echo "task id is required for endpoint ${endpoint}" >&2
      exit 2
    fi
    method="GET"
    path="/wb/tasks/${seed_task_id}?row_schema=1"
    expected_statuses_json='[200]'
    validation_status_expr='.ok == true and .data.id == $taskId'
    validation_detail_expr='".data.id=" + ((.data.id // "null") | tostring)'
    ;;
  wb-tasks-list)
    method="GET"
    path='/wb/tasks?params=%5B%22open%22%2C20%2C0%5D&row_schema=1'
    expected_statuses_json='[200]'
    validation_status_expr='.ok == true and (.data.items | type == "array") and (.data.count | type == "number")'
    validation_detail_expr='".data.count=" + ((.data.count // "null") | tostring)'
    ;;
  *)
    echo "unsupported workbench endpoint probe: ${endpoint}" >&2
    exit 2
    ;;
esac

curl_status=0
if [ -n "$request_body" ]; then
  http_status="$(curl -sS -o "$body_file" -w '%{http_code}' -X "$method" "${curl_headers[@]}" --data "$request_body" "${base_url}${path}")" || curl_status=$?
else
  http_status="$(curl -sS -o "$body_file" -w '%{http_code}' -X "$method" "${curl_headers[@]}" "${base_url}${path}")" || curl_status=$?
fi

body_preview="$(trim_preview "$body_file")"
observed_status_json="null"
if [ -n "${http_status:-}" ] && [[ "$http_status" =~ ^[0-9]+$ ]]; then
  observed_status_json="$http_status"
fi

if [ "$curl_status" -ne 0 ]; then
  reason="endpoint=${endpoint} method=${method} path=${path} curlExit=${curl_status}"
  emit_result "failed" "$reason" "$observed_status_json" "$body_preview"
  exit 1
fi

if ! jq -e --argjson observed "${http_status}" '$expectedStatuses | index($observed) != null' --argjson expectedStatuses "$expected_statuses_json" >/dev/null 2>&1 <<<"null"; then
  reason="endpoint=${endpoint} method=${method} path=${path} status=${http_status} expected=$(jq -c '.' <<<"$expected_statuses_json")"
  if [ -n "$body_preview" ]; then
    reason="${reason} body=${body_preview}"
  fi
  emit_result "failed" "$reason" "$observed_status_json" "$body_preview"
  exit 1
fi

if ! jq -e '.' "$body_file" >/dev/null 2>&1; then
  reason="endpoint=${endpoint} method=${method} path=${path} status=${http_status} invalid_json_response"
  if [ -n "$body_preview" ]; then
    reason="${reason} body=${body_preview}"
  fi
  emit_result "failed" "$reason" "$observed_status_json" "$body_preview"
  exit 1
fi

validation_detail="$(
  jq -r \
    --arg taskId "${probe_task_id:-$seed_task_id}" \
    --arg commentId "${probe_comment_id:-}" \
    "$validation_detail_expr" \
    "$body_file" 2>/dev/null || true
)"

if ! jq -e \
  --arg taskId "${probe_task_id:-$seed_task_id}" \
  --arg commentId "${probe_comment_id:-}" \
  "$validation_status_expr" \
  "$body_file" >/dev/null 2>&1; then
  reason="endpoint=${endpoint} method=${method} path=${path} status=${http_status} invalid_success_payload"
  if [ -n "$validation_detail" ]; then
    reason="${reason} ${validation_detail}"
  fi
  if [ -n "$body_preview" ]; then
    reason="${reason} body=${body_preview}"
  fi
  emit_result "failed" "$reason" "$observed_status_json" "$body_preview"
  exit 1
fi

emit_result "passed" "" "$observed_status_json" "$body_preview"
