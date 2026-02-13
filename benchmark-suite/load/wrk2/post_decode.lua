local payload_file = os.getenv("BENCH_PAYLOAD_FILE") or "spec/payloads/user_4kb.json"
local payload_handle = assert(io.open(payload_file, "r"))

wrk.method = "POST"
wrk.body = payload_handle:read("*a")
payload_handle:close()
wrk.headers["Content-Type"] = "application/json"
wrk.path = "/decode"
