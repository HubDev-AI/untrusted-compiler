request = function()
  local user_id = os.getenv("BENCH_USER_ID") or "6f1c2e7c-9c4a-4d0d-8c1b-2b59a4c8f8e1"
  return wrk.format("GET", "/users/" .. user_id)
end
