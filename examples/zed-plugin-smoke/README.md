# Zed Plugin Smoke Project

This project is a focused smoke fixture for the `zed-extension` + `sec4audit-language-server` workflow.

## What this project covers

- multi-file module resolution (`use http.handlers`, `use auth.access`, `use data.user_store`)
- go-to-definition / references / rename across files
- diagnostics + quickfixes in a deliberately broken playground file
- document formatting via LSP (`textDocument/formatting`)

## Prerequisites

1. Build the language server and CLI:

```bash
cargo build -p sec4audit-language-server -p sec4
```

2. Ensure binaries are available in `PATH` (or run Zed from this workspace where they are resolvable):

```bash
which sec4audit-language-server
which sec4
```

## CLI sanity checks

From repository root:

```bash
sec4 check --path examples/zed-plugin-smoke
sec4 fmt --path examples/zed-plugin-smoke
sec4 check --path examples/zed-plugin-smoke
```

## Zed smoke checklist

Open `$REPO_ROOT/examples/zed-plugin-smoke` in Zed.

1. **Diagnostics stream**
- Open `playground/lsp-errors.ut`.
- You should see an effect diagnostic (missing declared effect for `log.info`).
- Use the quickfix to insert the missing effect declaration.

2. **Format document**
- Open `playground/format-me.ut`.
- Trigger **Format Document**.
- Expected output:

```ut
fn sample() -> Int {
  0
}
```

3. **Go to definition**
- Open `src/main.ut`.
- On `healthHandler`, run Go to Definition.
- It should jump to `src/http/handlers.ut`.

4. **Find references + rename**
- In `src/auth/access.ut`, place cursor on `canReadUsers`.
- Find references should include `src/http/handlers.ut`.
- Rename symbol and verify both files update.

5. **Hover/completion**
- In `src/http/handlers.ut`, test hover and completion on builtins like `req.pathParam` and `res.text`.

## Optional runtime check

```bash
sec4 run --path examples/zed-plugin-smoke --backend lasm --oneshot --port 8080
curl -i http://127.0.0.1:8080/health
```

