# 1045 M39 Slice: Zed Extension Bundle Staging Helper

This slice adds deterministic local publish-bundle staging for the Zed extension.

## What changed

1. Added `scripts/stage-zed-extension-bundle.sh`.
2. The script now stages extension source under:
   - `build/zed-extension-bundle/<id>-<version>/`
3. The script emits:
   - `build/zed-extension-bundle/bundle-manifest.json`
4. Manifest entries include deterministic per-file metadata:
   - `path`
   - `size`
   - `sha256`
5. Updated `zed-extension/README.md` to include bundle staging command for operator use.

## Why

Release checks validated correctness, but there was no canonical local staging artifact for packaging review. This slice creates a stable bundle snapshot and hash inventory so release prep can be reviewed and reproduced.

## Validation

- `bash -n scripts/stage-zed-extension-bundle.sh`
- `scripts/stage-zed-extension-bundle.sh --clean`
- `test -f build/zed-extension-bundle/bundle-manifest.json`

## Notes

- `--clean` removes previously staged bundle directory before staging.
- The hash tool uses `sha256sum` when available, with `shasum -a 256` fallback.
