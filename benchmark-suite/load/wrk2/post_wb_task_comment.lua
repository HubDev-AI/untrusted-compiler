local seq = 0
local default_task_id = "__BENCH_WB_TASK_ID_DEFAULT__"
local default_run_tag = "__BENCH_WB_RUN_TAG_DEFAULT__"
local task_id = "__BENCH_WB_TASK_ID__"
local run_tag = "__BENCH_WB_RUN_TAG__"
local thread_tag = "t0"
local next_thread_id = 0

if task_id == default_task_id or task_id == "" then
  task_id = os.getenv("BENCH_WB_TASK_ID") or "wb-seed-task"
end
if run_tag == default_run_tag or run_tag == "" then
  run_tag = os.getenv("BENCH_WB_RUN_TAG") or tostring(os.time())
end

wrk.method = "POST"
wrk.body = "{}"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Authorization"] = "Bearer token123"

local function sanitize_thread_component(value)
  local sanitized = tostring(value or "")
  sanitized = sanitized:gsub("^thread:%s*", "")
  sanitized = sanitized:gsub("[^%w]+", "-")
  sanitized = sanitized:gsub("^-+", "")
  sanitized = sanitized:gsub("-+$", "")
  if sanitized == "" then
    return nil
  end
  return sanitized
end

local function resolve_thread_tag()
  local parts = {}
  local thread_index = sanitize_thread_component(thread_id)
  local thread_identity = sanitize_thread_component(wrk.thread)

  if thread_index ~= nil then
    parts[#parts + 1] = "i" .. thread_index
  end
  if thread_identity ~= nil then
    parts[#parts + 1] = thread_identity
  end
  if #parts == 0 then
    return "t0"
  end
  return "t" .. table.concat(parts, "-")
end

setup = function(thread)
  thread:set("thread_id", next_thread_id)
  next_thread_id = next_thread_id + 1
end

init = function(args)
  thread_tag = resolve_thread_tag()
end

request = function()
  seq = seq + 1
  local comment_id = "wb-" .. run_tag .. "-" .. thread_tag .. "-comment-" .. seq
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
