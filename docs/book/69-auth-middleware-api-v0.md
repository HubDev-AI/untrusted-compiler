# 69 Auth Middleware API (v0)

This chapter defines minimal, policy-driven auth middleware that integrates with security audit and CSRF rules.

## 1) Types

```ut
type AuthMode = "token" | "cookie" | "mixed"

type AuthConfig = {
  enabled: Bool
  mode: AuthMode

  crossSiteFrontend: Bool

  token?: {
    headerName: HeaderName
    scheme: "bearer"
  }

  cookie?: {
    cookieName: String
    sameSite: "Lax" | "Strict" | "None"
    secure: Bool
    httpOnly: Bool
  }
}
```

## 2) Principal model

```ut
type Principal = {
  userId: String
  role?: String
  scopes?: List<String>
}
```

## 3) Context integration

```ut
type Ctx = {
  ...
  principal?: Principal
}
```

## 4) Middleware API

```ut
module auth {
  fn fromPolicy() -> AuthConfig
  fn withAuth(r: Router, cfg: AuthConfig) -> Router
}
```

Behavior:
- attach principal to request context when authenticated
- emit middleware tag `middleware.auth` with effective attrs

## 5) Handler helpers

```ut
module auth {
  fn require(ctx: Ctx) -> Result<Principal, HttpError>
  fn requireRole(ctx: Ctx, role: String) -> Result<Principal, HttpError>
}
```

## 6) Security and audit requirements
- auth middleware must not log raw tokens/cookies
- capture/redaction rules apply to auth material by default
- middleware tag attrs should include: mode, crossSiteFrontend, cookieEnabled, tokenEnabled
- `sec4 audit` uses these attrs to determine CSRF requirements and cross-site cookie safety findings
