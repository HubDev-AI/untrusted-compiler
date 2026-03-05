local seq = 0
local run_tag = os.getenv("BENCH_WB_RUN_TAG") or tostring(os.time())

wrk.method = "POST"
wrk.body = "{}"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Authorization"] = "Bearer token123"

request = function()
  seq = seq + 1
  local task_id = "wb-" .. run_tag .. "-task-tx-" .. seq
  local comment_id = "wb-" .. run_tag .. "-comment-tx-" .. seq
  local created_task_ms = 1700000100000 + seq
  local created_comment_ms = 1700000200000 + seq

  local task_params = "%5B%22"
    .. task_id
    .. "%22%2C%22TaskTx"
    .. seq
    .. "%22%2C%22wrk-tx%22%2C%22in_progress%22%2C4%2C"
    .. created_task_ms
    .. "%5D"
  local comment_params = "%5B%22"
    .. comment_id
    .. "%22%2C%22"
    .. task_id
    .. "%22%2C%22bench-comment%22%2C"
    .. created_comment_ms
    .. "%5D"
  local path = "/wb/tasks/with-comment?task_params=" .. task_params .. "&comment_params=" .. comment_params
  return wrk.format("POST", path)
end
