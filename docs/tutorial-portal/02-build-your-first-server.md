# 02 - Build Your First Server

This is the shortest end-to-end path:

1. create app
2. check
3. build
4. run
5. hit endpoint

## 1) Create project

```bash
APP_DIR=/tmp/sec4-first-server
rm -rf "$APP_DIR"
./target/debug/sec4 init --path "$APP_DIR" --name sec4_first_server
```

Generated files:

- `sec4.toml`
- `sec4.policy`
- `src/main.ut`

## 2) Validate and build

```bash
./target/debug/sec4 check --path "$APP_DIR"
./target/debug/sec4 build --path "$APP_DIR" --emit lasm
```

## 3) Run server (LASM)

```bash
./target/debug/sec4 run \
  --path "$APP_DIR" \
  --backend lasm \
  --port 18080
```

## 4) Verify endpoint

```bash
curl -i http://127.0.0.1:18080/health
```

If your sample template serves a different route, check the generated `src/main.ut` and call that route.

## 5) One-request deterministic run

Useful for CI/debug:

```bash
./target/debug/sec4 run \
  --path "$APP_DIR" \
  --backend lasm \
  --oneshot \
  --port 18080
```

## Next step

- Continue to `03-real-db-postgres-lasm.md`.

