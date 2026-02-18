# sec4-lasm service

Untrusted<T> benchmark service using the LASM backend at runtime.
It implements:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
cargo run -q -p sec4 -- run --path benchmark-suite/services/sec4-lasm --backend lasm --port 8080
```

## Smoke test

```bash
benchmark-suite/services/sec4-lasm/smoke.sh
```

## Notes

- This service exercises the same benchmark contract as `services/sec4` but executes through the LASM backend directly.
- Current LASM benchmark fixture materializes request-aware benchmark JSON contracts (`/decode`, `/users`, `/users/:id`) from LASM route schemas while preserving status-code behavior.
- It is intended for side-by-side C-runtime vs LASM benchmark orchestration.
