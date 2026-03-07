local default_task_id = "__BENCH_WB_TASK_ID_DEFAULT__"
local task_id = "__BENCH_WB_TASK_ID__"

if task_id == default_task_id or task_id == "" then
  task_id = os.getenv("BENCH_WB_TASK_ID") or "wb-seed-task"
end

request = function()
  return wrk.format("GET", "/wb/tasks/" .. task_id .. "?row_schema=1")
end
