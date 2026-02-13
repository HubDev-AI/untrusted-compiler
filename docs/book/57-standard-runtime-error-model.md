# 57 Standard Runtime Error Model (v0)

This chapter defines the runtime error contract for HTTP, logs, and replay/debug workflows.

## Goals

Every runtime error should be:
- structured and machine-parseable
- stable (`code` is contract)
- traceable (`traceId` and optional debug span data)
- safe (no secret leakage)

## 1) Canonical runtime error type

```ut
type ErrorCode = String

type ErrorKind =
  | "validation"
  | "auth"
  | "not_found"
  | "conflict"
  | "rate_limit"
  | "resource_limit"
  | "dependency"
  | "timeout"
  | "internal"
  | "cancelled"
  | "policy"

type ErrorDetailValue =
  | String
  | Int64
  | Bool
  | List<ErrorDetailValue>
  | Map<String, ErrorDetailValue>

type ErrorDetail = { key: String, value: ErrorDetailValue }

type StdError = {
  code: ErrorCode
  kind: ErrorKind
  message: String
  status: Int
  traceId: String
  timeMs: Int64
  path?: String
  field?: String
  limit?: { name: String, max: Int64, actual?: Int64 }
  dependency?: { name: String, operation?: String, retryable?: Bool }
  details?: List<ErrorDetail>
  cause?: StdError
  debug?: {
    file?: String
    line?: Int
    column?: Int
    spanId?: String
    replayHint?: String
  }
}
```

Safety rule:
- `StdError` fields must not carry secrets; any accidental secret-like values must be redacted.

## 2) HTTP error envelope

Standard response shape:

```json
{
  "error": {
    "code": "JSON.FIELD_INVALID",
    "kind": "validation",
    "message": "Invalid value for field 'email'.",
    "status": 400,
    "traceId": "01HZY...ABC",
    "timeMs": 1760000000000,
    "path": "$.email",
    "details": [
      {"key": "expected", "value": "Email"},
      {"key": "validator", "value": "validate.email"}
    ]
  }
}
```

Rules:
- top-level key is always `error`
- include `traceId` always
- no stack traces in client responses by default

## 3) Status mapping defaults

- validation -> 400
- auth -> 401/403
- not_found -> 404
- conflict -> 409
- rate_limit -> 429
- resource_limit -> 413/429/503 depending on policy
- timeout -> 504/408
- dependency -> 502/503
- policy -> 403 (or internal mapping)
- internal -> 500
- cancelled -> 499/408

## 4) Standard code namespaces

Examples:
- `JSON.INVALID_SYNTAX`, `JSON.FIELD_INVALID`, `JSON.DEPTH_LIMIT`
- `VALIDATION.EMAIL_INVALID`
- `AUTH.UNAUTHORIZED`, `AUTH.FORBIDDEN`
- `LIMIT.BODY_BYTES`, `LIMIT.JSON_DEPTH`
- `NET.SSRF_BLOCKED`, `NET.REDIRECT_FORBIDDEN`
- `DB.TIMEOUT`, `DB.CONSTRAINT_VIOLATION`
- `POLICY.FORBIDDEN_EFFECT`

## 5) Constructor API

```ut
module err {
  fn validation(code: String, message: String) -> StdError
  fn auth(code: String, message: String, status: Int) -> StdError
  fn notFound(code: String, message: String) -> StdError
  fn conflict(code: String, message: String) -> StdError
  fn rateLimit(code: String, message: String, retryAfterMs: Int64) -> StdError
  fn internal(message: String) -> StdError

  fn withPath(e: StdError, path: String) -> StdError
  fn withDetail(e: StdError, key: String, value: ErrorDetailValue) -> StdError
  fn withLimit(e: StdError, name: String, max: Int64, actual?: Int64) -> StdError
  fn withDependency(e: StdError, name: String, operation?: String, retryable?: Bool) -> StdError
  fn withCause(e: StdError, cause: StdError) -> StdError
}
```

Typechecker rule:
- `Secret<_>` cannot be passed into `withDetail` values.

## 6) Handler integration

Recommended handler shape:

```ut
type HttpError = StdError
type Handler = fn(Ctx, Request) -> Result<Response, HttpError>
```

Runtime wrapper behavior:
- `Err(e)` maps to standard error envelope + status
- `traceId` propagated to both body and response headers

## 7) JSON path standard

Use deterministic path subset:
- `$`
- `$.field`
- `$.items[3]`

## 8) Debug vs release behavior

Release:
- expose safe fields only (`code`, `kind`, `message`, `status`, `traceId`, `timeMs`, optional path/limit)

Debug:
- may include `debug` and bounded cause chain
- may include replay hint/capture ID when enabled

## 9) Correlation and replay

- Create `traceId` at request entry and propagate through logs/errors.
- If replay capture exists, expose capture hint only in debug-controlled channels.
