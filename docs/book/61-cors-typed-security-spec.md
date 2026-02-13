# 61 CORS Typed Security Spec (v0)

CORS is a first-class attack surface for web backends and must be explicit, typed, and policy-governed.

## 1) Typed CORS config

```ut
type CorsConfig = {
  allowedOrigins: CorsOrigins
  allowedMethods: List<HttpMethod>
  allowedHeaders: List<HeaderName>
  exposedHeaders?: List<HeaderName>
  allowCredentials: Bool
  maxAgeSeconds?: Int
}

type CorsOrigins =
  | { any: true }
  | { exact: List<Origin> }
  | { pattern: List<OriginPattern> }

type Origin = opaque
```

Origin gate:

```ut
module cors {
  fn origin(u: Untrusted<String>) -> Result<Origin, ValidationError>
}
```

## 2) Middleware contract

```ut
module cors {
  fn withCors(r: Router, cfg: CorsConfig) -> Router
}
```

Required runtime behavior:
- preflight (`OPTIONS`) handling
- correct allow headers/methods/origin outputs
- `Vary: Origin` when origin is not wildcard

## 3) Unsafe configuration prevention

Hard security rules:
- if `allowCredentials=true`, wildcard origin is forbidden.
- origin reflection without allowlist/pattern is forbidden when policy disallows it.

## 4) Policy integration

Required policy keys are described in `docs/book/60-v0-policy-keys-spec.md` under `[cors]`.

Compiler/runtime behavior:
- forbid any-origin when policy demands it
- enforce `Vary: Origin` requirement
- enforce credentials/origin compatibility

## 5) Optional effect visibility

CORS configuration can remain policy-checked config only (simplest v0 path), or be represented by a dedicated security-config effect in future versions.

## 6) Why this is mandatory

Misconfigured CORS can cause cross-origin credential leakage and unintended browser-mediated access to protected endpoints. Treating CORS as typed middleware + policy prevents ad-hoc insecure header handling.
