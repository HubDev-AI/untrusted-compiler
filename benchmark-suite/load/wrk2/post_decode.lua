wrk.method = "POST"
wrk.body = assert(io.open("benchmark-suite/spec/payloads/user_4kb.json", "r")):read("*a")
wrk.headers["Content-Type"] = "application/json"
wrk.path = "/decode"
