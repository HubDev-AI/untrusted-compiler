# 01 - Zed Plugin Workflow

This track sets up the official Zed extension and verifies the language server flow.

Primary references:

- `zed-extension/README.md`
- `examples/zed-plugin-smoke/README.md`

## 1) Install extension locally

From repo root:

```bash
scripts/manage-zed-extension-local.sh status
scripts/manage-zed-extension-local.sh install
scripts/manage-zed-extension-local.sh status
```

For live local dev:

```bash
scripts/manage-zed-extension-local.sh install --mode symlink
```

## 2) Run plugin smoke

```bash
scripts/run-zed-plugin-smoke.sh --fast
```

This validates:

- diagnostics
- formatting
- definitions/references
- extension binary resolution

## 3) Open smoke fixture in Zed

Open:

- `examples/zed-plugin-smoke`

Perform:

1. Open `playground/lsp-errors.ut` and apply suggested quickfix.
2. Open `playground/format-me.ut` and run **Format Document**.
3. In `src/main.ut`, go-to-definition on `healthHandler`.
4. In `src/auth/access.ut`, run references/rename on `canReadUsers`.

## 4) Typical edit loop

```bash
sec4 fmt --path examples/zed-plugin-smoke
sec4 check --path examples/zed-plugin-smoke
```

## 5) Troubleshooting

- If the language server does not start:
  - confirm `which sec4audit-language-server`
  - run `scripts/manage-zed-extension-local.sh status`
- If extension update regresses:
  - run `scripts/manage-zed-extension-local.sh rollback`

## Next step

- Continue to `02-build-your-first-server.md`.

