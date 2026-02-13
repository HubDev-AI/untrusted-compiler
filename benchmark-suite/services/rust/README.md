# rust service

Rust benchmark baseline service implementing:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
cd benchmark-suite/services/rust
cargo run
```

Default port: `8080` (override with `PORT`).

## Smoke test

```bash
benchmark-suite/services/rust/smoke.sh
```

## Notes

- Current `/users` path is in-memory for contract coverage.
- DB-backed parity with benchmark schema is planned in a follow-up M10 slice.
