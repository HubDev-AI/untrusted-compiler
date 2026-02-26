local seq = 0
local task_id = os.getenv("BENCH_WB_TASK_ID") or "wb-seed-task"
local run_tag = os.getenv("BENCH_WB_RUN_TAG") or tostring(os.time())

wrk.method = "POST"
wrk.body = "{}"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Authorization"] = "Bearer token123"

request = function()
  seq = seq + 1
  local comment_id = "wb-" .. run_tag .. "-comment-" .. seq
  local created_comment_ms = 1700000300000 + seq
  local params = "%5B%22"
    .. comment_id
    .. "%22%2C%22"
    .. task_id
    .. "%22%2C%22bench-comment%22%2C"
    .. created_comment_ms
    .. "%5D"
  local path = "/wb/tasks/" .. task_id .. "/comments?params=" .. params
  return wrk.format("POST", path)
end
