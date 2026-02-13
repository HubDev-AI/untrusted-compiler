# 59 Request Capture and Deterministic Replay (v0)

This chapter defines capture/replay behavior for deterministic debugging under security constraints.

## 1) Goals
- deterministic reproduction of request-handling failures
- safe capture storage (redacted by default)
- trace linkage (`traceId` + `captureId`)
- replay that obeys policy, budget, and effect restrictions

## 2) Capture controls

Policy-controlled modes:
- `capture.mode = "off" | "errors" | "sample" | "all"`
- `capture.sample_rate = 0..1`
- `capture.max_body_bytes`

Identifiers:
- `captureId`: unique capture identifier
- `traceId`: request correlation id

If captured, logs should include `captureId`.

## 3) Canonical capture format

```json
{
  "version": "0.1",
  "captureId": "cap_01HZY...",
  "traceId": "01HZY...ABC",
  "timeMs": 1760000000000,
  "policyHash": "pol_...",
  "compilerHash": "cpl_...",
  "runtimeHash": "rt_...",
  "request": { },
  "env": { },
  "determinism": { },
  "redaction": { }
}
```

### Request section
- method/scheme/host/path/query
- normalized route (if known)
- redacted headers
- bounded body capture (bytes, hash, truncated marker)

### Environment section
Capture only determinism-relevant values (timezone/locale + explicit allowlist).

### Determinism section
- seeded randomness
- frozen time
- deterministic UUID mode
- active budget snapshot

### Redaction section
Must record what was redacted (headers/json paths/notes).

## 4) Redaction rules

Always redact:
- `Authorization`
- `Cookie`
- `Set-Cookie`
- common token headers
- policy-defined redact headers/JSON paths

Rules:
- never persist `Secret<T>` values
- JSON body redaction by configured paths
- non-JSON bodies can be hash-only by default

## 5) Replay model

CLI shape:
- `sec4 replay <capture.json> [--override-budget ...]`

Replay runtime responsibilities:
- reconstruct synthetic request
- inject deterministic providers (time/random/uuid)
- enforce captured budget (or explicit override policy)

External side effects in replay:
- `replay.effects = "deny" | "mock" | "allow"` (default `deny`)

`deny` mode should withhold sensitive capabilities so effectful calls fail predictably.

## 6) Optional IO stubbing

Optional recorded stubs for outbound net/DB calls can support mock replay mode.

## 7) Replay errors

- `REPLAY.CAPTURE_INVALID`
- `REPLAY.BODY_TRUNCATED`
- `REPLAY.EFFECT_FORBIDDEN`
- `REPLAY.STUB_MISSING`
- `REPLAY.POLICY_MISMATCH`

## 8) Logging requirements

On live capture:
- include `captureId` in request and error logs.

On replay:
- log `replay.start` and `replay.end` with capture metadata.

## 9) Invariants and limits

- enforce max capture size (`capture.max_capture_bytes`)
- include body/file hashes where available
- prefer append-only or content-addressed capture storage

## 10) Minimal v0 implementation checklist
1. capture bounded request+headers+body
2. redact headers/JSON paths
3. record seed/time/budget
4. persist capture JSON with IDs/hashes
5. replay with deterministic providers
6. default deny external effects via capability withholding
7. log `traceId` + `captureId` linkage
