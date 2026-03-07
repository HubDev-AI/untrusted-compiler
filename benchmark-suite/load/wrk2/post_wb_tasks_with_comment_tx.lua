local seq = 0
local default_run_tag = "__BENCH_WB_RUN_TAG_DEFAULT__"
local run_tag = "__BENCH_WB_RUN_TAG__"
local thread_tag = "t0"

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
  local thread_identity = sanitize_thread_component(wrk.thread)
  local table_identity = sanitize_thread_component(tostring({}))

  if thread_identity ~= nil then
    parts[#parts + 1] = thread_identity
  end
  if table_identity ~= nil then
    parts[#parts + 1] = table_identity
  end
  if #parts == 0 then
    return "t" .. tostring(math.random(100000, 999999))
  end
  return "t" .. table.concat(parts, "-")
end

init = function(args)
  local table_seed = tostring({}):gsub("%D", "")
  local seed_suffix = tonumber(table_seed:sub(-6)) or 0
  math.randomseed(os.time() + seed_suffix)
  thread_tag = resolve_thread_tag()
end

request = function()
  seq = seq + 1
  local task_id = "wb-" .. run_tag .. "-" .. thread_tag .. "-task-tx-chain-" .. seq
  local comment_id = "wb-" .. run_tag .. "-" .. thread_tag .. "-comment-tx-chain-" .. seq
  local created_task_ms = 1700000300000 + seq
  local created_comment_ms = 1700000400000 + seq

  local task_params = "%5B%22"
    .. task_id
    .. "%22%2C%22TaskTxChain"
    .. seq
    .. "%22%2C%22wrk-tx-chain%22%2C%22in_progress%22%2C4%2C"
    .. created_task_ms
    .. "%5D"
  local comment_params = "%5B%22"
    .. comment_id
    .. "%22%2C%22"
    .. task_id
    .. "%22%2C%22bench-comment-chain%22%2C"
    .. created_comment_ms
    .. "%5D"
  local path = "/wb/tasks/with-comment-tx?task_params=" .. task_params .. "&comment_params=" .. comment_params
  return wrk.format("POST", path)
end
