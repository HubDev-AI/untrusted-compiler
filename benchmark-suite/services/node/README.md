# node service

Node benchmark service implementing the shared endpoint contract:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
cd benchmark-suite/services/node
npm run start
```

Default port: `8080` (override with `PORT`).

## Smoke test

```bash
benchmark-suite/services/node/smoke.sh
```

## Notes

- Current implementation uses in-memory storage for `/users` contract wiring.
- DB-backed parity with benchmark Postgres schema is planned in a follow-up M10 slice.
