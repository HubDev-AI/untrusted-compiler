# c service

C floor-reference benchmark service implementing:
- `GET /ping`
- `POST /decode`
- `POST /users`
- `GET /users/:id`

## Run

```bash
cd benchmark-suite/services/c
cc -O2 -std=c11 server.c -o c-bench-server
./c-bench-server
```

Default port: `8080` (override with `PORT`).

## Smoke test

```bash
benchmark-suite/services/c/smoke.sh
```

## Notes

- This is a minimal baseline floor implementation using a simple socket server and in-memory storage.
- It intentionally favors low dependency overhead over feature completeness.
