local task_id = os.getenv("BENCH_WB_TASK_ID") or "wb-seed-task"

request = function()
  return wrk.format("GET", "/wb/tasks/" .. task_id .. "?row_schema=1")
end
