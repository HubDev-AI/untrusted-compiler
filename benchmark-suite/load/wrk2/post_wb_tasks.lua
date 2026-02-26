local seq = 0
local run_tag = os.getenv("BENCH_WB_RUN_TAG") or tostring(os.time())

wrk.method = "POST"
wrk.body = "{}"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Authorization"] = "Bearer token123"

request = function()
  seq = seq + 1
  local task_id = "wb-" .. run_tag .. "-task-" .. seq
  local title = "Task" .. seq
  local created_at_ms = 1700000000000 + seq
  local path = "/wb/tasks?params=%5B%22"
    .. task_id
    .. "%22%2C%22"
    .. title
    .. "%22%2C%22wrk%22%2C%22open%22%2C3%2C"
    .. created_at_ms
    .. "%5D"
  return wrk.format("POST", path)
end
