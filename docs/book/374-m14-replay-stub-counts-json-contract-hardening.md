# M14 Slice: Replay StubCounts JSON Contract Hardening

This slice extends replay JSON contract guards so `stubCounts` remains a locked output field.

## What it is

Updated:
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/368-m14-replay-cli-json-output-mode.md`
- `docs/book/370-m14-replay-cli-json-contract-closure-gate.md`

## Why it exists

`stubCounts` is now part of replay observability. Without explicit contract enforcement, that field could regress and break downstream CI/report consumers.

## What changed

1. Contract test now requires `stubCounts`
- Replay JSON contract checker now fails if the `stubCounts` key is absent.
- Checker also keeps core replay payload keys locked (`ok`, `capture`, `stubs`, hash-match flags, `allowPolicyMismatch`, `effectsMode`, `warnings`).

2. Guard fixtures now test `stubCounts` drift
- Passing fixtures include `stubCounts`.
- Added failing fixture that omits `stubCounts`.

3. Docs alignment
- Replay JSON output and closure-gate chapters now include `stubCounts` in required payload keys.

## Validation

```bash
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
```

## Tradeoffs

- This is static contract enforcement over source shape, not runtime replay execution behavior.
- Runtime correctness is still validated in CLI integration tests.
