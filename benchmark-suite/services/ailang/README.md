# ailang service

AILang benchmark service implementing:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
benchmark-suite/services/ailang/build.sh
PORT=8080 benchmark-suite/services/ailang/ailang-bench-server
```

## Smoke test

```bash
benchmark-suite/services/ailang/smoke.sh
```

## Notes

- This service compiles AILang source to generated C and links a benchmark-specific runtime adapter.
- The adapter provides contract-oriented HTTP/JSON behavior suitable for benchmark harness comparability.
