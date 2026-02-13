# go service

Go benchmark baseline service implementing:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
cd benchmark-suite/services/go
go run .
```

Default port: `8080` (override with `PORT`).

## Smoke test

```bash
benchmark-suite/services/go/smoke.sh
```

## Notes

- Current `/users` path is in-memory for contract coverage.
- DB-backed parity with benchmark schema is planned in a follow-up M10 slice.
